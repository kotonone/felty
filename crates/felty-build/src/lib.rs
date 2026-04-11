use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use image::ImageReader;
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct FeltyToml {
    #[serde(default)]
    pub app: AppConfigSection,
    #[serde(default)]
    pub window: WindowConfigSection,
    #[serde(default)]
    pub webview: WebviewConfigSection,
    #[serde(default)]
    pub internal: InternalConfigSection,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct AppConfigSection {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub copyright: Option<String>,
    pub internal_name: Option<String>,
    pub comments: Option<String>,
    pub icon_path: Option<String>,
}

impl Default for AppConfigSection {
    fn default() -> Self {
        Self {
            id: "com.example.app".into(),
            name: "Felty App".into(),
            version: "0.1.0".into(),
            author: None,
            copyright: None,
            internal_name: None,
            comments: None,
            icon_path: None,
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct WindowConfigSection {
    pub width: f64,
    pub height: f64,
    pub resizable: bool,
    pub maximizable: bool,
}

impl Default for WindowConfigSection {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 800.0,
            resizable: true,
            maximizable: true,
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct WebviewConfigSection {
    pub install_url: String,
}

impl Default for WebviewConfigSection {
    fn default() -> Self {
        Self { install_url: "https://example.com/help/install-webview".into() }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct InternalConfigSection {
    pub protocol: String,
    pub tld: String,
    pub host: String,
    pub dev_server: String,
    pub package_prefix: String,
    pub runtime_package: String,
    pub assets_package: String,
    pub save_directory: String,
    pub assets_directory: String,
    pub log_directory: String,
    pub cache_directory: String,
    pub website_url: String,
    pub release_note_url: String,
    pub report_url: String,
}

impl Default for InternalConfigSection {
    fn default() -> Self {
        Self {
            protocol: "felty".into(),
            tld: "tld".into(),
            host: "host".into(),
            dev_server: "http://localhost:5173".into(),
            package_prefix: "/@packages/".into(),
            runtime_package: "Runtime.pak".into(),
            assets_package: "Assets.pak".into(),
            save_directory: "Save".into(),
            assets_directory: "Data/Assets".into(),
            log_directory: "Data/Logs".into(),
            cache_directory: "Data/Caches".into(),
            website_url: "https://example.com".into(),
            release_note_url: "https://example.com/releases".into(),
            report_url: "https://example.com/report".into(),
        }
    }
}

pub fn build<P: AsRef<Path>>(toml_path: P) {
    let toml_content = fs::read_to_string(toml_path.as_ref()).expect("Failed to read felty.toml");
    let config: FeltyToml = toml::from_str(&toml_content).expect("Failed to parse felty.toml");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // 1. Process Icon and Metadata
    let mut icon_code = "None".to_string();

    if let Some(icon_path) = &config.app.icon_path {
        let icon_img = ImageReader::open(icon_path)
            .expect("Could not read icon file")
            .decode()
            .expect("Could not decode icon");
        let resized = icon_img.resize(32, 32, image::imageops::FilterType::Lanczos3);

        let icon_bin_path = out_path.join("icon.bin");
        File::create(&icon_bin_path)
            .expect("Could not create temp file")
            .write_all(&resized.to_rgba8().into_raw())
            .expect("Could not write icon");

        let w = resized.width();
        let h = resized.height();

        icon_code = format!(
            r#"Some(::felty::tao::window::Icon::from_rgba(include_bytes!(concat!(env!("OUT_DIR"), "/icon.bin")).to_vec(), {}, {}).unwrap())"#,
            w, h
        );

        set_metadata(&config.app, icon_path, out_path);
    } else {
        set_metadata(&config.app, "", out_path);
    }

    // 2. Generate config rust code
    let generated_code = format!(
        r#"
::felty::config::AppConfig {{
    id: "{id}".to_string(),
    name: "{name}".to_string(),
    version: "{version}".to_string(),
    window_size: ::felty::tao::dpi::LogicalSize::new({width}f64, {height}f64),
    resizable: {resizable},
    maximizable: {maximizable},
    webview_install_url: "{webview_url}".to_string(),
    icon: {icon_code},
    menu: None,
    internal_protocol: "{protocol}".to_string(),
    internal_tld: "{tld}".to_string(),
    internal_host: "{host}".to_string(),
    dev_server: "{dev_server}".to_string(),
    package_prefix: "{package_prefix}".to_string(),
    runtime_package: "{runtime_package}".to_string(),
    assets_package: "{assets_package}".to_string(),
    save_directory: "{save_directory}".to_string(),
    assets_directory: "{assets_directory}".to_string(),
    log_directory: "{log_directory}".to_string(),
    cache_directory: "{cache_directory}".to_string(),
    website_url: "{website_url}".to_string(),
    release_note_url: "{release_note_url}".to_string(),
    report_url: "{report_url}".to_string(),
}}
        "#,
        id = config.app.id,
        name = config.app.name,
        version = config.app.version,
        width = config.window.width,
        height = config.window.height,
        resizable = config.window.resizable,
        maximizable = config.window.maximizable,
        webview_url = config.webview.install_url,
        icon_code = icon_code,
        protocol = config.internal.protocol,
        tld = config.internal.tld,
        host = config.internal.host,
        dev_server = config.internal.dev_server,
        package_prefix = config.internal.package_prefix,
        runtime_package = config.internal.runtime_package,
        assets_package = config.internal.assets_package,
        save_directory = config.internal.save_directory,
        assets_directory = config.internal.assets_directory,
        log_directory = config.internal.log_directory,
        cache_directory = config.internal.cache_directory,
        website_url = config.internal.website_url,
        release_note_url = config.internal.release_note_url,
        report_url = config.internal.report_url,
    );

    let config_rs_path = out_path.join("felty_generated_config.rs");
    fs::write(&config_rs_path, generated_code).expect("Failed to write config.rs");
}

fn set_metadata(app: &AppConfigSection, icon_path_str: &str, out_path: &Path) {
    #[cfg(target_os = "windows")] {
        use ico_builder::IcoBuilder;
        use semver::Version;
        use winresource::{VersionInfo, WindowsResource};

        let mut resource = WindowsResource::new();
        if let Ok(v) = Version::parse(&app.version) {
            let version = v.major << 48 | v.minor << 32 | v.patch << 16;
            resource.set_version_info(VersionInfo::FILEVERSION, version);
            resource.set_version_info(VersionInfo::PRODUCTVERSION, version);
        }
        resource.set("ProductName", &app.name);
        resource.set("FileDescription", &app.name);
        resource.set("ProductVersion", &app.version);
        resource.set("FileVersion", &app.version);
        if let Some(author) = &app.author {
            resource.set("CompanyName", author);
        }
        if let Some(copyright) = &app.copyright {
            resource.set("LegalCopyright", copyright);
        }
        if let Some(internal_name) = &app.internal_name {
            resource.set("InternalName", internal_name);
        }
        if let Some(comments) = &app.comments {
            resource.set("Comments", comments);
        }

        if !icon_path_str.is_empty() {
            let ico_path = out_path.join("icon.ico");
            IcoBuilder::default()
                .add_source_file(icon_path_str)
                .build_file(ico_path.to_str().unwrap())
                .unwrap();
            resource.set_icon(ico_path.to_str().unwrap());
        }

        resource.set_language(0x0411); // LANG_JAPANESE
        resource.compile().unwrap();
    }
}
