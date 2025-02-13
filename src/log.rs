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

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        print!("{}: ", "info".blue());
        println!($($arg)*);
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        print!("{}: ", "warning".yellow());
        println!($($arg)*);
    }};
}

#[macro_export]
macro_rules! hint {
    ($($arg:tt)*) => {{
        use colored::Colorize;
        print!("{}: ", "hint".green());
        println!($($arg)*);
    }};
}
