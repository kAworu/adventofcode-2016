mod squares_with_three_sides {

    #[derive(Eq, PartialEq, Copy, Clone, Debug)]
    pub struct Triangle {
        a: u32,
        b: u32,
        c: u32,
    }

    impl Triangle {
        pub fn new(a: u32, b: u32, c: u32) -> Option<Triangle> {
            let sides = [a, b, c];
            let max = *sides.iter().max().unwrap();
            let sum: u32 = sides.iter().sum();
            if (sum - max) > max {
                Some(Triangle { a: a, b: b, c: c })
            } else {
                None
            }
        }
    }
}


use std::io::Read;
use squares_with_three_sides::*;

fn main() {
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");
    let mut numbers: Vec<u32> = Vec::new();
    for line in input.lines() {
        for part in line.split(" ") {
            if let Ok(num) = part.parse::<u32>() {
                numbers.push(num);
            }
        }
    }

    let mut rows: Vec<Option<Triangle>> = Vec::new();
    let mut cols: Vec<Option<Triangle>> = Vec::new();
    for chunk in numbers.chunks(9) {
        if chunk.len() != 9 {
            panic!("bad input");
        }
        rows.push(Triangle::new(chunk[0], chunk[1], chunk[2]));
        rows.push(Triangle::new(chunk[3], chunk[4], chunk[5]));
        rows.push(Triangle::new(chunk[6], chunk[7], chunk[8]));
        cols.push(Triangle::new(chunk[0], chunk[3], chunk[6]));
        cols.push(Triangle::new(chunk[1], chunk[4], chunk[7]));
        cols.push(Triangle::new(chunk[2], chunk[5], chunk[8]));
    }
    println!("found {} valid triangles specifications on the graphic design department walls horizontally",
             rows.iter().filter_map(|&x| x).count());
    println!("found {} valid triangles specifications on the graphic design department walls vertically",
             cols.iter().filter_map(|&x| x).count());
}


#[test]
fn part1_example() {
    assert_eq!(Triangle::new(5, 10, 25), None);
}
