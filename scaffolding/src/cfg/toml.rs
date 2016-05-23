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
use std::borrow::Cow;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::atomic::Ordering::Relaxed;
use std::{error, fmt, io, result};

use notty::cfg::{SCROLLBACK, TAB_STOP};
use notty::datatypes::{CodeGroup, Color, ConfigStyle};
use notty::terminal::Styles;
use notty_cairo::{Config as CairoConfig, TrueColor, PALETTE};

use super::Config;

use toml::{self, Table, Value};

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

fn update_general(shell: &mut Cow<'static, str>, font: &mut Cow<'static, str>, table: &Table) {
    for (k, v) in table.iter() {
        match &k[..] {
            "shell" => {
                *shell = Cow::Owned(String::from(v.as_str().unwrap()))
            }
            "font" => {
                *font = Cow::Owned(String::from(v.as_str().unwrap()))
            }
            "tabstop" => TAB_STOP.store(v.as_integer().unwrap() as usize, Relaxed),
            "scrollback" => SCROLLBACK.store(v.as_integer().unwrap() as isize, Relaxed),
            _ => {},
        };
    }
}

fn update_colors(config: &mut CairoConfig, table: &Table) {
    for (k, v) in table.iter() {
        match &k[..] {
            "background"    => config.bg_color = convert_tomlv_to_color(v),
            "foreground"    => config.fg_color = convert_tomlv_to_color(v),
            "cursor"        => config.cursor_color = convert_tomlv_to_color(v),
            "palette"       => config.palette = convert_tomlv_to_palette(v),
            _               => {},
        };
    }
}

fn update_syntax(config: &mut CairoConfig, table: &Table) {
    for (k, v) in table.iter() {
        match &k[..] {
            "plain" => {
                let styles = convert_tomlv_to_styles(v.as_table().unwrap());
                config.styles.insert(ConfigStyle::Plain, styles);
            }
            "code"  => update_syntax_code(config, v.as_table().unwrap()),
            _       => {}
        }
    }
}

fn update_syntax_code(config: &mut CairoConfig, table: &Table) {
    for (k, v) in table.iter() {
        let code_group = match &k[..] {
            "comment"       => CodeGroup::Comment,
            "documentation" => CodeGroup::Documentation,
            "error"         => CodeGroup::Error,
            "identifier"    => CodeGroup::Identifier,
            "keyword"       => CodeGroup::Keyword,
            "literal"       => CodeGroup::Literal,
            "macro"         => CodeGroup::Macro,
            "special"       => CodeGroup::Special,
            "todo"          => CodeGroup::Todo,
            "type"          => CodeGroup::Type,
            _               => continue
        };
        let styles = convert_tomlv_to_styles(v.as_table().unwrap());
        config.styles.insert(ConfigStyle::CodeGroup(code_group), styles);
    }
}

/// Update &config from toml file identified by path string.
pub fn update_from_file<P: AsRef<Path>>(cfg: &mut Config, path: P) -> Result<()> {
    let table = try!(read_toml_file(path));

    for (k, v) in table.iter() {
        match &k[..] {
            "color"     => update_colors(&mut cfg.cairo, v.as_table().unwrap()),
            "general"   => update_general(&mut cfg.shell, &mut cfg.cairo.font, v.as_table().unwrap()),
            "syntax"    => update_syntax(&mut cfg.cairo, v.as_table().unwrap()),
            _ => {},
        };
    }
    Ok(())
}

fn convert_tomlv_to_color(value: &Value) -> TrueColor {
    match *value {
        Value::String(ref string)   => (
            u8::from_str_radix(&string[0..2], 16).unwrap(),
            u8::from_str_radix(&string[2..4], 16).unwrap(),
            u8::from_str_radix(&string[4..6], 16).unwrap(),
        ),
        Value::Array(ref array)     => (
            array[0].as_integer().unwrap() as u8,
            array[1].as_integer().unwrap() as u8,
            array[2].as_integer().unwrap() as u8
        ),
        _                           => panic!()
    }
}

fn convert_tomlv_to_palette(value: &Value) -> [TrueColor; 256] {
    let mut palette = PALETTE;
    for (key, value) in value.as_table().unwrap() {
        let n: usize = key.trim_left_matches("color").parse().unwrap();
        palette[n.saturating_sub(1)] = convert_tomlv_to_color(value);
    }
    palette
}

fn convert_tomlv_to_styles(table: &Table) -> Styles {
    let fg_color = table.get("foreground").map_or(Color::Default, |v| {
        let (r, g, b) = convert_tomlv_to_color(v);
        Color::True(r, g, b)
    });
    let bg_color = table.get("background").map_or(Color::Default, |v| {
        let (r, g, b) = convert_tomlv_to_color(v);
        Color::True(r, g, b)
    });
    let opacity = table.get("opacity").map_or(0xff, |v| v.as_integer().unwrap() as u8);
    let (underline, double_underline) = match table.get("underline") {
        Some(&Value::Boolean(true)) | Some(&Value::Integer(1))  => (true, false),
        Some(&Value::Integer(2))                                => (true, true),
        _                                                       => (false, false),
    };
    let bold = table.get("bold").map_or(false, |v| v.as_bool().unwrap());
    let italic = table.get("italic").map_or(false, |v| v.as_bool().unwrap());
    let strikethrough = table.get("strikethrough").map_or(false, |v| v.as_bool().unwrap());
    let inverted = table.get("inverted").map_or(false, |v| v.as_bool().unwrap());
    let blink = table.get("blink").map_or(false, |v| v.as_bool().unwrap());
    Styles {
        fg_color: fg_color,
        bg_color: bg_color,
        opacity: opacity,
        underline: underline,
        double_underline: double_underline,
        bold: bold,
        italic: italic,
        strikethrough: strikethrough,
        inverted: inverted,
        blink: blink,
    }
}

fn read_toml_file<P: AsRef<Path>>(path: P) -> Result<Table> {
    let mut file = try!(File::open(&path));
    let mut source = String::new();
    try!(file.read_to_string(&mut source));
    parse_toml(&source.to_string(), path)
}

fn parse_toml<P: AsRef<Path>>(toml_string: &String, toml_path: P)
                              -> Result<Table> {
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

    use super::*;

    use super::super::Config;
    use notty_cairo::Config as CairoConfig;

    fn test_default_config(config: &CairoConfig) {
        assert_eq!(config.font, "Inconsolata 10");
        assert_eq!(config.fg_color, (0xff,0xff,0xff));
        assert_eq!(config.bg_color, (0x00,0x00,0x00));
        assert_eq!(config.cursor_color, (0xbb,0xbb,0xbb));
        assert_eq!(config.palette[0], (0x00,0x00,0x00));
        assert_eq!(config.palette[5], (0xff,0x55,0xff));
    }

    #[test]
    fn test_default() {
        let config = CairoConfig::default();
        test_default_config(&config);
    }

    #[test]
    fn test_update_from_file() {
        let mut config = Config::default();
        test_default_config(&config.cairo);

        let update_path = "resources/update-config.toml".to_string();
        update_from_file(&mut config, &update_path).unwrap();
        assert_eq!(config.cairo.font, "Liberation Mono 8");
    }
}
