//! System audio: volume, mute, microphone, per-app volume (Windows), output device.

pub use imp::*;

// ============================================================== macOS
#[cfg(target_os = "macos")]
mod imp {
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::ptr;

    use core_foundation::base::TCFType;
    use core_foundation::string::{CFString, CFStringRef};

    type ObjId = u32;

    #[repr(C)]
    struct Addr {
        sel: u32,
        scope: u32,
        elem: u32,
    }

    const fn fcc(s: &[u8; 4]) -> u32 {
        ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | s[3] as u32
    }
    const SYSTEM: ObjId = 1;
    const GLOBAL: u32 = fcc(b"glob");
    const OUTPUT: u32 = fcc(b"outp");
    const INPUT: u32 = fcc(b"inpt");
    const DEFAULT_OUT: u32 = fcc(b"dOut");
    const DEFAULT_IN: u32 = fcc(b"dIn ");
    const DEVICES: u32 = fcc(b"dev#");
    const NAME: u32 = fcc(b"lnam");
    const STREAMS: u32 = fcc(b"stm#");
    const VIRTUAL_VOL: u32 = fcc(b"vmvc");
    const VOLUME_SCALAR: u32 = fcc(b"volm");
    const MUTE: u32 = fcc(b"mute");

    #[link(name = "CoreAudio", kind = "framework")]
    extern "C" {
        fn AudioObjectGetPropertyDataSize(id: ObjId, a: *const Addr, qs: u32, qd: *const c_void, size: *mut u32) -> i32;
        fn AudioObjectGetPropertyData(id: ObjId, a: *const Addr, qs: u32, qd: *const c_void, size: *mut u32, data: *mut c_void) -> i32;
        fn AudioObjectSetPropertyData(id: ObjId, a: *const Addr, qs: u32, qd: *const c_void, size: u32, data: *const c_void) -> i32;
        fn AudioObjectHasProperty(id: ObjId, a: *const Addr) -> u8;
    }
    // The "virtual main volume" (what the volume keys change) lives in AudioToolbox.
    #[link(name = "AudioToolbox", kind = "framework")]
    extern "C" {
        fn AudioHardwareServiceSetPropertyData(id: ObjId, a: *const Addr, qs: u32, qd: *const c_void, size: u32, data: *const c_void) -> i32;
        fn AudioHardwareServiceHasProperty(id: ObjId, a: *const Addr) -> u8;
    }

    fn get<T: Copy + Default>(id: ObjId, sel: u32, scope: u32, elem: u32) -> Option<T> {
        let a = Addr { sel, scope, elem };
        let mut v = T::default();
        let mut size = size_of::<T>() as u32;
        let r = unsafe { AudioObjectGetPropertyData(id, &a, 0, ptr::null(), &mut size, &mut v as *mut T as *mut c_void) };
        (r == 0).then_some(v)
    }

    fn set<T: Copy>(id: ObjId, sel: u32, scope: u32, elem: u32, v: T) -> bool {
        let a = Addr { sel, scope, elem };
        unsafe { AudioObjectSetPropertyData(id, &a, 0, ptr::null(), size_of::<T>() as u32, &v as *const T as *const c_void) == 0 }
    }

    fn default_device(input: bool) -> Result<ObjId, String> {
        get::<ObjId>(SYSTEM, if input { DEFAULT_IN } else { DEFAULT_OUT }, GLOBAL, 0)
            .filter(|d| *d != 0)
            .ok_or_else(|| "no default audio device".into())
    }

    fn set_volume(dev: ObjId, scope: u32, v: f64) -> Result<(), String> {
        let v = v.clamp(0.0, 1.0) as f32;
        let a = Addr { sel: VIRTUAL_VOL, scope, elem: 0 };
        unsafe {
            if AudioHardwareServiceHasProperty(dev, &a) != 0
                && AudioHardwareServiceSetPropertyData(dev, &a, 0, ptr::null(), 4, &v as *const f32 as *const c_void) == 0
            {
                return Ok(());
            }
        }
        // Fallback: set every channel that has its own volume
        let mut any = false;
        for elem in 0..=8 {
            let a = Addr { sel: VOLUME_SCALAR, scope, elem };
            if unsafe { AudioObjectHasProperty(dev, &a) } != 0 {
                any |= set(dev, VOLUME_SCALAR, scope, elem, v);
            }
        }
        if any { Ok(()) } else { Err("this audio device has no volume control".into()) }
    }

    fn toggle_mute(dev: ObjId, scope: u32) -> Result<(), String> {
        let m = get::<u32>(dev, MUTE, scope, 0).ok_or("this audio device can't be muted")?;
        if set(dev, MUTE, scope, 0, (m == 0) as u32) { Ok(()) } else { Err("mute failed".into()) }
    }

    pub fn set_output_volume(v: f64) -> Result<(), String> {
        set_volume(default_device(false)?, OUTPUT, v)
    }
    pub fn toggle_output_mute() -> Result<(), String> {
        toggle_mute(default_device(false)?, OUTPUT)
    }
    pub fn set_input_volume(v: f64) -> Result<(), String> {
        set_volume(default_device(true)?, INPUT, v)
    }
    pub fn toggle_input_mute() -> Result<(), String> {
        toggle_mute(default_device(true)?, INPUT)
    }
    pub fn set_app_volume(_app: &str, _v: f64) -> Result<(), String> {
        Err("Per-app volume isn't possible on macOS without an extra audio driver".into())
    }

    fn device_name(dev: ObjId) -> String {
        let a = Addr { sel: NAME, scope: GLOBAL, elem: 0 };
        let mut s: CFStringRef = ptr::null();
        let mut size = size_of::<CFStringRef>() as u32;
        let r = unsafe { AudioObjectGetPropertyData(dev, &a, 0, ptr::null(), &mut size, &mut s as *mut CFStringRef as *mut c_void) };
        if r != 0 || s.is_null() {
            return format!("device {dev}");
        }
        unsafe { CFString::wrap_under_create_rule(s) }.to_string()
    }

    fn output_devices() -> Vec<ObjId> {
        let a = Addr { sel: DEVICES, scope: GLOBAL, elem: 0 };
        let mut size = 0u32;
        if unsafe { AudioObjectGetPropertyDataSize(SYSTEM, &a, 0, ptr::null(), &mut size) } != 0 {
            return vec![];
        }
        let mut ids = vec![0 as ObjId; size as usize / size_of::<ObjId>()];
        if unsafe { AudioObjectGetPropertyData(SYSTEM, &a, 0, ptr::null(), &mut size, ids.as_mut_ptr() as *mut c_void) } != 0 {
            return vec![];
        }
        ids.into_iter()
            .filter(|&d| {
                let a = Addr { sel: STREAMS, scope: OUTPUT, elem: 0 };
                let mut s = 0u32;
                unsafe { AudioObjectGetPropertyDataSize(d, &a, 0, ptr::null(), &mut s) == 0 && s > 0 }
            })
            .collect()
    }

    /// Switch to the next output device; returns its name.
    pub fn next_output_device() -> Result<String, String> {
        let devs = output_devices();
        if devs.is_empty() {
            return Err("no output devices".into());
        }
        let cur = default_device(false).unwrap_or(0);
        let i = devs.iter().position(|&d| d == cur).map_or(0, |i| (i + 1) % devs.len());
        if set(SYSTEM, DEFAULT_OUT, GLOBAL, 0, devs[i]) { Ok(device_name(devs[i])) } else { Err("couldn't switch output".into()) }
    }
}

// ============================================================== Windows
#[cfg(target_os = "windows")]
mod imp {
    use windows::core::{Interface, PWSTR};
    use windows::Win32::Foundation::{CloseHandle, BOOL};
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::Media::Audio::{
        eCapture, eMultimedia, eRender, IAudioSessionControl2, IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator,
        ISimpleAudioVolume, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};
    use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};

    fn e(err: windows::core::Error) -> String {
        err.message().to_string()
    }

    fn device(capture: bool) -> Result<IMMDevice, String> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let en: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(e)?;
            en.GetDefaultAudioEndpoint(if capture { eCapture } else { eRender }, eMultimedia).map_err(e)
        }
    }

    fn endpoint(capture: bool) -> Result<IAudioEndpointVolume, String> {
        unsafe { device(capture)?.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).map_err(e) }
    }

    pub fn set_output_volume(v: f64) -> Result<(), String> {
        unsafe { endpoint(false)?.SetMasterVolumeLevelScalar(v.clamp(0.0, 1.0) as f32, std::ptr::null()).map_err(e) }
    }
    pub fn set_input_volume(v: f64) -> Result<(), String> {
        unsafe { endpoint(true)?.SetMasterVolumeLevelScalar(v.clamp(0.0, 1.0) as f32, std::ptr::null()).map_err(e) }
    }
    fn toggle(capture: bool) -> Result<(), String> {
        unsafe {
            let ep = endpoint(capture)?;
            let m = ep.GetMute().map_err(e)?;
            ep.SetMute(BOOL::from(!m.as_bool()), std::ptr::null()).map_err(e)
        }
    }
    pub fn toggle_output_mute() -> Result<(), String> {
        toggle(false)
    }
    pub fn toggle_input_mute() -> Result<(), String> {
        toggle(true)
    }

    fn process_name(pid: u32) -> Option<String> {
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut buf = [0u16; 512];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
            let _ = CloseHandle(h);
            if !ok {
                return None;
            }
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            Some(path.rsplit('\\').next().unwrap_or(&path).to_lowercase())
        }
    }

    /// `app` matches the program's file name, with or without ".exe" (e.g. "spotify" or "Discord.exe").
    pub fn set_app_volume(app: &str, v: f64) -> Result<(), String> {
        let want = app.trim().to_lowercase();
        let want = want.strip_suffix(".exe").unwrap_or(&want).to_string();
        let mut found = false;
        unsafe {
            let mgr: IAudioSessionManager2 = device(false)?.Activate(CLSCTX_ALL, None).map_err(e)?;
            let list = mgr.GetSessionEnumerator().map_err(e)?;
            for i in 0..list.GetCount().map_err(e)? {
                let Ok(ctl) = list.GetSession(i) else { continue };
                let Ok(ctl2) = ctl.cast::<IAudioSessionControl2>() else { continue };
                let Ok(pid) = ctl2.GetProcessId() else { continue };
                let Some(name) = process_name(pid) else { continue };
                if name.strip_suffix(".exe").unwrap_or(&name) == want {
                    if let Ok(vol) = ctl.cast::<ISimpleAudioVolume>() {
                        let _ = vol.SetMasterVolume(v.clamp(0.0, 1.0) as f32, std::ptr::null());
                        found = true;
                    }
                }
            }
        }
        if found { Ok(()) } else { Err(format!("{app} isn't playing audio right now")) }
    }

    pub fn next_output_device() -> Result<String, String> {
        Err("Switching the output device isn't supported on Windows yet".into())
    }
}

// ============================================================== other
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod imp {
    fn no<T>() -> Result<T, String> {
        Err("not supported on this system".into())
    }
    pub fn set_output_volume(_: f64) -> Result<(), String> { no() }
    pub fn toggle_output_mute() -> Result<(), String> { no() }
    pub fn set_input_volume(_: f64) -> Result<(), String> { no() }
    pub fn toggle_input_mute() -> Result<(), String> { no() }
    pub fn set_app_volume(_: &str, _: f64) -> Result<(), String> { no() }
    pub fn next_output_device() -> Result<String, String> { no() }
}
