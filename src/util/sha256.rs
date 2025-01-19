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

//! Implements the SHA-256 algorithm (found under the SHA-2 group).
//! Reference: https://sha256algorithm.com/

use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub struct Sha256Hash {
    digest: [u32; 8],
}

use serde::de::{self};
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;

impl<'de> Deserialize<'de> for Sha256Hash {
    fn deserialize<D>(deserializer: D) -> Result<Sha256Hash, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = Sha256Hash;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 256-character checksum")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Sha256Hash::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for Sha256Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Sha256Hash {
    /// Creates a new blank hash filled with all 0's.
    pub fn new() -> Self {
        Self { digest: [0; 8] }
    }

    /// Creates a new hash filled with `digest`.
    pub fn from_u32s(digest: [u32; 8]) -> Self {
        Self { digest: digest }
    }

    /// Transforms the digest, which is 8 32-bit values, into a series of 32 8-bit
    /// values.
    pub fn into_bytes(self) -> [u8; 32] {
        let mut bytes: [u8; 32] = [0; 32];
        // split every integer into 4 bytes
        let mut index = 0;
        for i in 0..8 {
            for j in (0..4).rev() {
                bytes[index] = (self.digest[i] >> j * 8) as u8;
                index += 1;
            }
        }
        bytes
    }

    /// Displays the checksum to its shorthand form (16 characters).
    pub fn to_string_short(&self) -> String {
        self.to_string().get(0..16).unwrap().to_string()
    }
}

impl FromStr for Sha256Hash {
    type Err = Sha256Error;

    // @TODO design better error handling
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 64 {
            return Err(Sha256Error::BadLen(s.len()));
        }
        let mut digest: [u32; 8] = [0; 8];
        for i in 0..8 {
            digest[i] = u32::from_str_radix(&s[8 * i..8 * i + 8], 16)?;
        }
        Ok(Sha256Hash { digest: digest })
    }
}

impl Display for Sha256Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(for num in self.digest {
            write!(f, "{:08x}", num)?
        })
    }
}

/// Compute the SHA-256 hash function for a slice of bytes.
pub fn compute_sha256(s: &[u8]) -> Sha256Hash {
    // [1] preprocessing
    // convert the string into bytes
    let mut bytes = s.to_vec();
    // compute the input bytes total length to store in 8 bytes
    let message_len: u64 = (bytes.len() as u64) * 8;
    // append a single '1' as 1000 0000 (0x80)
    bytes.push(128);

    // small boost if vector has not already reserved enough space for adding another chunk
    bytes.reserve(64 - (bytes.len() % 64));

    // pad with zeros until data is a multiple of 512 -> 64 bytes
    let zero_cnt_bytes = {
        let zero_cnt = 512 - ((bytes.len() * 8 + 64) % 512);
        if zero_cnt == 512 {
            0
        } else {
            zero_cnt
        }
    } / 8;

    for _ in 0..zero_cnt_bytes {
        bytes.push(0);
    }

    // append 64 bits to the end, where 64 bits represent integer length of original input in binary
    {
        // store as big-endian
        for i in (0..8).rev() {
            bytes.push((message_len >> (8 * i)) as u8);
        }
    }
    assert_eq!(bytes.len() % 64, 0); // 512 bits -> 64 bytes

    // [2] initialize hash values
    let mut hashes: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    // [3] initialize array of round constants
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    // [4] chunk loop
    let chunk_count = bytes.len() / (512 / 8);
    //dbg!(chunk_count);
    for n in 0..chunk_count {
        // access correct slice of chunk from byte vector
        let chunk = &bytes[n * 64..(n + 1) * 64];
        // create 64-entry message schedule array w[0..63] of 32-bit words
        let mut message: [u32; 64] = [0; 64];
        // copy chunk into first 16 words
        for j in 0..16 {
            message[j] = (chunk[4 * j] as u32) << 24
                | (chunk[4 * j + 1] as u32) << 16
                | (chunk[4 * j + 2] as u32) << 8
                | (chunk[4 * j + 3] as u32);
        }

        // extend the first 16 words into the remaining 48 words w[16..63] of the message
        for j in 16..64 {
            //s0 := (w[i-15] rightrotate  7) xor (w[i-15] rightrotate 18) xor (w[i-15] rightshift  3)
            let s0: u32 = message[j - 15].rotate_right(7)
                ^ message[j - 15].rotate_right(18)
                ^ (message[j - 15] >> 3);
            //s1 := (w[i-2] rightrotate 17) xor (w[i-2] rightrotate 19) xor (w[i-2] rightshift 10)
            let s1: u32 = message[j - 2].rotate_right(17)
                ^ message[j - 2].rotate_right(19)
                ^ (message[j - 2] >> 10);
            message[j] = message[j - 16]
                .wrapping_add(s0)
                .wrapping_add(message[j - 7])
                .wrapping_add(s1);
        }

        // initialize current working variables to the current hash values
        let mut wh = hashes;

        // compression function main loop
        for i in 0..64 {
            // S1 := (e rightrotate 6) xor (e rightrotate 11) xor (e rightrotate 25)
            let s1 = wh[4].rotate_right(6) ^ wh[4].rotate_right(11) ^ wh[4].rotate_right(25);
            // ch := (e and f) xor ((not e) and g)
            let ch = (wh[4] & wh[5]) ^ ((!wh[4]) & wh[6]);
            // temp1 := h + S1 + ch + k[i] + w[i]
            let temp1 = wh[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k[i])
                .wrapping_add(message[i]);
            // S0 := (a rightrotate 2) xor (a rightrotate 13) xor (a rightrotate 22)
            let s0 = wh[0].rotate_right(2) ^ wh[0].rotate_right(13) ^ wh[0].rotate_right(22);
            // maj := (a and b) xor (a and c) xor (b and c)
            let maj = (wh[0] & wh[1]) ^ (wh[0] & wh[2]) ^ (wh[1] & wh[2]);
            // temp2 := S0 + maj
            let temp2 = s0.wrapping_add(maj);

            // h := g
            wh[7] = wh[6];
            // g := f
            wh[6] = wh[5];
            // f := e
            wh[5] = wh[4];
            // e := d + temp1
            wh[4] = wh[3].wrapping_add(temp1);
            // d := c
            wh[3] = wh[2];
            // c := b
            wh[2] = wh[1];
            // b := a
            wh[1] = wh[0];
            // a := temp1 + temp2
            wh[0] = temp1.wrapping_add(temp2);
        }
        // add the compressed chunk to the current hash value
        for i in 0..8 {
            hashes[i] = hashes[i].wrapping_add(wh[i]);
        }
    }
    // produce the final hash value (big-endian)
    Sha256Hash { digest: hashes }
}

#[derive(Debug)]
pub enum Sha256Error {
    BadLen(usize),
    InvalidDigit(ParseIntError),
}

impl std::error::Error for Sha256Error {}

impl Display for Sha256Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadLen(l) => write!(f, "invalid length {}; expecting 64", l),
            Self::InvalidDigit(c) => write!(f, "invalid hexadecimal digit {}", c),
        }
    }
}

impl From<ParseIntError> for Sha256Error {
    fn from(e: ParseIntError) -> Self {
        Self::InvalidDigit(e)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn into_bytes() {
        let sum = Sha256Hash {
            digest: [
                0x1010f0f0, 0xaabbccdd, 0x33223322, 0x44554455, 0x98989898, 0xabcdef01, 0xfedcba98,
                0x54637281,
            ],
        };

        assert_eq!(
            sum.into_bytes(),
            [
                0x10, 0x10, 0xf0, 0xf0, 0xaa, 0xbb, 0xcc, 0xdd, 0x33, 0x22, 0x33, 0x22, 0x44, 0x55,
                0x44, 0x55, 0x98, 0x98, 0x98, 0x98, 0xab, 0xcd, 0xef, 0x01, 0xfe, 0xdc, 0xba, 0x98,
                0x54, 0x63, 0x72, 0x81
            ]
        );
    }

    #[test]
    fn it_works() {
        assert_eq!(
            compute_sha256("hello world".as_bytes()),
            Sha256Hash::from_str(
                "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
            )
            .unwrap()
        );

        assert_eq!(
            compute_sha256(&[]),
            Sha256Hash::from_str(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            )
            .unwrap()
        );

        assert_eq!(
            compute_sha256("abc".as_bytes()),
            Sha256Hash::from_str(
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            )
            .unwrap()
        );

        assert_eq!(
            compute_sha256(
                "\
Tyger Tyger, burning bright, 
In the forests of the night; 
What immortal hand or eye, 
Could frame thy fearful symmetry? 

In what distant deeps or skies. 
Burnt the fire of thine eyes? 
On what wings dare he aspire? 
What the hand, dare seize the fire? 

And what shoulder, & what art, 
Could twist the sinews of thy heart? 
And when thy heart began to beat, 
What dread hand? & what dread feet? 

What the hammer? what the chain, 
In what furnace was thy brain? 
What the anvil? what dread grasp, 
Dare its deadly terrors clasp! 

When the stars threw down their spears 
And water'd heaven with their tears: 
Did he smile his work to see? 
Did he who made the Lamb make thee? 

Tyger Tyger burning bright, 
In the forests of the night: 
What immortal hand or eye, 
Dare frame thy fearful symmetry?"
                    .as_bytes()
            ),
            Sha256Hash::from_str(
                "0d732bb7f24e68fb3858646ba33bc9ce3240def191cde285a3f03ad1f763f52d"
            )
            .unwrap()
        );

        assert_eq!(
            compute_sha256(
                "\
She had the jitters
She had the flu
She showed up late
She missed her cue
She kicked the director
She screamed at the crew
And tripped on a prop
And fell in some goo
And ripped her costume
A place or two
Then she forgot
A line she knew
And went “Meow”
Instead of “Moo”
She heard 'em giggle
She heard 'em boo
The programs sailed
The popcorn flew
As she stomped offstage
With a boo-hoo-hoo
The fringe of the curtain
Got caught in her shoe
The set crashed down
The lights did too
Maybe that's why she didn't want to do
An interview."
                    .as_bytes()
            ),
            Sha256Hash::from_str(
                "b8094baea873a8003d6a2bca47511b027042b6537e1dcd0cfe65d8dd8f3651da"
            )
            .unwrap()
        );

        assert_eq!(
            compute_sha256(
                "This sentence will have 63 bytes, to force the algorithm to be-".as_bytes()
            ),
            Sha256Hash::from_str(
                "8c91268f847af008383bd04a43d5937de99594f957d64c1f2816aa7cba833929"
            )
            .unwrap()
        );

        assert_eq!(
            compute_sha256("This sentence will have 63 bytes, to force the algorith".as_bytes()),
            Sha256Hash::from_str(
                "461343e6a222a43ac38073130191c865cf18ee2df0b7998ba6518ba7c57a3b87"
            )
            .unwrap()
        );

        assert_eq!(
            compute_sha256("Go Gators!".as_bytes()),
            Sha256Hash::from_str(
                "9771f8214e6f7fefe33686f88b977aa839e1b69f0d7d937c4cb8401082bd37db"
            )
            .unwrap()
        );
    }

    #[test]
    fn package_zip() {
        let body_bytes = std::fs::read("./tests/t4/archive.zip").unwrap();
        let sum = compute_sha256(&body_bytes);
        assert_eq!(
            sum,
            Sha256Hash::from_str(
                "adcbb89b2a4b08bf0f2c1170129e8c41f5b7c255ec51e0f3d23e6b11da35e904"
            )
            .unwrap()
        )
    }

    #[test]
    fn package_zip2() {
        let body_bytes = std::fs::read("./tests/t4/archive2.zip").unwrap();
        let sum = compute_sha256(&body_bytes);
        println!("{}", sum.to_string());
        assert_eq!(
            sum,
            Sha256Hash::from_str(
                "cc8059fc610a3212cd23f48a1d9b709be8ab61ae1c9563dd4598e38e80bf3385"
            )
            .unwrap()
        )
    }

    #[test]
    fn str_repr() {
        assert_eq!(
            compute_sha256(&[]).to_string(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        assert_eq!(
            compute_sha256("Go Gators!".as_bytes()).to_string(),
            "9771f8214e6f7fefe33686f88b977aa839e1b69f0d7d937c4cb8401082bd37db"
        );

        assert_eq!(
            Sha256Hash::from_str(
                "0d732bb7f24e68fb3858646ba33bc9ce3240def191cde285a3f03ad1f763f52d"
            )
            .unwrap()
            .to_string(),
            "0d732bb7f24e68fb3858646ba33bc9ce3240def191cde285a3f03ad1f763f52d"
        );
    }
}
