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

use std::collections::HashMap;

use crate::core::lang::{
    lexer::{Position, Token},
    LangIdentifier,
};

use super::token::token::SystemVerilogToken;

/// Takes in a list of tokens, and a hashmap of the identifiers and their respective
/// UIE (unique identifier extension).
///
/// Performs a swap on the identifiers (keys) and appends their extensions (values) to write to
/// new Verilog text.
pub fn dyn_symbol_transform(
    tkns: &[Token<SystemVerilogToken>],
    lut: &HashMap<LangIdentifier, String>,
) -> String {
    let mut result = String::with_capacity(tkns.len());
    let mut tkns_iter = tkns.into_iter();

    let mut prev_pos = Position::new();
    let mut offset: usize = 0;
    let mut transform_diff: usize = 0;
    let mut comment_lines: usize = 0;
    while let Some(tkn) = tkns_iter.next() {
        let pos = tkn.locate().clone();

        let line_diff = pos.line() - prev_pos.line() - comment_lines;
        // add appropriate new lines
        for _ in 0..line_diff {
            result.push('\n')
        }
        let col_diff = if line_diff == 0 {
            transform_diff + pos.col() - prev_pos.col() - offset
        } else {
            pos.col() - 1
        };
        // add appropriate spaces
        for _ in 0..col_diff {
            result.push(' ');
        }
        comment_lines = 0;
        transform_diff = 0;
        // check if the identifier needs to be transformed
        let (diff, text) = match tkn.as_ref() {
            SystemVerilogToken::Identifier(name) => {
                match lut.get(&LangIdentifier::SystemVerilog(name.clone())) {
                    Some(ext) => {
                        let t = name.into_extension(ext).to_string();
                        // compute the extra space shifted for next token
                        transform_diff = t.len() - name.len();
                        (t.len(), t)
                    }
                    None => {
                        let t = name.to_string();
                        (t.len(), t)
                    }
                }
            }
            SystemVerilogToken::Comment(c) => {
                let tmp_pos = c.ending_position();
                // needed to be set to balance for next token
                comment_lines = tmp_pos.line() - 1;
                (tmp_pos.col(), c.to_string())
            }
            _ => {
                let t = tkn.as_ref().to_string();
                (t.len(), t)
            }
        };
        offset = diff;

        // println!("text: {}, os: {}", text, offset);

        result.push_str(&text);
        // update position
        prev_pos = pos.clone();
    }
    result
}
