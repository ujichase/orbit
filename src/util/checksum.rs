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

use crate::util::sha256;
use std::path::PathBuf;

/// Given a list of files, compute a single sha256 digest to encapsulate the
/// entire directory state.
///
/// Assumes the filepaths are already sorted before entering this function. It
/// provides a cross-compatible mode for computing a sha256 over a series of
/// files by removing \r carriage return bytes from windows system file reads.
/// This function also skips binary files (not intended for reading) by detecting
/// if a NUL character appears in the byte vector.
pub fn checksum(files: &[String], root: &PathBuf) -> sha256::Sha256Hash {
    // determine the amount of bytes required
    let total_hashes = files.len() + 1;
    let mut total_bytes = Vec::<u8>::with_capacity(total_hashes * 32);

    let mut filename_bytes = Vec::<u8>::new();
    // use a single vector to keep allocated capacity throughout rounds
    let mut bytes = Vec::new();
    // perform a hash on contents
    for file in files {
        bytes.clear();
        bytes.append(&mut std::fs::read(&root.join(file)).expect("failed to read as bytes"));
        // detect and skip binary-encoded files (.pdf, .jpg, etc.) by reading NUL char
        if bytes.contains(&0x00) == true {
            continue;
        }
        // @NOTE windows uses \r\n for newlines, compared to unix systems using just \n
        bytes = bytes
            .into_iter()
            .filter(|f| f != &0x0d)
            .collect::<Vec<u8>>();
        total_bytes.append(&mut sha256::compute_sha256(&bytes).into_bytes().to_vec());
        filename_bytes.append(&mut file.as_bytes().to_vec());
    }
    // perform hash on filenames
    total_bytes.append(
        &mut sha256::compute_sha256(&filename_bytes)
            .into_bytes()
            .to_vec(),
    );

    // perform hash on all hashes
    sha256::compute_sha256(&total_bytes)
}

#[cfg(test)]
mod test {
    use std::env::set_current_dir;

    use super::*;

    #[test]
    fn it_works() {
        // use list of files
        let files = vec![
            "tests/t3/file1.txt".to_owned(),
            "tests/t3/file2.txt".to_owned(),
            "tests/t3/file3.txt".to_owned(),
        ];
        let sum1 = checksum(&files, &PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        assert_eq!(
            sum1,
            sha256::Sha256Hash::from_u32s([
                2852469063, 2394267693, 3536796411, 3444057503, 3461580948, 2722782168, 3560248183,
                1134838683
            ])
        );

        // modifying a file name results in a different hash
        let files = vec![
            "tests/t3/file1.txt".to_owned(),
            "tests/t3/file2.txt".to_owned(),
            "tests/t3/file3copy.txt".to_owned(), // same contents as file3.txt
        ];
        assert_eq!(
            std::fs::read("tests/t3/file3.txt").unwrap(),
            std::fs::read("tests/t3/file3copy.txt").unwrap(),
            "file3 and file3copy must have same contents"
        );
        assert_ne!(
            checksum(&files, &PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
            sum1
        );

        // taking away a file results in a different hash
        let files = vec![
            "tests/t3/file1.txt".to_owned(),
            "tests/t3/file2.txt".to_owned(),
        ];
        let sum2 = checksum(&files, &PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        assert_ne!(sum2, sum1);

        // adding a file results in a different hash
        let files = vec![
            "tests/t3/file1.txt".to_owned(),
            "tests/t3/file2.txt".to_owned(),
            "tests/t3/file3.txt".to_owned(),
            "Cargo.toml".to_owned(),
        ];
        let sum3 = checksum(&files, &PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        assert_ne!(sum3, sum1);
        assert_ne!(sum3, sum2);
    }

    #[test]
    fn from_filesystem() {
        set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
        let test_files = crate::util::filesystem::gather_current_files(
            &std::path::PathBuf::from("./tests/t3"),
            false,
        );
        println!("{:?}", test_files);
        let checksum = crate::util::checksum::checksum(
            &test_files,
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        );
        assert_eq!(
            checksum,
            sha256::Sha256Hash::from_u32s([
                2993778881, 3869022948, 506055411, 4272900414, 2495153976, 2810665789, 3654766766,
                1404775970
            ])
        );
    }
}
