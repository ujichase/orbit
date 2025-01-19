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

use crate::core::lang::LangIdentifier;

use super::super::lexer::{Position, Token};
use super::token::VhdlToken;
use std::collections::HashMap;

/// Takes in a list of tokens, and a hashmap of the identifiers and their respective
/// UIE (unique identifier extension).
///
/// Performs a swap on the identifiers (keys) and appends their extensions (values) to write to
/// new VHDL text.
pub fn dyn_symbol_transform(
    tkns: &[Token<VhdlToken>],
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
            VhdlToken::Identifier(id) => {
                match lut.get(&LangIdentifier::Vhdl(id.clone())) {
                    Some(ext) => {
                        let t = id.into_extension(ext).to_string();
                        // compute the extra space shifted for next token
                        transform_diff = t.len() - id.len();
                        (t.len(), t)
                    }
                    None => {
                        let t = id.to_string();
                        (t.len(), t)
                    }
                }
            }
            VhdlToken::Comment(c) => {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::lang::vhdl::{
        dst::dyn_symbol_transform,
        token::{identifier::Identifier, tokenizer::VhdlTokenizer},
    };

    #[test]
    fn simple() {
        let mut map = HashMap::new();
        map.insert(
            LangIdentifier::Vhdl(Identifier::Basic(String::from("adder"))),
            "_sha12345".to_string(),
        );
        map.insert(
            LangIdentifier::Vhdl(Identifier::Extended(String::from("adder_tb"))),
            "_sha12345".to_string(),
        );

        let code: &str = r#"
--! module: adder (name here is untouched)
library ieee;
use ieee.std_logic_1164.all;

entity adder is 
generic (
    WIDTH: positive := 8
);
port (
    cin, adder_in_a, adder_in_b : in  std_logic_vector(WIDTH-1 downto 0); -- comment
    sum, cout : out std_logic_vector(WIDTH-1 downto 0)
);
end entity adder;

/*
two-line
comment */ library ieee;
-- another comment 

architecture rtl of adder is
    CONSTANT GO_ADDR_MMAP:integer:=2#001_1100.001#E14; -- keywords will get converted to lowercase
    constant \magic_character\ : char := 'a';
    constant word : string := "hello world!";
    constant MAGIC_NUM_2 : std_logic_vector(7 downto 0) := 8b"11";
    signal   MAGIC_NUM_3 : bit_vector(3 downto 0) := 0sx"";
begin

end architecture;

entity adder is end entity  adder;


entity \adder_tb\ is generic (WIDTH : positive := 2); end entity \adder_tb\;
        "#;
        let tokens = VhdlTokenizer::from_source_code(&code).into_tokens_all();
        let transform = dyn_symbol_transform(&tokens, &map);
        let result: &str = r#"
--! module: adder (name here is untouched)
library ieee;
use ieee.std_logic_1164.all;

entity adder_sha12345 is
generic (
    WIDTH: positive := 8
);
port (
    cin, adder_in_a, adder_in_b : in  std_logic_vector(WIDTH-1 downto 0); -- comment
    sum, cout : out std_logic_vector(WIDTH-1 downto 0)
);
end entity adder_sha12345;

/*
two-line
comment */ library ieee;
-- another comment 

architecture rtl of adder_sha12345 is
    constant GO_ADDR_MMAP:integer:=2#001_1100.001#E14; -- keywords will get converted to lowercase
    constant \magic_character\ : char := 'a';
    constant word : string := "hello world!";
    constant MAGIC_NUM_2 : std_logic_vector(7 downto 0) := 8b"11";
    signal   MAGIC_NUM_3 : bit_vector(3 downto 0) := 0sx"";
begin

end architecture;

entity adder_sha12345 is end entity  adder_sha12345;


entity \adder_tb_sha12345\ is generic (WIDTH : positive := 2); end entity \adder_tb_sha12345\;
        "#;
        println!("{}", transform);
        assert_eq!(result, transform);
    }
}
