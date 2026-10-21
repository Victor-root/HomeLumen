//! Whether the desktop is set to light or dark, every time that changes.

use iced::Subscription;
use iced::theme::Mode;

/// Every change of the desktop between its light and dark settings.
#[cfg(not(target_os = "windows"))]
pub fn changes() -> Subscription<Mode> {
    iced::system::theme_changes()
}

/// Winit asks Windows through an undocumented call that keeps answering
/// with the setting the program started under, so it never reports a
/// change. The setting itself lives in the registry, watched here instead.
#[cfg(target_os = "windows")]
pub fn changes() -> Subscription<Mode> {
    Subscription::run(personalize::watch)
}

#[cfg(target_os = "windows")]
mod personalize {
    use std::{ptr, thread};

    use iced::futures::Stream;
    use iced::futures::channel::mpsc;
    use iced::theme::Mode;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_NOTIFY, KEY_QUERY_VALUE,
        REG_NOTIFY_CHANGE_LAST_SET, RRF_RT_REG_DWORD, RegCloseKey,
        RegGetValueW, RegNotifyChangeKeyValue, RegOpenKeyExW,
    };
    use windows_sys::core::w;

    /// Sends the current setting, then the new one on every change.
    pub fn watch() -> impl Stream<Item = Mode> {
        let (sender, receiver) = mpsc::unbounded();
        thread::spawn(move || follow(&sender));
        receiver
    }

    /// Blocks its own thread between changes, for as long as anyone listens.
    fn follow(sender: &mpsc::UnboundedSender<Mode>) {
        let Some(key) = open() else {
            return;
        };

        let mut last = None;

        loop {
            let mode = read(key);

            if last != Some(mode) {
                if sender.unbounded_send(mode).is_err() {
                    break;
                }
                last = Some(mode);
            }

            // SAFETY: `key` is open with `KEY_NOTIFY`; without an event
            // the call simply returns once a value under it changes.
            let status = unsafe {
                RegNotifyChangeKeyValue(
                    key,
                    0,
                    REG_NOTIFY_CHANGE_LAST_SET,
                    ptr::null_mut(),
                    0,
                )
            };

            if status != ERROR_SUCCESS {
                break;
            }
        }

        // SAFETY: `key` was opened by `open` and is not used past here.
        unsafe { RegCloseKey(key) };
    }

    fn open() -> Option<HKEY> {
        let mut key = ptr::null_mut();

        // SAFETY: both strings are null-terminated literals and `key` is a
        // valid place for the handle.
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                w!(
                    "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"
                ),
                0,
                KEY_QUERY_VALUE | KEY_NOTIFY,
                &raw mut key,
            )
        };

        (status == ERROR_SUCCESS).then_some(key)
    }

    fn read(key: HKEY) -> Mode {
        let mut light = 0u32;
        let mut size = size_of::<u32>() as u32;

        // SAFETY: `key` is open with `KEY_QUERY_VALUE`, and `light` holds
        // exactly the `size` bytes a `REG_DWORD` is restricted to.
        let status = unsafe {
            RegGetValueW(
                key,
                ptr::null(),
                w!("AppsUseLightTheme"),
                RRF_RT_REG_DWORD,
                ptr::null_mut(),
                (&raw mut light).cast(),
                &raw mut size,
            )
        };

        match (status, light) {
            (ERROR_SUCCESS, 0) => Mode::Dark,
            (ERROR_SUCCESS, _) => Mode::Light,
            _ => Mode::None,
        }
    }
}
