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

use datatypes::{Color,Palette};

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

fn parse_toml(toml_string: &String, toml_path: &String) -> Option<toml::Table> {
    let mut parser = toml::Parser::new(toml_string);
    match parser.parse() {
        Some(toml) => {
            Some(toml)
        }
        None => {
            for err in &parser.errors {
                let (loline, locol) = parser.to_linecol(err.lo);
                let (hiline, hicol) = parser.to_linecol(err.hi);
                panic!("{}:{}:{}:{}:{} error: {}",
                       toml_path, loline, locol, hiline, hicol, err.desc);
            }
            None
        }
    }
}

fn convert_color(value: &toml::Value) -> Color {
    let slice = value.as_slice().unwrap();
    Color(slice[0].as_integer().unwrap() as u8,
          slice[1].as_integer().unwrap() as u8,
          slice[2].as_integer().unwrap() as u8)
}

impl Config {
    pub fn new_from_file(path: &String) -> Config {
        let source = &mut String::new();
        File::open(path).and_then(|mut f| {
            f.read_to_string(source)
        }).unwrap();

        parse_toml(&source.to_string(), path).map(|table| {
            Config::new_from_toml(&toml::Value::Table(table))
        }).unwrap()
    }

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
                map(convert_color).
                unwrap(),
            bg_color: toml.lookup("colors.bg").
                map(convert_color).
                unwrap(),
            cursor_color: toml.lookup("colors.cursor").
                map(convert_color).
                unwrap(),
            colors: toml.lookup("colors.palette").
                map(|slice| {
                    let v: Vec<Color> = slice.
                        as_slice().
                        unwrap().
                        into_iter().
                        map(convert_color).
                        collect();
                    Palette::new_from_slice(&v).unwrap()
                }).
                unwrap(),
        }
    }
}
