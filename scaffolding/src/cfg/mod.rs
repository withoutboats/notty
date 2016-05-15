use std::env;
use std::path::PathBuf;

use notty_cairo::ColorConfig;

mod toml;

const CONFIG_FILE: &'static str = "scaffolding.toml";

pub struct Config {
    pub color_cfg: ColorConfig,
    pub font: String,
}

impl Config {
    pub fn new() -> Config {
        let mut config = Config::default();
        let user_config_path = match env::var("XDG_CONFIG_HOME") {
            Ok(dir) => PathBuf::from(dir).join(CONFIG_FILE),
            Err(_)  => env::home_dir().unwrap().join(".config").join(CONFIG_FILE),
        };
        let _ = toml::update_from_file(&mut config, user_config_path);
        config
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            color_cfg: ColorConfig::default(),
            font: String::from("Inconsolata 10"),
        }
    }
}
