// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use felty::*;
use felty::app::Responder;

fn main() {
    core::set_current_dir();

    let (config, icon) = load_config!();
    app::setup_log();

    core::process_cleaning(&config.cache_directory);
    core::check_webview(&config.webview_install_url);

    app::FeltyApp::new(config)
        .with_icon(icon)
        .on_setup(|app_handle| {
            let handle_clone = app_handle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let _ = handle_clone.evaluate_script("console.log('Hello from Rust background thread!');");
                let _ = handle_clone.dispatch(|webview| {
                    let _ = webview.zoom(1.5);
                });
            });
        })
        .on_before_run(|| {
            // 独自のアップデートチェック処理など
        })
        .on_custom_protocol_request(|request, responder| {
            // カスタムプロトコルのインターセプト（アセット復号など）
            if request.uri().path().ends_with(".encrypted") {
                // let decrypted_data = my_decrypt_function(request);
                // responder.respond_with(decrypted_data);

                // ここではダミーで404を返す例
                responder.respond_with(http::StatusCode::NOT_FOUND);
                Ok(()) // 標準の処理をスキップ
            } else {
                Err((request, responder)) // 標準のルーター(Vite等)にフォールバック
            }
        })
        .on_menu_event(|event_id| {
            if event_id == "my_custom_menu_id" {
                println!("独自のメニューが押されました！");
                false // 標準のハンドラーをスキップ
            } else {
                true // 標準のハンドラーを実行
            }
        })
        .run();
}
