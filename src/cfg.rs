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

use std::fmt::{Debug,Formatter,Result};
use std::fs::File;
use std::io::prelude::*;

use datatypes::{Color,Palette};

#[derive(Clone, Eq, PartialEq)]
pub struct Config {
    pub font: String,
    pub scrollback: u32,
    pub tab_stop: u32,
    pub fg_color: Color,
    pub bg_color: Color,
    pub cursor_color: Color,
    pub colors: Palette,
}

fn empty_config() -> Config {
    Config {
        font: "Inconsolata 10".to_string(),
        scrollback: 512,
        tab_stop: 4,
        fg_color: Color(0x00,0x00,0x00),
        bg_color: Color(0xbb,0xbb,0xbb),
        cursor_color: Color(0xbb,0xbb,0xbb),
        colors: Palette::new_from_slice(&[Color(0xbb,0xbb,0xbb);256]).unwrap(),
    }
}

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Config {{ }}")
    }
}

impl Default for Config {
    fn default() -> Config {
        let source = include_str!("../resources/default-config.toml");
        parse_toml(&source.to_string(),
                   &"../resources/default-config.toml".to_string()).unwrap()
    }
}

fn parse_toml(toml_string: &String, toml_path: &String) -> Option<Config> {
    let mut parser = toml::Parser::new(toml_string);
    match parser.parse() {
        Some(toml) => {
            let mut config = empty_config();
            config.update_from_toml(&toml);
            Some(config.clone())
        }
        None => {
            for err in &parser.errors {
                let (loline, locol) = parser.to_linecol(err.lo);
                let (hiline, hicol) = parser.to_linecol(err.hi);
                println!("{}:{}:{}:{}:{} error: {}",
                         toml_path, loline, locol, hiline, hicol, err.desc);
            }
            None
        }
    }
}

pub fn config_loader(path: &String) -> Option<Config> {
    let source = &mut String::new();
    File::open(path).and_then(|mut f| {
        f.read_to_string(source)
    }).unwrap();
    parse_toml(&source.to_string(), path)
}

fn convert_color(value: &toml::Value) -> Color {
    let slice = value.as_slice().unwrap();
    Color(slice[0].as_integer().unwrap() as u8,
          slice[1].as_integer().unwrap() as u8,
          slice[2].as_integer().unwrap() as u8)
}

impl Config {
    fn update_colors(&mut self, table: &toml::Table) {
        for (k, v) in table {
            match &k[..] {
                "fg" => self.fg_color = convert_color(v),
                "bg" => self.bg_color = convert_color(v),
                "cursor" => self.cursor_color = convert_color(v),
                "palette" => {
                    let slice = v.as_slice().unwrap();
                    let thing1 = slice.into_iter().map(convert_color);
                    let thing2: Vec<_> = thing1.collect();
                    self.colors = Palette::new_from_slice(&thing2).unwrap();
                }
                _ => {},
            }
        }
    }

    fn update_general(&mut self, table: &toml::Table) {
        for (k, v) in table {
            match &k[..] {
                "font" => self.font = v.as_str().unwrap().to_string(),
                "scrollback" => self.scrollback = v.as_integer().unwrap() as u32,
                "tabstop" => self.tab_stop = v.as_integer().unwrap() as u32,
                _ => {},
            }
        }
    }

    fn update_from_toml(&mut self, table: &toml::Table) {
        for (k, v) in table {
            let value = v.clone();
            match (&k[..], value) {
                ("general", toml::Value::Table(t)) =>
                    self.update_general(&t),
                ("colors", toml::Value::Table(t)) =>
                    self.update_colors(&t),
                (_, _) => {},
            }
        }
    }
}
