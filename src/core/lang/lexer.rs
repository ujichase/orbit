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

use std::fmt::Display;
use std::iter::Peekable;

pub trait Tokenize {
    type TokenType;
    type Err;

    fn tokenize(s: &str) -> Vec<Result<Token<Self::TokenType>, TokenError<Self::Err>>>
    where
        <Self as Tokenize>::Err: Display;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<T> {
    position: Position,
    ttype: T,
}

impl<T> Token<T> {
    pub fn as_type(&self) -> &T {
        &self.ttype
    }

    /// Transforms the token into its type.
    pub fn take(self) -> T {
        self.ttype
    }

    /// Decouples the position and token into their separate structs.
    pub fn decouple(self) -> (Position, T) {
        (self.position, self.ttype)
    }

    /// Returns the position in the file where the token was captured.
    pub fn locate(&self) -> &Position {
        &self.position
    }

    /// Creates a new token.
    pub fn new(ttype: T, loc: Position) -> Self {
        Self {
            position: loc,
            ttype: ttype,
        }
    }

    /// Transforms the data into a `Position`.
    pub fn into_position(self) -> Position {
        self.position
    }

    /// References the inner token type.
    pub fn as_ref(&self) -> &T {
        &self.ttype
    }
}

#[derive(Debug, PartialEq)]
pub struct TokenError<T: Display> {
    position: Position,
    err: T,
}

impl<T: Display> TokenError<T> {
    /// Creates a new `TokenError` struct at position `loc` with error `T`.
    pub fn new(err: T, loc: Position) -> Self {
        Self {
            position: loc,
            err: err,
        }
    }
}

impl<T: Display> Display for TokenError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.position, self.err)
    }
}

use std::cmp::Ordering;

#[derive(Debug, PartialEq, Clone, Ord, Eq)]
/// (Line, Col)
pub struct Position(usize, usize);

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.0.partial_cmp(&other.0).unwrap() {
            Ordering::Equal => self.1.partial_cmp(&other.1),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Less => Some(Ordering::Less),
        }
    }
}

impl Position {
    /// Creates a new `Position` struct as line 1, col 0.
    pub fn new() -> Self {
        Position(1, 0)
    }

    /// Creates a `Position` struct at a particular location `line`:`col`.
    pub fn place(line: usize, col: usize) -> Self {
        Self(line, col)
    }

    /// Increments the column counter by 1.
    pub fn next_col(&mut self) {
        self.1 += 1;
    }

    /// Increments the column counter by 1. If the current char `c` is a newline,
    /// it will then drop down to the next line.
    pub fn step(&mut self, c: &char) {
        self.next_col();
        if c == &'\n' {
            self.next_line();
        }
    }

    /// Increments the line counter by 1.
    ///
    /// Also resets the column counter to 0.
    pub fn next_line(&mut self) {
        self.0 += 1;
        self.1 = 0;
    }

    /// Access the line (`.0`) number.
    pub fn line(&self) -> usize {
        self.0
    }

    /// Access the col (`.1`) number.
    pub fn col(&self) -> usize {
        self.1
    }

    /// Appends the position by adding lines and setting column.
    pub fn fast_forward(&mut self, other: &Position) {
        if other.0 > 1 {
            self.0 += other.0;
        }
        match other.0 > 1 {
            true => self.1 = other.1,   // set column
            false => self.1 += other.1, // add column
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{}:{}", self.0, self.1)
    }
}

/// Helps keep the current position in the contents as the characters are consumed.
pub struct TrainCar<T>
where
    T: Iterator<Item = char>,
{
    contents: Peekable<T>,
    loc: Position,
}

impl<T> TrainCar<T>
where
    T: Iterator<Item = char>,
{
    /// Creates a new `TrainCar` struct with an initial position (1, 0) and a
    /// train `s`.
    pub fn new(s: T) -> Self {
        Self {
            loc: Position::new(),
            contents: s.peekable(),
        }
    }

    /// Takes the next char in the iterator and steps the `Position` marker
    /// accordingly, if a char exists.
    pub fn consume(&mut self) -> Option<char> {
        if let Some(c) = self.contents.next() {
            self.loc.step(&c);
            Some(c)
        } else {
            None
        }
    }

    /// References the next char in the iterator, if it exists.
    pub fn peek(&mut self) -> Option<&char> {
        self.contents.peek()
    }

    /// Access the position of the first remaining character.
    pub fn locate(&self) -> &Position {
        &self.loc
    }

    /// References the entire iterator still remaining in `self`.
    pub fn peekable(&self) -> &Peekable<T> {
        &self.contents
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn moving_position() {
        let mut pos = Position::new();
        assert_eq!(pos, Position::place(1, 0));
        pos.next_col();
        assert_eq!(pos, Position::place(1, 1));
        pos.next_col();
        assert_eq!(pos, Position::place(1, 2));
        pos.next_line();
        assert_eq!(pos, Position::place(2, 0));
        pos.next_line();
        assert_eq!(pos, Position::place(3, 0));
    }
}
