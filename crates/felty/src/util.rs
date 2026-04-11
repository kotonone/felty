/// 指定された URL を開きます。
pub fn open(url: &str) {
    #[cfg(windows)] {
        use windows::{
            core::{w, PCWSTR},
            Win32::UI::Shell::ShellExecuteW,
            Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        };

        let lpfile_str = url
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        let lpfile_ptr = lpfile_str.as_ptr();

        unsafe {
            ShellExecuteW(
                None,
                w!("open"),
                PCWSTR(lpfile_ptr),
                None,
                None,
                SW_SHOWNORMAL,
            );
        }
    }

    // TODO: macOS
}

