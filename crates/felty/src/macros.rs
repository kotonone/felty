#[macro_export]
macro_rules! load_config {
    () => {{
        let preset = include!(concat!(env!("OUT_DIR"), "/felty_generated_config.rs"));
        $crate::config::init_global(&preset.0);
        preset
    }};
}
