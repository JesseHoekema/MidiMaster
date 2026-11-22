//! Screen brightness.
//! macOS: built-in display (and Apple displays) via DisplayServices.
//! Windows: laptop screen via WMI, external monitors via DDC/CI (if the monitor allows it).

pub use imp::set_brightness;

#[cfg(target_os = "macos")]
mod imp {
    use once_cell::sync::Lazy;

    type SetFn = unsafe extern "C" fn(u32, f32) -> i32;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGGetActiveDisplayList(max: u32, displays: *mut u32, count: *mut u32) -> i32;
    }

    static SET: Lazy<Option<SetFn>> = Lazy::new(|| unsafe {
        let h = libc::dlopen(c"/System/Library/PrivateFrameworks/DisplayServices.framework/DisplayServices".as_ptr(), libc::RTLD_LAZY);
        if h.is_null() {
            return None;
        }
        let s = libc::dlsym(h, c"DisplayServicesSetBrightness".as_ptr());
        (!s.is_null()).then(|| std::mem::transmute::<*mut libc::c_void, SetFn>(s))
    });

    pub fn set_brightness(v: f64) -> Result<(), String> {
        let set = SET.ok_or("brightness control isn't available on this Mac")?;
        let mut ids = [0u32; 16];
        let mut n = 0u32;
        unsafe { CGGetActiveDisplayList(16, ids.as_mut_ptr(), &mut n) };
        let mut ok = false;
        for &d in &ids[..n as usize] {
            ok |= unsafe { set(d, v.clamp(0.0, 1.0) as f32) } == 0;
        }
        if ok { Ok(()) } else { Err("no display here allows brightness control".into()) }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use ddc_hi::{Ddc, Display};
    use std::os::windows::process::CommandExt;

    pub fn set_brightness(v: f64) -> Result<(), String> {
        let pct = (v.clamp(0.0, 1.0) * 100.0).round() as u16;
        let mut ok = false;
        // external monitors
        for mut d in Display::enumerate() {
            ok |= d.handle.set_vcp_feature(0x10, pct).is_ok();
        }
        // laptop screen
        let status = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods | Select -First 1 | Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{{Timeout=1;Brightness={pct}}}) | Out-Null"),
            ])
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
            .status();
        ok |= status.map(|s| s.success()).unwrap_or(false);
        if ok { Ok(()) } else { Err("no display here allows brightness control".into()) }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod imp {
    pub fn set_brightness(_: f64) -> Result<(), String> {
        Err("not supported on this system".into())
    }
}
