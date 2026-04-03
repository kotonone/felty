use std::{env, fs, process};

use waitpid_any::WaitHandle;

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

/// 引数に `--wait-pid [PID]` が設定されている場合、指定された PID を持つプロセスを待機します。
pub fn process_waiting() {
    if let Some(argument_position) = env::args().position(|a| a == "--wait-pid") {
        let pid = env::args().nth(argument_position + 1).expect("PID must be specified").parse::<i32>().expect("PID must be specified in number");
        log::debug!("Waiting for PID {} to exit", pid);
        if let Ok(mut process) = WaitHandle::open(pid) {
            process.wait().expect("Could not wait process");
        }
    }
}

/// キャッシュファイルを削除します。
///
/// 引数に `--delete-caches-only` が指定されている場合、キャッシュを削除した後終了します。
pub fn process_cleaning(cache_directory: &str) {
    let executable_path = env::current_exe().unwrap();
    let directory = executable_path.parent().unwrap();
    let _ = fs::remove_dir_all(directory.join(cache_directory));

    if env::args().position(|a| a == "--delete-caches-only").is_some() {
        process::exit(0);
    }
}
