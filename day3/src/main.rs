extern crate regex;

mod squares_with_three_sides {

    #[derive(Eq, PartialEq, Copy, Clone, Debug)]
    pub struct Triangle {
        a: u32,
        b: u32,
        c: u32,
    }

    impl Triangle {
        pub fn new(a: u32, b: u32, c: u32) -> Option<Triangle> {
            let xs = [a, b, c];
            let max = *xs.iter().max().unwrap();
            let sum: u32 = xs.iter().sum();
            if (sum - max) > max {
                Some(Triangle { a: a, b: b, c: c })
            } else {
                None
            }
        }
    }
}


use std::io::Read;
use ::regex::Regex;
use squares_with_three_sides::*;

fn main() {
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");
    let mut n = 0;
    let re = Regex::new(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s*$").unwrap();
    for line in input.lines() {
        for cap in re.captures_iter(line) {
            let a: u32 = cap.at(1).expect("invalid input").parse().expect("invalid input");
            let b: u32 = cap.at(2).expect("invalid input").parse().expect("invalid input");
            let c: u32 = cap.at(3).expect("invalid input").parse().expect("invalid input");
            n += match Triangle::new(a, b, c) {
                None => 0,
                Some(_) => 1,
            }
        }
    }
    println!("found {} valid triangles specifications on the graphic design department walls", n);
}


#[test]
fn part1_example() {
    assert_eq!(Triangle::new(5, 10, 25), None);
}
