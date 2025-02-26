//
//  Copyright (C) 2022-2025  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use colored::ColoredString;
use colored::Colorize;
use palette::*;
use std::fmt::Display;

pub trait ToColor: Display {
    fn to_color(&self) -> ColoredString;
}

#[derive(Debug, PartialEq)]
pub enum ColorTone {
    Color(ColoredString),
    Bland(String),
}

impl Display for ColorTone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Color(c) => write!(f, "{}", c),
            Self::Bland(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ColorVec(Vec<ColorTone>);

impl Display for ColorVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.0 {
            write!(f, "{}", item)?
        }
        Ok(())
    }
}

impl ColorVec {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push_str(&mut self, s: &str) -> () {
        self.0.push(ColorTone::Bland(String::from(s)));
    }

    pub fn push_color(&mut self, c: ColoredString) -> () {
        self.0.push(ColorTone::Color(c));
    }

    pub fn push(&mut self, ct: ColorTone) -> () {
        self.0.push(ct);
    }

    pub fn append(&mut self, mut cv: ColorVec) -> () {
        self.0.append(&mut cv.0);
    }

    pub fn push_whitespace(&mut self, count: usize) -> () {
        self.0
            .push(ColorTone::Bland(format!("{:<width$}", " ", width = count)));
    }

    pub fn swap(mut self, index: usize, hue: Rgb) -> Self {
        let item = self.0.get_mut(index).unwrap();
        *item = ColorTone::Color(color(&item.to_string(), hue));
        self
    }

    pub fn into_all_bland(self) -> String {
        self.0
            .into_iter()
            .map(|f| match f {
                ColorTone::Bland(s) => s,
                ColorTone::Color(s) => String::from_utf8_lossy(s.as_bytes()).to_string(),
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn pop(&mut self) -> Option<ColorTone> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        let mut size = 0;
        self.0.iter().for_each(|f| match f {
            ColorTone::Bland(s) => size += s.len(),
            ColorTone::Color(s) => size += String::from_utf8_lossy(s.as_bytes()).to_string().len(),
        });
        size
    }
}

pub type Rgb = (u8, u8, u8);

pub fn color(s: &str, hue: (u8, u8, u8)) -> ColoredString {
    s.truecolor(hue.0, hue.1, hue.2)
}

mod palette {
    use super::*;

    // reds
    pub const BURNT_ORANGE: Rgb = (204, 85, 0);
    pub const GOLDEN_ROD: Rgb = (0xDA, 0xA5, 0x20);
    pub const TOMATO: Rgb = (0xFF, 0x63, 0x47);
    pub const ORANGE: Rgb = (0xFF, 0xA5, 0x00);
    pub const RED: Rgb = (0xFF, 0x00, 0x00);
    pub const FIREBRICK: Rgb = (0xB2, 0x22, 0x22);

    //blues
    pub const DARK_CYAN: Rgb = (0x00, 0x8B, 0x8B);
    pub const LT_SKY_BLUE: Rgb = (135, 206, 250);
    pub const MED_BLUE: Rgb = (0x00, 0x00, 0xCD);
    pub const TURQUOISE: Rgb = (0x40, 0xE0, 0xD0);

    // greens
    pub const MED_SPRING_GREEN: Rgb = (0, 250, 154);
    pub const SEAFOAM_GREEN: Rgb = (159, 226, 191);
    pub const GREEN: Rgb = (0x00, 0x80, 0x00);
    pub const LIME_GREEN: Rgb = (0x32, 0xCD, 0x32);
    pub const PALM_LEAF: Rgb = (0x64, 0x97, 0x50);
    pub const NATURE_GREEN: Rgb = (0x4F, 0xAD, 0x27);
}

/* standard colorings */
pub const NUMBERS: Rgb = GOLDEN_ROD;
pub const CHARS: Rgb = SEAFOAM_GREEN;
pub const STRINGS: Rgb = BURNT_ORANGE;

/* `orbit get` colorings */
pub const SIGNAL_DEC_IDENTIFIER: Rgb = LT_SKY_BLUE;
pub const INSTANCE_LHS_IDENTIFIER: Rgb = LT_SKY_BLUE;
pub const DATA_TYPE: Rgb = NATURE_GREEN;
pub const ENTITY_NAME: Rgb = NATURE_GREEN;
