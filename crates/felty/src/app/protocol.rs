mod responder;

pub use responder::Responder;

use std::{fs, path::PathBuf, process};

use http::{Method, Request, StatusCode};
use wry::RequestAsyncResponder;

use crate::core::open_webview_manual;

/// MIME タイプを予測します。
fn guess_mime(path: &str) -> &str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "text/javascript"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".txt") || path.ends_with(".yaml") || path.ends_with(".yml") || path.ends_with(".toml") {
        "text/plain"
    } else if path.ends_with(".mp3") {
        "audio/mpeg"
    } else if path.ends_with(".flac") {
        "audio/flac"
    } else if path.ends_with(".ogg") {
        "audio/ogg"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else if path.ends_with(".otf") {
        "font/otf"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else if path.ends_with(".glb") {
        "model/gltf-binary"
    } else if path.ends_with(".gltf") {
        "model/gltf+json"
    } else if path.ends_with(".obj") {
        "model/obj"
    } else {
        "application/octet-stream"
    }
}

/// カスタムプロトコルを使用した URL を、パッケージおよびパスに変換します。
///
/// OS やバックエンドの WebView (WebView2 / WebKit) の仕様により、
/// 要求される URL のスキーマやホスト名が異なるパターンが存在するため、複数のパターンでフォールバック処理を行います。
///
/// また、URL のパスに `package_prefix` が含まれるかどうかでパッケージを判別します:
/// - ルートへのアクセス (例: `protocol://host/index.html`) -> `(runtime_package, "index.html")`
/// - 別パッケージへのアクセス (例: `protocol://host/package_prefix/Assets.pak/img.png`) -> `("Assets.pak", "img.png")`
pub fn to_package_and_path(url: &str) -> Option<(String, String)> {
    let config = crate::config::get_global();

    // NOTE: OS や wry のバージョンによって渡される URL の形式が異なるため、以下の3パターンをすべて許容する
    // 1. http://protocol.tld/ (Windows の古いフォールバック)
    // 2. protocol://tld/ (Windows のカスタムプロトコル)
    // 3. protocol://host/ (macOS 等のカスタムプロトコル)
    let prefix_windows_http = format!("http://{}.{}/", config.internal_protocol, config.internal_tld);
    let prefix_windows_custom = format!("{}://{}/", config.internal_protocol, config.internal_tld);
    let prefix_macos = format!("{}://{}/", config.internal_protocol, config.internal_host);

    let path_part = if url.starts_with(&prefix_windows_http) {
        &url[prefix_windows_http.len()..]
    } else if url.starts_with(&prefix_windows_custom) {
        &url[prefix_windows_custom.len()..]
    } else if url.starts_with(&prefix_macos) {
        &url[prefix_macos.len()..]
    } else {
        return None;
    };

    // NOTE: 先頭のスラッシュを取り除いたプレフィックスを取得 (例: "/_pkg/" -> "_pkg/")
    let prefix = if config.package_prefix.starts_with('/') {
        &config.package_prefix[1..]
    } else {
        &config.package_prefix
    };

    // NOTE: パスが package_prefix から始まる場合は他パッケージへのアクセスとみなす
    // 例: "_pkg/Assets.pak/images/logo.png" -> package: "Assets.pak", path: "images/logo.png"
    if path_part.starts_with(prefix) {
        let rest = &path_part[prefix.len()..];
        let (package, path) = rest.split_once('/').unwrap_or((rest, ""));
        Some((package.to_string(), path.to_string()))
    } else {
        // NOTE: プレフィックスを持たない通常のアクセスは、ランタイムパッケージへのアクセスとして扱う
        let clean_path = if path_part.starts_with('/') {
            &path_part[1..]
        } else {
            path_part
        };
        Some((config.runtime_package.clone(), clean_path.to_string()))
    }
}

/// パッケージおよびパスを、カスタムプロトコルを使用した URL に変換します。
///
/// `to_package_and_path` の逆の処理を行いますが、生成する URL はプラットフォームに応じた正しいベース URL に統一します。
///
/// - Windows でランタイムパッケージ: `protocol://tld/index.html`
/// - Windows で別パッケージ: `protocol://tld/package_prefix/Assets.pak/img.png`
/// - macOS でランタイムパッケージ: `protocol://host/index.html`
/// - macOS で別パッケージ: `protocol://host/package_prefix/Assets.pak/img.png`
pub fn to_custom_protocol_path(package: &str, path: &str) -> String {
    let config = crate::config::get_global();

    // NOTE: Windows WebView2 では `protocol://tld`、macOS WebKit では `protocol://host` をベース URL に設定
    let base = if cfg!(target_os = "windows") {
        format!("{}://{}", config.internal_protocol, config.internal_tld)
    } else {
        format!("{}://{}", config.internal_protocol, config.internal_host)
    };

    let clean_path = if path.starts_with('/') { &path[1..] } else { path };
    if package == config.runtime_package {
        // NOTE: ランタイムパッケージへのアクセスは、ルート直下にマッピング
        format!("{}/{}", base, clean_path)
    } else {
        // NOTE: 他のパッケージへのアクセスは、プレフィックスを付与してルーティング用の URL を組み立てる
        let prefix = if config.package_prefix.starts_with('/') {
            &config.package_prefix[1..]
        } else {
            &config.package_prefix
        };
        format!("{}/{}{}/{}", base, prefix, package, clean_path)
    }
}

pub async fn respond(request: Request<Vec<u8>>, responder: RequestAsyncResponder) {
    let config = crate::config::get_global();
    log::debug!("Raw URI: {}", request.uri().to_string());
    match request.uri().path() {
        // TODO: ユーザーが指定できるようにする
        "/_/update_webview" => {
            match request.method() {
                &Method::POST => {
                    open_webview_manual(&config.webview_install_url, true);
                    process::exit(1);
                },
                _ => responder.respond_with(StatusCode::METHOD_NOT_ALLOWED),
            }
        },
        "/_/version" => {
            match request.method() {
                &Method::GET => responder.respond_with(config.version.clone()),
                _ => responder.respond_with(StatusCode::METHOD_NOT_ALLOWED),
            }
        },
        "/_/config" => {
            let config_path = PathBuf::from(&config.save_directory).join("carat.cfg");
            match request.method() {
                &Method::GET => responder.respond_with(fs::read(config_path)),
                &Method::POST => responder.respond_with(fs::write(config_path, request.body())),
                &Method::DELETE => responder.respond_with(fs::remove_file(config_path)),
                _ => responder.respond_with(StatusCode::METHOD_NOT_ALLOWED),
            }
        },
        #[cfg(debug_assertions)]
        _ => respond_in_develop(request, responder).await,
        #[cfg(not(debug_assertions))]
        _ => respond_in_release(request, responder).await,
    }
}

#[cfg(not(debug_assertions))]
async fn respond_in_release(request: Request<Vec<u8>>, responder: RequestAsyncResponder) {
    // TODO: 実装
    responder.respond_with(StatusCode::IM_A_TEAPOT);
}

#[cfg(debug_assertions)]
async fn respond_in_develop(request: Request<Vec<u8>>, responder: RequestAsyncResponder) {
    use std::{path::PathBuf, str::FromStr};

    use http::{HeaderName, HeaderValue};
    use responder::Response;
    let config = crate::config::get_global();

    let (package, path) = to_package_and_path(&request.uri().to_string()).unwrap_or(("".to_string(), "".to_string()));
    if !path.starts_with("@fs") && !path.starts_with("@vite") {
        log::debug!("Asset requested: {}/{}", package, path);
    }

    if package == config.runtime_package {
        // NOTE: ランタイムパッケージは Vite に転送する
        // TODO: シャルフェルトの仕様を引き継いでいるので直したい
        let mut body = Vec::new();
        if let Ok(response) = http_req::request::get(format!("{}/{}", config.dev_server, path), &mut body) {
            // NOTE: ビルダーを作成し、ステータスをコピーする
            let mut builder = http::Response::builder()
                .status::<u16>(response.status_code().into());

            // NOTE: ヘッダーをコピーする
            let builder_headers = builder.headers_mut().unwrap();
            for (key, value) in response.headers().iter() {
                builder_headers.append(HeaderName::from_str(key).unwrap(), HeaderValue::from_str(value).unwrap());
            }
            builder_headers.append("Content-Security-Policy", HeaderValue::from_str("default-src 'self' data: blob:; script-src 'self' 'wasm-unsafe-eval' 'unsafe-inline' blob:; style-src 'self' 'unsafe-inline'; connect-src 'self' blob: ipc: http://ipc.localhost ws://localhost:1421").unwrap());

            // NOTE: ボディをコピーする
            responder.respond(
                builder
                    .body(body)
                    .unwrap()
            );
        } else {
            responder.respond_with(StatusCode::BAD_GATEWAY);
        }
    } else if package == config.assets_package {
        let package_path = PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string());
        let target_path = package_path.parent().unwrap().join(&path);

        if request.headers().get("accept").is_some_and(|v| v.to_owned() == "inode/directory") {
            return responder.respond_with(fs::read_dir(target_path));
        } else {
            match fs::read(target_path) {
                Ok(body) => {
                    responder.respond_with(Response {
                        code: StatusCode::OK,
                        mime: guess_mime(&path).to_owned(),
                        body
                    });
                },
                Err(error) => responder.respond_with(error),
            };
        }
    } else {
        responder.respond_with(StatusCode::NOT_IMPLEMENTED);
        todo!("開発環境でカスタムパッケージを呼び出すシステムは実装されていません ({})", path);
    }
}
