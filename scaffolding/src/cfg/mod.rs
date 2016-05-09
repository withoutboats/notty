use std::env;

use notty::cfg::Config as NottyConfig;
use notty_cairo::ColorConfig;

mod toml;

pub struct Config {
    pub notty_cfg: NottyConfig,
    pub color_cfg: ColorConfig,
    pub font: String,
}

impl Config {
    pub fn new() -> Config {
        let mut config = Config::default();
        let user_config_path = env::home_dir().unwrap().join(".scaffolding.rc");
        let _ = toml::update_from_file(&mut config, user_config_path);
        config
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            notty_cfg: NottyConfig::default(),
            color_cfg: ColorConfig::default(),
            font: String::from("Inconsolata 10"),
        }
    }
}
