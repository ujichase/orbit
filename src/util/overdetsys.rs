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

//! Overdetermined system solver

/// Given a partially defined `target`, reduce the `space` for solutions that match
/// the `target` as much as possible.
///
/// @TODO accept a `fn` as argument to customize how to 'filter' at each part...useful for search command
pub fn reduce<'a, T, U>(mut space: Vec<Vec<U>>, target: T) -> Vec<Vec<U>>
where
    U: std::cmp::PartialEq + 'a,
    T: Iterator<Item = &'a U> + 'a,
{
    // at each level provide filter until target provides no more information
    let mut target_iter = target.enumerate();
    while let Some((i, t)) = target_iter.next() {
        space = space
            .into_iter()
            .filter(|f| f.get(i).unwrap() == t)
            .collect();
    }
    space
}

/// Given a partially defined `target`, try to find the unique occurence among the entire
/// `space`.
///
/// Errors if there are multiple solutions (ambiguous) or no solution.
pub fn solve<'a, T, U>(space: Vec<Vec<U>>, target: T) -> Result<Vec<U>, OverDetSysError<U>>
where
    U: std::cmp::PartialEq + 'a,
    T: Iterator<Item = &'a U> + 'a,
{
    let mut space = reduce(space, target);
    match space.len() {
        0 => Err(OverDetSysError::NoSolution),
        1 => Ok(space.pop().unwrap()),
        _ => Err(OverDetSysError::Ambiguous(space)),
    }
}

#[derive(Debug, PartialEq)]
pub enum OverDetSysError<T: std::cmp::PartialEq> {
    Ambiguous(Vec<Vec<T>>),
    NoSolution,
}

impl<T> std::error::Error for OverDetSysError<T> where T: std::cmp::PartialEq + std::fmt::Debug {}

impl<T> std::fmt::Display for OverDetSysError<T>
where
    T: std::cmp::PartialEq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Ambiguous(_) => write!(f, "multiple solutions"),
            Self::NoSolution => write!(f, "no solution"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn one_soln() {
        let space = vec![vec![10, 0, 9], vec![4, 1, 2], vec![0, 1, 2]];
        let nums = vec![4];
        assert_eq!(reduce(space, nums.iter()), vec![vec![4, 1, 2],]);

        let space = vec![vec![10, 0, 9], vec![4, 1, 2], vec![4, 2, 5]];
        let nums = vec![4, 2];
        assert_eq!(reduce(space, nums.iter()), vec![vec![4, 2, 5],]);
    }

    #[test]
    fn mult_soln() {
        let space = vec![vec![10, 0, 9], vec![4, 1, 2], vec![0, 1, 2], vec![4, 9, 5]];
        let nums = vec![4];
        assert_eq!(
            reduce(space, nums.iter()),
            vec![vec![4, 1, 2], vec![4, 9, 5],]
        );
    }

    use crate::core::pkgid;
    use std::str::FromStr;

    #[test]
    fn reduce_pkgid() {
        // one solution
        let space = vec![
            pkgid::PkgId::from_str("ks-tech.rary1.ip1")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary1.ip2")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary1.ip3")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary2.ip1")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary2.ip2")
                .unwrap()
                .into_full_vec()
                .unwrap(),
        ];
        let target = pkgid::PkgId::from_str("ip3").unwrap();
        assert_eq!(
            reduce(space, target.iter()),
            vec![pkgid::PkgId::from_str("ks-tech.rary1.ip3")
                .unwrap()
                .into_full_vec()
                .unwrap(),]
        );

        // one solution (comparing as `PkgPart`)
        let space = vec![
            pkgid::PkgId::from_str("ks-tech.rary1.ip1")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary1.ip2")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary1.ip3")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary2.ip1")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary2.ip2")
                .unwrap()
                .into_full_vec()
                .unwrap(),
        ];
        let target = pkgid::PkgId::from_str("IP3").unwrap();
        assert_eq!(
            reduce(space, target.iter()),
            vec![vec![
                pkgid::PkgPart::from_str("ip3").unwrap(),
                pkgid::PkgPart::from_str("rary1").unwrap(),
                pkgid::PkgPart::from_str("ks-tech").unwrap()
            ],]
        );

        // no solution
        let space = vec![
            pkgid::PkgId::from_str("ks-tech.rary1.ip1")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary1.ip2")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary1.ip3")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary2.ip1")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary2.ip2")
                .unwrap()
                .into_full_vec()
                .unwrap(),
        ];
        let target = pkgid::PkgId::from_str("ip3.unknown").unwrap();
        assert_eq!(
            reduce(space, target.iter()),
            Vec::<Vec<pkgid::PkgPart>>::new()
        );

        // multiple solutions
        let space = vec![
            pkgid::PkgId::from_str("ks-tech.rary1.ip1")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary1.ip2")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary1.ip3")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary2.ip1")
                .unwrap()
                .into_full_vec()
                .unwrap(),
            pkgid::PkgId::from_str("ks-tech.rary2.ip2")
                .unwrap()
                .into_full_vec()
                .unwrap(),
        ];
        let target = pkgid::PkgId::from_str("ip2").unwrap();
        assert_eq!(
            reduce(space, target.iter()),
            vec![
                pkgid::PkgId::from_str("ks-tech.rary1.ip2")
                    .unwrap()
                    .into_full_vec()
                    .unwrap(),
                pkgid::PkgId::from_str("ks-tech.rary2.ip2")
                    .unwrap()
                    .into_full_vec()
                    .unwrap()
            ]
        );
    }
}
