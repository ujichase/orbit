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
use std::io;
use std::io::{Error, Read};

/// Conditionally operates on `status` to return an string representation.
pub fn report_eval(status: bool) -> ColoredString {
    match status {
        true => ColoredString::from("ok").green(),
        false => ColoredString::from("no").red(),
    }
}

/// Outputs the text `s` with a ? mark and y/n option. Accepts '\n' or
/// 'y' to return `true`, and `n` to return `false`.
pub fn prompt(s: &str, use_info: bool) -> Result<bool, Error> {
    if use_info == true {
        crate::info!("{}? [y/n]", s);
    } else {
        println!("{}? [y/n]", s);
    }
    check_for_response(&mut io::stdin().lock())
}

/// Infinitely loops until a valid response is entered. "Y\n" and "\n" map to `true`, while
/// "N\n" maps to `false`.
///
/// Also supports checking windows-style line endings `\r\n`.
fn check_for_response(input: &mut (impl Read + std::io::BufRead)) -> Result<bool, Error> {
    let mut buffer: String = String::new();
    loop {
        input.read_line(&mut buffer)?;
        let result = match buffer.to_uppercase().as_ref() {
            "\r\n" | "\n" | "Y\n" | "Y\r\n" => Some(true),
            "N\n" | "N\r\n" => Some(false),
            _ => {
                buffer.clear();
                None
            }
        };
        if let Some(r) = result {
            break Ok(r);
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_input_to_output() {
        let r = check_for_response(&mut "n\n".as_bytes()).unwrap();
        assert_eq!(r, false);
        let r = check_for_response(&mut "N\n".as_bytes()).unwrap();
        assert_eq!(r, false);
        let r = check_for_response(&mut "\n".as_bytes()).unwrap();
        assert_eq!(r, true);
        let r = check_for_response(&mut "Y\n".as_bytes()).unwrap();
        assert_eq!(r, true);
        let r = check_for_response(&mut "y\n".as_bytes()).unwrap();
        assert_eq!(r, true);
    }

    #[test]
    fn windows_style() {
        let r = check_for_response(&mut "y\r\n".as_bytes()).unwrap();
        assert_eq!(r, true);
        let r = check_for_response(&mut "\r\n".as_bytes()).unwrap();
        assert_eq!(r, true);
        let r = check_for_response(&mut "N\r\n".as_bytes()).unwrap();
        assert_eq!(r, false);
    }
}
