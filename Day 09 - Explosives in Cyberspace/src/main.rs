#[macro_use]
extern crate nom;

mod explosives_in_cyberspace {
    /// Represents a string repeated one or more times.
    #[derive(Debug)]
    struct ExpzipToken {
        data: String,
        repeat: usize,
    }

    impl ExpzipToken {
        /// Create a new compressed, `repeat` is expected to be greater than one.
        fn compressed(data: String, repeat: usize) -> ExpzipToken {
            ExpzipToken { data: data, repeat: repeat }
        }

        /// Create a new uncompressed token.
        fn uncompressed(data: String) -> ExpzipToken {
            Self::compressed(data, 1)
        }

        /// Returns the uncompressed data length for this token.
        fn uncompressed_len(&self) -> usize {
            self.data.len() * self.repeat
        }
    }

    /// Experimental data compression format found in the Easter Bunny HQ.
    #[derive(Debug)]
    pub struct Expzip {
        tokens: Vec<ExpzipToken>,
    }

    impl Expzip {
        /// Returns the uncompressed data length of the file.
        pub fn uncompressed_len(&self) -> usize {
            self.tokens.iter().map(|token| token.uncompressed_len()).sum()
        }
    }

    // the parsing module impl Str for Expzip using nom.
    mod parsing {
        use explosives_in_cyberspace::{ExpzipToken, Expzip};
        use nom::{self, digit};
        use std::str::{self, FromStr};

        // parse a string of digit as usize, used for the compression data length and repeat count.
        named!(number<usize>,
            map_res!(
                map_res!(ws!(digit), str::from_utf8),
                FromStr::from_str
            )
        );

        // parse a compressed marker and its associated data (i.e. something like "(3x3)XYZ").
        named!(compressed<ExpzipToken>,
            do_parse!(
                char!('(') >>
                len: number >>
                char!('x') >>
                count: number >>
                char!(')') >>
                to_repeat: take_str!(len) >>
                (ExpzipToken::compressed(to_repeat.to_string(), count))
            )
        );

        // helper returning true as long as `x` is not the start of a compression marker.
        fn not_marker_start(x: u8) -> bool {
            x != '(' as u8
        }

        // parse an uncompressed chunk of data (i.e. "decompressed section").
        named!(uncompressed<ExpzipToken>,
            do_parse!(
                data: map_res!(take_while!(not_marker_start), str::from_utf8) >>
                (ExpzipToken::uncompressed(data.trim_right().to_string()))
            )
        );

        // parse a chain of compressed and uncompressed chunk.
        named!(tokens<Vec<ExpzipToken>>, many1!(alt!(compressed | uncompressed)));

        // parse a full Expzip file.
        named!(parse_expzip<Expzip>, do_parse!(xs: tokens >> (Expzip { tokens: xs })));

        impl FromStr for Expzip {
            type Err = nom::IError;

            fn from_str(s: &str) -> Result<Expzip, Self::Err> {
                parse_expzip(s.as_bytes()).to_full_result()
            }
        }
    }
}


use std::io::Read;
use explosives_in_cyberspace::*;

fn main() {
    // acquire data from stdin.
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");

    let compressed: Expzip = input.parse().unwrap();
    println!("the decompressed length of the file is {}.", compressed.uncompressed_len());
}


#[test]
fn part1_first_example() {
    let compressed: Expzip = "ADVENT".parse().unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 6);
}

#[test]
fn part1_second_example() {
    let compressed: Expzip = "A(1x5)BC".parse().unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 7);
}

#[test]
fn part1_third_example() {
    let compressed: Expzip = "(3x3)XYZ".parse().unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 9);
}

#[test]
fn part1_fourth_example() {
    let compressed: Expzip = "A(2x2)BCD(2x2)EFG".parse().unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 11);
}

#[test]
fn part1_fifth_example() {
    let compressed: Expzip = "(6x1)(1x3)A".parse().unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 6);
}

#[test]
fn part1_sixth_example() {
    let compressed: Expzip = "X(8x2)(3x3)ABCY".parse().unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 18);
}
