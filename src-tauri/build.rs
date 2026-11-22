fn main() {
    // `MM_CHECK_ONLY=1 cargo check --target x86_64-pc-windows-msvc` type-checks the
    // Windows code on a Mac (the Windows resource compiler isn't available there).
    if std::env::var("MM_CHECK_ONLY").is_ok() {
        return;
    }
    tauri_build::build()
}
