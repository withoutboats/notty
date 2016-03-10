//  notty is a new kind of terminal emulator.
//  Copyright (C) 2015 without boats
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

use datatypes::{Color,Palette};

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
    pub font: String,
    pub scrollback: u32,
    pub tab_stop: u32,
    pub fg_color: Color,
    pub bg_color: Color,
    pub cursor_color: Color,
    pub colors: Palette,
}

impl Default for Config {
    fn default() -> Config {
        let source = include_str!("../resources/default-config.toml");
        let table = parse_toml(&source.to_string(),
                               &"../resources/default-config.toml".to_string()).unwrap();
        Config::new_from_toml(&toml::Value::Table(table))
    }
}

impl Config {
    /// Constructs a new Config from a toml file specifed by the input string "path".
    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let table = try!(read_toml_file(path));
        Ok(Config::new_from_toml(&toml::Value::Table(table)))
    }

    /// Constructs a new Config from a toml::Value object.
    pub fn new_from_toml(toml: &toml::Value) -> Config {
        Config {
            font: toml.lookup("general.font").
                and_then(|v| v.as_str()).
                map(|s| s.to_string()).
                unwrap(),
            tab_stop: toml.lookup("general.tabstop").
                and_then(|v| v.as_integer()).
                unwrap() as u32,
            scrollback: toml.lookup("general.scrollback").
                and_then(|v| v.as_integer()).
                unwrap() as u32,
            fg_color: toml.lookup("colors.fg").
                map(convert_tomlv_to_color).
                unwrap(),
            bg_color: toml.lookup("colors.bg").
                map(convert_tomlv_to_color).
                unwrap(),
            cursor_color: toml.lookup("colors.cursor").
                map(convert_tomlv_to_color).
                unwrap(),
            colors: toml.lookup("colors.palette").
                map(convert_tomlv_to_palette).
                unwrap(),
        }
    }

    fn update_general(&mut self, table: &toml::Table) {
        for (k, v) in table.iter() {
            match &k[..] {
                "font" => self.font = v.as_str().
                    map(|s| s.to_string()).
                    unwrap(),
                "tabstop" => self.tab_stop = v.as_integer().
                    unwrap() as u32,
                "scrollback" => self.scrollback = v.as_integer().
                    unwrap() as u32,
                _ => {},
            };
        }
    }

    fn update_colors(&mut self, table: &toml::Table) {
        for (k, v) in table.iter() {
            match &k[..] {
                "fg" => self.bg_color = convert_tomlv_to_color(v),
                "bg" => self.fg_color = convert_tomlv_to_color(v),
                "cursor" => self.cursor_color = convert_tomlv_to_color(v),
                "palette" => self.colors = convert_tomlv_to_palette(v),
                _ => {},
            };
        }
    }

    /// Update &self from toml file identified by path string.
    pub fn update_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let table = try!(read_toml_file(path));

        for (k, v) in table.iter() {
            match &k[..] {
                "colors" => self.update_colors(v.as_table().unwrap()),
                "general" => self.update_general(v.as_table().unwrap()),
                _ => {},
            };
        }
        Ok(())
    }
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
    use datatypes::Color;

    fn test_default_config(config: &Config) {
        assert_eq!(config.font, "Inconsolata 10");
        assert_eq!(config.scrollback, 512);
        assert_eq!(config.tab_stop, 4);
        assert_eq!(config.fg_color, Color(255,255,255));
        assert_eq!(config.bg_color, Color(0,0,0));
        assert_eq!(config.cursor_color, Color(187,187,187));
        assert_eq!(config.colors[0], Color(0,0,0));
        assert_eq!(config.colors[5], Color(255,85,255));
    }

    #[test]
    fn test_default() {
        let config = Config::default();
        test_default_config(&config);
    }

    #[test]
    fn test_new_from_file() {
        let path = "resources/default-config.toml".to_string();
        let config = Config::new_from_file(&path).unwrap();

        test_default_config(&config);
    }

    #[test]
    fn test_new_from_toml() {
        let toml_string = include_str!("../resources/default-config.toml");
        let mut parser = toml::Parser::new(toml_string);
        let config = Config::new_from_toml(&toml::Value::Table(parser
                                                               .parse()
                                                               .unwrap()));
        test_default_config(&config);
    }

    #[test]
    fn test_update_from_file() {
        let mut config = Config::default();
        test_default_config(&config);

        let update_path = "resources/update-config.toml".to_string();
        config.update_from_file(&update_path).unwrap();
        assert_eq!(config.font, "Liberation Mono 8");
    }
}
