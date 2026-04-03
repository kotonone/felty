use tao::dpi::LogicalSize;
use muda::Menu;
use tao::window::Icon;
use std::sync::OnceLock;

pub struct AppConfig {
    pub id: String,
    pub name: String,
    pub version: String,
    pub window_size: LogicalSize<f64>,
    pub resizable: bool,
    pub maximizable: bool,
    pub webview_install_url: String,
    pub icon: Option<Icon>,
    pub menu: Option<Menu>,
    pub internal_protocol: String,
    pub internal_tld: String,
    pub internal_host: String,
    pub internal_dev_port: u16,
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

#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub id: String,
    pub name: String,
    pub version: String,
    pub webview_install_url: String,
    pub internal_protocol: String,
    pub internal_tld: String,
    pub internal_host: String,
    pub internal_dev_port: u16,
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

static GLOBAL_CONFIG: OnceLock<GlobalConfig> = OnceLock::new();

pub fn get_global() -> &'static GlobalConfig {
    GLOBAL_CONFIG.get().expect("Global config is not initialized. Make sure to use load_config!() properly.")
}

pub fn init_global(config: &AppConfig) {
    let _ = GLOBAL_CONFIG.set(GlobalConfig {
        id: config.id.clone(),
        name: config.name.clone(),
        version: config.version.clone(),
        webview_install_url: config.webview_install_url.clone(),
        internal_protocol: config.internal_protocol.clone(),
        internal_tld: config.internal_tld.clone(),
        internal_host: config.internal_host.clone(),
        internal_dev_port: config.internal_dev_port,
        package_prefix: config.package_prefix.clone(),
        runtime_package: config.runtime_package.clone(),
        assets_package: config.assets_package.clone(),
        save_directory: config.save_directory.clone(),
        assets_directory: config.assets_directory.clone(),
        log_directory: config.log_directory.clone(),
        cache_directory: config.cache_directory.clone(),
        website_url: config.website_url.clone(),
        release_note_url: config.release_note_url.clone(),
        report_url: config.report_url.clone(),
    });
}


