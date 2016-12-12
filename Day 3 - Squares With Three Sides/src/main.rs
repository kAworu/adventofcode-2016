mod squares_with_three_sides {

    /// Represent a triangle with three sides length.
    #[derive(Eq, PartialEq, Copy, Clone, Debug)]
    pub struct Triangle(u32, u32, u32);

    impl Triangle {
        /// Create a new triangle given its sides.
        ///
        /// Returns `None` when the sides combination is invalid according to the puzzle
        /// definition:
        /// > In a valid triangle, the sum of any two sides must be larger than
        /// > the remaining side.
        pub fn new(sides: (u32, u32, u32)) -> Option<Triangle> {
            let xs = [sides.0, sides.1, sides.2];
            let max = *xs.iter().max().unwrap();
            let sum: u32 = xs.iter().sum();
            if (sum - max) > max {
                Some(Triangle(sides.0, sides.1, sides.2))
            } else {
                None
            }
        }
    }
}


use std::io::Read;
use squares_with_three_sides::*;

fn main() {
    // acquire data from stdin.
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");

    // parse the input as a vector of u32.
    let mut numbers: Vec<u32> = Vec::new();
    for line in input.lines() {
        for part in line.split_whitespace() {
            numbers.push(part.parse().expect("bad input"));
        }
    }

    // build vectors of triangle for each puzzle parts; rows is for part1, cols for part2.
    let mut rows: Vec<Option<Triangle>> = Vec::new();
    let mut cols: Vec<Option<Triangle>> = Vec::new();
    for chunk in numbers.chunks(9) {
        if chunk.len() != 9 {
            panic!("bad input");
        }
        rows.push(Triangle::new((chunk[0], chunk[1], chunk[2])));
        rows.push(Triangle::new((chunk[3], chunk[4], chunk[5])));
        rows.push(Triangle::new((chunk[6], chunk[7], chunk[8])));
        cols.push(Triangle::new((chunk[0], chunk[3], chunk[6])));
        cols.push(Triangle::new((chunk[1], chunk[4], chunk[7])));
        cols.push(Triangle::new((chunk[2], chunk[5], chunk[8])));
    }

    // report.
    println!("found {} valid triangles specifications on the graphic design department walls \
              horizontally",
             rows.iter().filter_map(|&x| x).count());
    println!("found {} valid triangles specifications on the graphic design department walls \
              vertically",
             cols.iter().filter_map(|&x| x).count());
}


#[test]
fn part1_example() {
    assert_eq!(Triangle::new((5, 10, 25)), None);
}
