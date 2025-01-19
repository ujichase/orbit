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

use crate::util::{anyerror::Fault, environment::Environment};
use std::collections::HashMap;

use super::ip::Ip;

pub struct StrSwapTable(HashMap<String, String>);

impl StrSwapTable {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn load_environment(mut self, env: &Environment) -> Result<Self, Fault> {
        for entry in env.iter() {
            let (key, value) = entry.to_variable();
            self.0.insert(key, value);
        }
        Ok(self)
    }

    pub fn load_ip(mut self, ip: &Ip) -> Result<Self, Fault> {
        self.0.insert(
            String::from("orbit.ip.name"),
            ip.get_man().get_ip().get_name().to_string(),
        );
        self.0.insert(
            String::from("orbit.ip.version"),
            ip.get_man().get_ip().get_version().to_string(),
        );
        // self.0.insert(String::from("orbit.ip.checksum"), )
        Ok(self)
    }

    pub fn add(&mut self, key: &str, value: &str) -> Option<String> {
        self.0.insert(key.to_string(), value.to_string())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}

const L_VAR_DELIMITER: char = '{';
const R_VAR_DELIMITER: char = '}';

/// Performs variable replacement on the given `text`, looking up variables in
/// the `code` to swap with their values.
pub fn substitute(text: String, code: &StrSwapTable) -> String {
    let mut result = String::new();

    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        // check if there is a valid variable replacement
        match c {
            L_VAR_DELIMITER => {
                match gather_variable(&mut chars, c, R_VAR_DELIMITER) {
                    Ok(r) => {
                        // remove delimiters and surrounding whitespace to get key name
                        let key = &r[2..r.len() - 2].trim();
                        // look up the key in the code book
                        match code.get(*key) {
                            Some(value) => result.push_str(value),
                            None => result.push_str(&r),
                        }
                    }
                    Err(e) => result.push_str(&e),
                }
            }
            _ => result.push(c),
        }
    }
    result
}

/// Builds a variable following the syntax `c0c0*c_nc_n`.
///
/// Assumes the first token was already consumed and is passed as `c0`.
///
/// Errors if the syntax is not fulfilled.
fn gather_variable<T: Iterator<Item = char>>(
    chars: &mut T,
    c0: char,
    c_n: char,
) -> Result<String, String> {
    let mut var = String::from(c0);
    let mut last: Option<char> = None;
    // verify next character is also `c0`
    if let Some(c) = chars.next() {
        var.push(c);
        if c != c0 {
            return Err(var);
        }
        last = Some(c);
    }
    // collect remaining characters until finding double cN occurrences
    while let Some(c) = chars.next() {
        var.push(c);
        if c == c_n {
            // check if the last char was also `cN`
            if last.unwrap() == c_n {
                return Ok(var);
            }
        }
        last = Some(c);
    }
    // never was able to close the variable
    Err(var)
}

#[cfg(test)]
mod test {
    use super::*;

    /// Internal helper test `fn` to generate a sample code book for looking up variables.
    fn create_code() -> StrSwapTable {
        let mut code = StrSwapTable::new();
        code.add("orbit.name", "gates");
        code.add("orbit.library", "rary");
        code.add("orbit.place", "bar");
        code
    }

    #[test]
    fn gather_var() {
        let text = "{ variable }}";
        assert_eq!(
            gather_variable(&mut text.chars(), '{', '}'),
            Ok("{{ variable }}".to_owned())
        );

        let text = "{ variable }";
        assert_eq!(
            gather_variable(&mut text.chars(), '{', '}'),
            Err("{{ variable }".to_owned())
        );

        let text = "variable }";
        assert_eq!(
            gather_variable(&mut text.chars(), '{', '}'),
            Err("{v".to_owned())
        );

        let text = "{variable } } ";
        assert_eq!(
            gather_variable(&mut text.chars(), '{', '}'),
            Err("{{variable } } ".to_owned())
        );
    }

    #[test]
    fn replace_variables() {
        let text = "The quick brown fox jumped over the lazy {{ orbit.name }}.";
        let code = create_code();
        assert_eq!(
            substitute(text.to_owned(), &code),
            "The quick brown fox jumped over the lazy gates.".to_owned()
        );

        let text = "A duck, a bear, and a {{ animal }} walk into a {{  orbit.place   }}...";
        let code = create_code();
        assert_eq!(
            substitute(text.to_owned(), &code),
            "A duck, a bear, and a {{ animal }} walk into a bar...".to_owned()
        );
    }
}
