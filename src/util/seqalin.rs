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

//! File     : seqalin.rs  
//! Author   : Chase Ruskin  
//! Topic    : Dynamic Programming
//! Abstract :
//!     Given two strings `s1` and `s2`, find a min-cost alignment. Costs are
//!     supplied to _gaps_ and _mismatches_.

type Cost = usize;

/// Given two strings `s1` of length _n_ and `s2` of length _m_, find a min-cost
/// alignment. Costs are defined as gap penalties and mismatch penalties.
///
/// __time complexity__: O(nm)   
/// __space complexity__: O(nm)
///
/// Note: Case sensitivity is not applied within the function.
fn sequence_alignment(s1: &str, s2: &str, gap_penalty: Cost, mismatch_penalty: Cost) -> Cost {
    // create 2D cache filling 0th row and 0th col with gap penalties
    let mut lut = Vec::<Vec<usize>>::with_capacity(s1.len() + 1);
    for i in 0..=s1.len() {
        lut.push(Vec::<usize>::with_capacity(s2.len() + 1));
        for j in 0..=s2.len() {
            match i {
                0 => lut[i].push(j * gap_penalty),
                _ => match j {
                    0 => lut[i].push(i * gap_penalty),
                    _ => lut[i].push(0),
                },
            }
        }
    }
    let min3 = |x, y, z| -> Cost {
        let mut min = x;
        if y < min {
            min = y;
        }
        if z < min {
            min = z;
        }
        min
    };
    // note: enumeration starts at '0' but we want to avoid filling in those
    // indices because they were already computed (thus [i+1][j+1] is used).
    let mut s1_it = s1.chars().enumerate();
    while let Some((i, c1)) = s1_it.next() {
        let mut s2_it = s2.chars().enumerate();
        while let Some((j, c2)) = s2_it.next() {
            // choose minimum cost of 3 options
            lut[i + 1][j + 1] = min3(
                mismatch_penalty * ((c1 != c2) as Cost) + lut[i][j],
                gap_penalty + lut[i][j + 1],
                gap_penalty + lut[i + 1][j],
            );
        }
    }
    lut[s1.len()][s2.len()]
}

/// Given a word `s` and a known set of words `bank`, determine which word has
/// the minimum edit distance to the given word while being below the `threshold`.
///
/// The `gap_penalty` and `mismatch penalty` for sequence alignment are internally set.
pub fn sel_min_edit_str<'a, T: AsRef<str>>(
    s: &str,
    bank: &'a [T],
    threshold: Cost,
) -> Option<&'a str> {
    let (w, c) = bank
        .iter()
        .map(|f| (f, sequence_alignment(s, f.as_ref(), 1, 1)))
        .min_by(|x, y| x.1.cmp(&y.1))?;
    if c < threshold {
        Some(w.as_ref())
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(sequence_alignment("identity", "similarity", 2, 1), 8);
        assert_eq!(sequence_alignment("palate", "palette", 2, 1), 3);
        assert_eq!(sequence_alignment("ctaccg", "tacatg", 2, 1), 5);
        assert_eq!(sequence_alignment("stop", "tops", 2, 1), 4);
        assert_eq!(sequence_alignment("ocurrance", "occurrence", 2, 1), 3);
        assert_eq!(sequence_alignment("go gators", "go gators", 2, 1), 0);
        assert_eq!(sequence_alignment("", "alpha", 2, 1), 10);
        assert_eq!(sequence_alignment("", "", 2, 1), 0);
        assert_eq!(sequence_alignment("--verbsoe", "--verbose", 1, 1), 2);
        assert_eq!(sequence_alignment("--verbsoe", "--version", 1, 1), 3);
        // case sensitivity is not applied inside the fn
        assert_eq!(sequence_alignment("ALPHA", "alpha", 2, 1), 5);
    }

    #[test]
    fn get_closest_word() {
        let bank: Vec<&str> = vec![];
        assert_eq!(sel_min_edit_str("word", &bank, 3), None);

        let bank: Vec<&str> = vec!["run", "check", "build", "plan", "config", "play", "digit"];

        assert_eq!(sel_min_edit_str("buif", &bank, 3), Some("build"));
        assert_eq!(sel_min_edit_str("word", &bank, 3), None);
        assert_eq!(sel_min_edit_str("plug", &bank, 3), Some("plan"));
        assert_eq!(sel_min_edit_str("cck", &bank, 3), Some("check"));
        assert_eq!(sel_min_edit_str("digt", &bank, 3), Some("digit"));
    }
}
