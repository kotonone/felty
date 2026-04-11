use std::{env, fs, process};

use crate::util::open;

pub fn open_webview_manual(base_url: &str, update: bool) {
    let url = if base_url.contains('?') {
        format!("{}&os={}{}", base_url, env::consts::OS, if update { "&update" } else { "" })
    } else {
        format!("{}?os={}{}", base_url, env::consts::OS, if update { "&update" } else { "" })
    };
    open(&url);
}

/// カレントディレクトリをゲームのルートディレクトリに移動します。
pub fn set_current_dir() {
    env::set_current_dir(env::current_exe().unwrap().parent().unwrap().to_path_buf()).unwrap();
}

/// 実行されている環境において、WebView を正常に使用できるかどうかチェックします。
///
/// 正常に使用できない場合、サポートページを開きます。
pub fn check_webview(webview_install_url: &str) {
    if wry::webview_version().is_err() {
        log::warn!("Could not use WebView");
        open_webview_manual(webview_install_url, false);
        process::exit(1);
    }
}

/// キャッシュファイルを削除します。
pub fn process_cleaning(cache_directory: &str) {
    let executable_path = env::current_exe().unwrap();
    let directory = executable_path.parent().unwrap();
    if let Err(e) = fs::remove_dir_all(directory.join(cache_directory)) {
        log::warn!("Failed to clean cache: {}", e);
    }
}
