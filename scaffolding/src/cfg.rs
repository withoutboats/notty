//  notty is a new kind of terminal emulator.
//  Copyright (C) 2016 Wayne Warren
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Affero General Public License for more details.
//
//  You should have received a copy of the GNU Affero General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::{error,fmt,io,result};

use notty::datatypes::{Color,Palette};
use notty::cfg::Config;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Parse(String), // TODO: once https://github.com/alexcrichton/toml-rs/issue#69
                   // is closed, change this to Parse(toml::ParserError)
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Io(ref err) => write!(f, "IO Error: {}", err),
            ConfigError::Parse(ref string) => write!(f, "{}", string),
        }
    }
}

impl error::Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::Io(ref err) => err.description(),
            ConfigError::Parse(ref string) => &string,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ConfigError::Io(ref err) => err.cause(),
            ConfigError::Parse(_) => None,
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError{
        ConfigError::Io(err)
    }
}

pub type Result<T> = result::Result<T, ConfigError>;

fn update_general(config: &mut Config, table: &toml::Table) {
    for (k, v) in table.iter() {
        match &k[..] {
            "font" => config.font = v.as_str().
                map(|s| s.to_string()).
                unwrap(),
            "tabstop" => config.tab_stop = v.as_integer().
                unwrap() as u32,
            "scrollback" => config.scrollback = v.as_integer().
                unwrap() as u32,
            _ => {},
        };
    }
}

fn update_colors(config: &mut Config, table: &toml::Table) {
    for (k, v) in table.iter() {
        match &k[..] {
            "fg" => config.bg_color = convert_tomlv_to_color(v),
            "bg" => config.fg_color = convert_tomlv_to_color(v),
            "cursor" => config.cursor_color = convert_tomlv_to_color(v),
            "palette" => config.colors = convert_tomlv_to_palette(v),
            _ => {},
        };
    }
}

/// Update &config from toml file identified by path string.
pub fn update_from_file<P: AsRef<Path>>(config: &mut Config, path: P) -> Result<()> {
    let table = try!(read_toml_file(path));

    for (k, v) in table.iter() {
        match &k[..] {
            "colors" => update_colors(config, v.as_table().unwrap()),
            "general" => update_general(config, v.as_table().unwrap()),
            _ => {},
        };
    }
    Ok(())
}

fn convert_tomlv_to_color(value: &toml::Value) -> Color {
    let slice = value.as_slice().unwrap();
    Color(slice[0].as_integer().unwrap() as u8,
          slice[1].as_integer().unwrap() as u8,
          slice[2].as_integer().unwrap() as u8)
}

fn convert_tomlv_to_palette(value: &toml::Value) -> Palette {
    let v: Vec<Color> = value.
        as_slice().
        unwrap().
        into_iter().
        map(convert_tomlv_to_color).
        collect();
    Palette::new_from_slice(&v).unwrap()
}

fn read_toml_file<P: AsRef<Path>>(path: P) -> Result<toml::Table> {
    let mut file = try!(File::open(&path));
    let mut source = String::new();
    try!(file.read_to_string(&mut source));
    parse_toml(&source.to_string(), path)
}

fn parse_toml<P: AsRef<Path>>(toml_string: &String, toml_path: P)
                              -> Result<toml::Table> {
    let mut parser = toml::Parser::new(toml_string);
    match parser.parse() {
        Some(toml) => {
            Ok(toml)
        }
        None => {
            let mut error_string = String::new();
            for err in &parser.errors {
                let (loline, locol) = parser.to_linecol(err.lo);
                let (hiline, hicol) = parser.to_linecol(err.hi);
                error_string = format!("{}\n{}:{}:{}:{}:{} error: {}",
                        error_string, toml_path.as_ref().display(), loline,
                        locol, hiline, hicol, err.desc);
            }
            Err(ConfigError::Parse(error_string))
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate toml;

    use super::*;

    use notty::cfg::Config;
    use notty::datatypes::Color;

    fn test_default_config(config: &Config) {
        assert_eq!(config.font, "Inconsolata 10");
        assert_eq!(config.scrollback, 512);
        assert_eq!(config.tab_stop, 4);
        assert_eq!(config.fg_color, Color(0xff,0xff,0xff));
        assert_eq!(config.bg_color, Color(0x00,0x00,0x00));
        assert_eq!(config.cursor_color, Color(0xbb,0xbb,0xbb));
        assert_eq!(config.colors[0], Color(0x00,0x00,0x00));
        assert_eq!(config.colors[5], Color(0xff,0x55,0xff));
    }

    #[test]
    fn test_default() {
        let config = Config::default();
        test_default_config(&config);
    }

    #[test]
    fn test_new_from_file() {
        let path = "resources/default-config.toml".to_string();
        let config = new_from_file(&path).unwrap();

        test_default_config(&config);
    }

    #[test]
    fn test_new_from_toml() {
        let toml_string = include_str!("../resources/default-config.toml");
        let mut parser = toml::Parser::new(toml_string);
        let config = new_from_toml(&toml::Value::Table(parser
                                                       .parse()
                                                       .unwrap()));
        test_default_config(&config);
    }

    #[test]
    fn test_update_from_file() {
        let mut config = &mut Config::default();
        test_default_config(&config);

        let update_path = "resources/update-config.toml".to_string();
        update_from_file(config, &update_path).unwrap();
        assert_eq!(config.font, "Liberation Mono 8");
    }
}
