//! Embeds the application icon into the `.exe` itself on Windows.
//!
//! `icon::window` sets the running window's icon at start-up, which is
//! enough for the title bar and the taskbar's own icon, but not for
//! Explorer or the taskbar's right-click jump list: both read the icon
//! straight off the executable's own resources, before HomeLumen ever
//! runs. `assets/icon.ico` is a static export of exactly what `icon::window`
//! draws; if the mark in `src/icon.rs` ever changes, regenerate it with the
//! same pixels so the two never drift apart.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()
            .expect("embedding the Windows icon resource failed");
    }
}
