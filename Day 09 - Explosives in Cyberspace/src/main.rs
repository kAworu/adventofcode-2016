#[macro_use]
extern crate nom;

mod explosives_in_cyberspace {
    /// Represents a node from the `Ezip` "tree". Either an uncompressed chunk of data or a
    /// sub-`Ezip` to be repeated.
    #[derive(Debug)]
    enum EzipNode {
        Uncompressed(String),
        Compressed(usize, Ezip),
    }

    impl EzipNode {
        /// Returns the uncompressed data length for this node.
        fn uncompressed_len(&self) -> usize {
            match *self {
                EzipNode::Uncompressed(ref s) => s.len(),
                EzipNode::Compressed(repeat, ref children) => {
                    repeat * children.uncompressed_len()
                },
            }
        }
    }

    /// Experimental data compression format found in the Easter Bunny HQ.
    #[derive(Debug)]
    pub struct Ezip {
        nodes: Vec<EzipNode>,
    }

    impl Ezip {
        /// Parse a string formated in the Experimental data compression format version 1.
        // XXX: leaking nom stuff through the error, oh well.
        pub fn parse_v1(s: &str) -> Result<Ezip, ::nom::IError> {
                parsing::ezipv1(s).to_full_result()
        }

        /// Parse a string formated in the Experimental data compression format version 2.
        // XXX: leaking nom stuff through the error, oh well.
        pub fn parse_v2(s: &str) -> Result<Ezip, ::nom::IError> {
                parsing::ezipv2(s).to_full_result()
        }

        /// Returns the uncompressed data length of the file.
        pub fn uncompressed_len(&self) -> usize {
            self.nodes.iter().map(|node| node.uncompressed_len()).sum()
        }

        /// Build a new `Ezip` containing the given nodes.
        fn build(nodes: Vec<EzipNode>) -> Ezip {
            Ezip { nodes: nodes }
        }

        /// Build a new `Ezip` containing only one uncompressed node.
        fn build_uncompressed(data: &str) -> Ezip {
            Ezip {
                nodes: vec![EzipNode::Uncompressed(data.to_string())],
            }
        }
    }

    // the Ezip parsing stuff using nom.
    mod parsing {
        use explosives_in_cyberspace::{EzipNode, Ezip};
        use nom::{self, digit};
        use std::str::{self, FromStr};

        // parse a string of digit as usize, used for the compression data length and repeat count.
        named!(number<usize>,
            map_res!(
                map_res!(ws!(digit), str::from_utf8),
                FromStr::from_str
            )
        );

        // helper returning true as long as `x` is not the start of a compression marker.
        fn not_marker_start(x: u8) -> bool {
            x != '(' as u8
        }

        // parse an uncompressed chunk of data (i.e. "decompressed section").
        named!(uncompressed<EzipNode>,
            do_parse!(
                data: map_res!(take_while!(not_marker_start), str::from_utf8) >>
                (EzipNode::Uncompressed(data.trim_right().to_string()))
            )
        );

        // parse a marker (eg. "(3x6)") and return a tuple with its two numbers (eg. `(3, 6)`).
        named!(marker<(usize, usize)>,
            do_parse!(
                char!('(') >> len: number >> char!('x') >> count: number >> char!(')') >>
                (len, count)
            )
        );

        // parse a full marker (eg. "(3x6)") and return only the data length (eg. `3`).
        named!(marker_len<usize>,
            do_parse!(
                char!('(') >> len: number >> char!('x') >> number >> char!(')') >>
                (len)
            )
        );

        // parse a compressed version 1 marker and its associated data, eg. "(3x6)XYZ".
        named!(compressed_v1<EzipNode>,
            do_parse!(
                mark: marker >>
                children: map!(take_str!(mark.0), Ezip::build_uncompressed) >>
                (EzipNode::Compressed(mark.1, children))
            )
        );

        // parse a compressed version 2 marker and its associated data, eg. "(3x6)XYZ".
        named!(compressed_v2<EzipNode>,
            do_parse!(
                mark: peek!(marker) >> // peek! the marker so that length_value! can consume it.
                children: map!(length_value!(marker_len, nodes_v2), Ezip::build) >>
                (EzipNode::Compressed(mark.1, children))
            )
        );

        // parse a chain of compressed and uncompressed chunk.
        named!(nodes_v1<Vec<EzipNode>>, many1!(alt!(compressed_v1 | uncompressed)));
        named!(nodes_v2<Vec<EzipNode>>, many1!(alt!(compressed_v2 | uncompressed)));

        // parse a full Ezip file.
        named!(parse_ezipv1<Ezip>, map!(nodes_v1, Ezip::build));
        named!(parse_ezipv2<Ezip>, map!(nodes_v2, Ezip::build));

        // expose the ezipv1 parser outside this mod.
        pub fn ezipv1(s: &str) -> nom::IResult<&[u8], Ezip> {
            parse_ezipv1(s.as_bytes())
        }

        // expose the ezipv2 parser outside this mod.
        pub fn ezipv2(s: &str) -> nom::IResult<&[u8], Ezip> {
            parse_ezipv2(s.as_bytes())
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

    // parse input as Ezip version 1
    let compressed = Ezip::parse_v1(input.as_str()).unwrap();
    println!("the decompressed length of the file (v1) is {}.", compressed.uncompressed_len());

    // parse input as Ezip version 2
    let compressed = Ezip::parse_v2(input.as_str()).unwrap();
    println!("the decompressed length of the file (v2) is {}.", compressed.uncompressed_len());
}


#[test]
fn part1_first_example() {
    let s = "ADVENT";
    let compressed = Ezip::parse_v1(s).unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 6);
}

#[test]
fn part1_second_example() {
    let s = "A(1x5)BC";
    let compressed = Ezip::parse_v1(s).unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 7);
}

#[test]
fn part1_third_example() {
    let s = "(3x3)XYZ";
    let compressed = Ezip::parse_v1(s).unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 9);
}

#[test]
fn part1_fourth_example() {
    let s = "A(2x2)BCD(2x2)EFG";
    let compressed = Ezip::parse_v1(s).unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 11);
}

#[test]
fn part1_fifth_example() {
    let s = "(6x1)(1x3)A";
    let compressed = Ezip::parse_v1(s).unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 6);
}

#[test]
fn part1_sixth_example() {
    let s = "X(8x2)(3x3)ABCY";
    let compressed = Ezip::parse_v1(s).unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 18);
}

#[test]
fn part2_first_example() {
    let s = "(3x3)XYZ";
    let compressed = Ezip::parse_v2(s).unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 9);
}

#[test]
fn part2_second_example() {
    let s = "X(8x2)(3x3)ABCY";
    let compressed = Ezip::parse_v2(s).unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 20);
}

#[test]
fn part2_third_example() {
    let s = "(27x12)(20x12)(13x14)(7x10)(1x12)A";
    let compressed = Ezip::parse_v2(s).unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 241920);
}

#[test]
fn part2_fourth_example() {
    let s = "(25x3)(3x3)ABC(2x3)XY(5x2)PQRSTX(18x9)(3x2)TWO(5x7)SEVEN";
    let compressed = Ezip::parse_v2(s).unwrap();
    println!("{:?}", compressed);
    assert_eq!(compressed.uncompressed_len(), 445);
}
