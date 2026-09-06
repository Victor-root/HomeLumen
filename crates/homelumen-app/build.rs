//! Embeds the application icon and a DPI-awareness manifest into the `.exe`
//! itself on Windows.
//!
//! `icon::window` sets the running window's icon at start-up, which is
//! enough for the title bar and the taskbar's own icon, but not for
//! Explorer or the taskbar's right-click jump list: both read the icon
//! straight off the executable's own resources, before HomeLumen ever
//! runs. `assets/icon/icon.ico` is a static export of exactly what
//! `icon::window` draws; if the mark in `src/icon.rs` ever changes,
//! regenerate it with the same pixels so the two never drift apart.
//!
//! Without a manifest saying otherwise, Windows assumes an old,
//! DPI-unaware program and stretches its whole window as a bitmap on a
//! high-density screen, the blur this exists to avoid. Declaring
//! per-monitor DPI awareness instead lets Winit report the screen's real
//! scale factor, so HomeLumen renders sharp wherever the window ends up on
//! a multi-monitor setup.
const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
  <asmv3:application>
    <asmv3:windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </asmv3:windowsSettings>
  </asmv3:application>
</assembly>
"#;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("../../assets/icon/icon.ico")
            .set_manifest(MANIFEST)
            .compile()
            .expect("embedding the Windows icon and manifest failed");
    }
}
