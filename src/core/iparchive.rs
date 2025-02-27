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

use super::ip::Ip;
use super::lockfile::LockFile;
use super::manifest::Manifest;
use crate::util::anyerror::{AnyError, Fault};
use crate::util::compress;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use zip::ZipArchive;

const ARCHIVE_MARKER: [u8; 4] = [0xc7, 0x9e, 0xf1, 0x6b];

pub const ARCHIVE_EXT: &str = "ip";

/// Number of bytes to read a [u32] value.
const U32_SIZE: usize = 4;

pub type IpBytesZipped = Vec<u8>;

/// The IP archive stores the compressed version of a project along with any
/// metadata in its 'header'. The format is: \[MARKER HEADER_LEN HEADER_BYTES PROJECT_DIR_BYTES].
#[derive(Debug, PartialEq)]
pub struct IpArchive {
    manifest: Manifest,
    lock: LockFile,
    /// Compressed data containing the [Ip].
    archive: IpBytesZipped,
}

impl IpArchive {
    fn slice(buf: &[u8], offset: usize, size: usize) -> &[u8] {
        &buf[offset..offset + size]
    }

    pub fn read(path: &PathBuf) -> Result<Self, Fault> {
        let contents = fs::read(&path)?;
        Self::parse(contents, false, &path)
    }

    /// Converts the series of bytes into the [String] to be read as the struct.
    ///
    /// Returns [None] if there is any point of failure.
    fn parse_struct<T: FromStr>(bytes: &[u8], offset: usize) -> Option<(T, usize)> {
        // attempt to read the size
        let len: usize = {
            let size_bytes: [u8; 4] = match Self::slice(&bytes, offset, U32_SIZE).try_into() {
                Ok(arr) => arr,
                Err(_) => return None,
            };
            u32::from_be_bytes(size_bytes) as usize
        };
        // attempt to parse from string
        match String::from_utf8(Self::slice(&bytes, offset + U32_SIZE, len).to_vec()) {
            Ok(s) => match T::from_str(&s) {
                Ok(t) => Some((t, len + U32_SIZE)),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    /// Parses according to version of [IpArchive] format.
    ///
    /// The `repairing` argument should be asserted only when a repair process
    /// is occurring.
    fn parse(buf: Vec<u8>, repairing: bool, path: &PathBuf) -> Result<Self, Fault> {
        // read the marker back to verify the file is for orbit
        let marker: [u8; 4] = Self::slice(&buf, 0, ARCHIVE_MARKER.len()).try_into()?;
        if marker != ARCHIVE_MARKER {
            return Err(AnyError(format!("{}", "The download file is corrupted")))?;
        }

        // read length of header
        let header_offset = ARCHIVE_MARKER.len();
        let header_len: usize =
            u32::from_be_bytes(Self::slice(&buf, header_offset, U32_SIZE).try_into()?) as usize;

        // slice to the bytes for the relevant zipped archive
        let archive_offset: usize = header_offset + U32_SIZE + header_len;
        let archive = &buf[archive_offset..];

        // decompress the header bytes
        let mut d = ZlibDecoder::new(Self::slice(&buf, header_offset + U32_SIZE, header_len));
        let mut header_bytes = Vec::new();
        d.read_to_end(&mut header_bytes)?;

        // parse the decompressed header bytes

        // handle manifest
        let mut offset: usize = 0;
        let (man, bytes_read): (Manifest, usize) = match Self::parse_struct(&header_bytes, offset) {
            Some(t) => t,
            None => match repairing {
                true => panic!("Repairing function failed for manifest"),
                false => {
                    crate::info!("{}", "failed to parse downloaded file's header bytes; running repair function ...");
                    let repaired_bytes = Self::repair(archive, &path)?;
                    match Self::parse(repaired_bytes, true, &path) {
                        Ok(rp) => {
                            crate::info!("{}", "repair successful");
                            return Ok(rp);
                        }
                        Err(e) => return Err(e)?,
                    }
                }
            },
        };
        // handle lockfile
        offset += bytes_read;
        let (lock, _bytes_read): (LockFile, usize) = match Self::parse_struct(&header_bytes, offset)
        {
            Some(t) => t,
            None => match repairing {
                true => panic!("Repairing function failed for lockfile"),
                false => {
                    crate::info!("{}", "failed to parse downloaded file's header bytes; running repair function ...");
                    let repaired_bytes = Self::repair(archive, &path)?;
                    match Self::parse(repaired_bytes, true, &path) {
                        Ok(rp) => {
                            crate::info!("{}", "repair successful");
                            return Ok(rp);
                        }
                        Err(e) => return Err(e)?,
                    }
                }
            },
        };
        // @todo: handle stats?

        // offset += bytes_read;

        Ok(Self {
            manifest: man,
            lock: lock,
            archive: archive.to_vec(),
        })
    }

    /// Fixes any issues with header bytes.
    ///
    /// This function will skip the header bytes, unzip the archive into a
    /// temporary location, and then re-run the `write` function to update the
    /// header bytes.
    ///
    /// This function is helpful whenever there is an update to the download
    /// compression algorithm and new data gets stored in the header.
    pub fn repair(archive: &[u8], path: &PathBuf) -> Result<Vec<u8>, Fault> {
        // place the dependency into a temporary directory
        let dir = tempfile::tempdir()?.into_path();
        if let Err(e) = IpArchive::extract(&archive, &dir) {
            fs::remove_dir_all(dir)?;
            return Err(e);
        }
        // load the IP
        let extracted_ip = match Ip::load(dir.clone(), false, false) {
            Ok(x) => x,
            Err(e) => {
                fs::remove_dir_all(dir)?;
                return Err(e);
            }
        };
        // re-perform a write
        let repaired_bytes = Self::write(&extracted_ip, &path)?;
        Ok(repaired_bytes)
    }

    /// Separates the inner data into their own structs.
    ///
    /// This function is useful for implementing From<[IpArchive]> for [Ip].
    pub fn decouple(self) -> (Manifest, LockFile, IpBytesZipped) {
        (self.manifest, self.lock, self.archive)
    }

    /// Unzips the archive and places it at `dest`. The `dest` path will be
    /// the root folder of the decompressed archive.
    ///
    /// Assumes `dest` does not exist.
    pub fn extract(bytes: &[u8], dest: &Path) -> Result<(), Fault> {
        let mut temp_file = tempfile::tempfile()?;
        temp_file.write_all(bytes)?;
        let mut zip_archive = ZipArchive::new(temp_file)?;
        zip_archive.extract(dest)?;

        Ok(())
    }

    /// Stores the project's state and additional metadata into a .zip archive.
    pub fn write(ip: &Ip, dest: &PathBuf) -> Result<Vec<u8>, Fault> {
        // compress the ip package
        compress::write_zip_dir(ip.get_root(), &dest)?;
        // read back the bytes
        let archive_bytes = fs::read(&dest)?;

        // compress the header bytes
        let header_bytes = {
            // create a Zlib encoder for compression scheme
            let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
            let embedded_data = vec![
                // get the manifest bytes
                ip.get_man().to_string(),
                // get the lockfile bytes
                ip.get_lock().to_string(),
            ];
            for data in embedded_data {
                // write the size of the string
                e.write_all(&(data.len() as u32).to_be_bytes())?;
                // write the string contents
                e.write_all(&data.as_bytes())?;
            }
            // complete the compression algorithm
            e.finish()?
        };

        // open the destination file and truncate it to overwrite with new scheme
        let mut file = File::options().write(true).truncate(true).open(&dest)?;

        // write the marker to start the file
        file.write(&ARCHIVE_MARKER)?;

        // write the number of compressed header bytes to the archive
        file.write(&(header_bytes.len() as u32).to_be_bytes())?;

        // write the compressed bytes
        file.write(&header_bytes)?;

        // write the entire compressed file back
        file.write(&archive_bytes)?;

        let mut all_bytes = header_bytes;
        all_bytes.extend(archive_bytes);
        Ok(all_bytes)
    }

    /// Detects all Ip found as archives.
    pub fn detect_all(dir: &PathBuf) -> Result<Vec<Ip>, Fault> {
        // for each .ip file
        fs::read_dir(&dir)?
            .filter_map(|result| if let Ok(r) = result { Some(r) } else { None })
            .map(|entry| entry.path().to_path_buf())
            .filter(|path| path.extension().is_some() && path.extension().unwrap() == ARCHIVE_EXT)
            .map(|path| match IpArchive::read(&path) {
                Ok(arc) => Ok(Ip::from(arc)),
                Err(e) => Err(e),
            })
            .collect()
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn write_and_read() {

//     }
// }
