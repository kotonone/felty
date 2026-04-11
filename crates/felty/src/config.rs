use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub id: String,
    pub name: String,
    pub version: String,
    pub webview_install_url: String,
    pub internal_protocol: String,
    pub internal_tld: String,
    pub internal_host: String,
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

static GLOBAL_CONFIG: OnceLock<GlobalConfig> = OnceLock::new();

pub fn get_global() -> &'static GlobalConfig {
    GLOBAL_CONFIG.get().expect("Global config is not initialized. Make sure to use load_config!() properly.")
}

pub fn init_global(config: &GlobalConfig) {
    let _ = GLOBAL_CONFIG.set(config.clone());
}
