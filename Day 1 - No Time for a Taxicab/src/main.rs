extern crate rand;

mod no_time_for_a_taxicab {
    use ::std::str::FromStr;
    use ::std::collections::HashSet;
    use ::rand::Rng;

    /// Used to represent a Cardinal direction.
    #[derive(Copy, Clone, Debug)]
    enum Direction {
        North,
        East,
        South,
        West,
    }

    /// Represent a position on the city grid.
    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    pub struct Point {
        x: i32,
        y: i32,
    }

    impl Point {
        /// Generate a new random `Point`.
        pub fn random() -> Point {
            let mut rng = ::rand::thread_rng();
            // take our random coordinates from the "small" set of i16 in order to generate a
            // "central" random point "far from the edges" of our Point representation (i.e. i32).
            Point {
                x: rng.gen::<i16>() as i32,
                y: rng.gen::<i16>() as i32,
            }
        }

        /// Compute the "snake distance" from a given other `Point`.
        /// see [Taxicab geometry](https://en.wikipedia.org/wiki/Taxicab_geometry)
        pub fn snake_distance(&self, other: &Self) -> u32 {
            (self.x - other.x).abs() as u32 + (self.y - other.y).abs() as u32
        }
    }

    /// Represent an instruction from the Easter Bunny Recruiting Document.
    #[derive(Copy, Clone, Debug)]
    enum Instruction {
        TurnRight,
        TurnLeft,
        Walk(i32), // NOTE: i32 allow us walk backward
    }

    impl FromStr for Instruction {
        type Err = String;

        /// Parse a string into an `Instruction`.
        ///
        /// Expect `s` to be either "R", "L", or a number.
        fn from_str(s: &str) -> Result<Instruction, String> {
            match s {
                "R" => Ok(Instruction::TurnRight),
                "L" => Ok(Instruction::TurnLeft),
                _ => {
                    if let Ok(stepcount) = s.parse::<i32>() {
                        Ok(Instruction::Walk(stepcount))
                    } else {
                        Err(format!("{}: unrecognized walking step count", s))
                    }
                }
            }
        }
    }

    /// represent an Easter Bunny Recruiting Document.
    #[derive(Debug)]
    pub struct RecruitingDocument {
        starting_point: Point,
        initial_direction: Direction,
        instructions: Vec<Instruction>,
    }

    impl FromStr for RecruitingDocument {
        type Err = String;

        /// parse a string into a `RecruitingDocument`.
        ///
        /// Expect `s` to look like [the puzzle input](input.txt) or examples. Only the
        /// `instructions` are parsed, `initial_direction` is always `Direction::North` and
        /// `starting_point` is generated randomly.
        fn from_str(s: &str) -> Result<RecruitingDocument, String> {
            let tokens: Vec<&str> = s.split(',').map(|s| s.trim()).collect();
            let mut instructions = Vec::new();
            for token in tokens.iter() {
                if token.len() < 2 {
                    return Err(format!("{}: unrecognized instruction", token));
                }
                // NOTE: this implementation is actually more permissive than documented:
                // - token == "12"  would be parsed as (Walk(1), Walk(2))
                // - token == "1L"  would be parsed as (Walk(1), TurnLeft)
                // - token == "LR"  would be parsed as (TurnLeft, TurnRight)
                // - token == "R-1" would be parsed as (TurnRight, Walk(-1))
                // Also negative numbers for Walk(_) could be accepted.
                let direction: Instruction = try!(token[..1].parse());
                let stepcount: Instruction = try!(token[1..].parse());
                instructions.push(direction);
                instructions.push(stepcount);
            }
            Ok(RecruitingDocument {
                starting_point: Point::random(),
                initial_direction: Direction::North,
                instructions: instructions,
            })
        }
    }

    impl RecruitingDocument {
        /// Expose a public method to borrow a reference to the document's `starting_point`.
        pub fn starting_point(&self) -> &Point {
            &self.starting_point
        }
    }

    /// Represent someone able to follow the Easter Bunny Recruiting Document instructions.
    #[derive(Debug)]
    pub struct Traveler {
        position: Point,
    }

    impl Traveler {
        /// Create a new `Traveler` at the given `landing_point`.
        pub fn airdrop_at(landing_point: Point) -> Traveler {
            Traveler { position: landing_point }
        }

        /// Compute the final point and the first point visited twice after having completely
        /// followed the given `RecruitingDocument` instructions.
        ///
        /// return a tuple `t` with two values: `t.0` is the final `Point` and `t.1` the optional
        /// first `Point` visited twice.
        // NOTE: This method does not update the state of self, the puzzle description clearly
        // state that we don't have the time to actually _perform_ the instructions: we only need
        // to _find_ the Easter Bunny Headquarters position(s) in order to compute the distance(s).
        pub fn follow(&self, document: &RecruitingDocument) -> (Point, Option<Point>) {
            let (mut position, mut direction) = (self.position, document.initial_direction);
            let mut visited = HashSet::new();
            visited.insert(position);
            let mut first_position_visited_twice = None;
            for instruction in &document.instructions {
                match *instruction {
                    Instruction::TurnRight => {
                        direction = match direction {
                            Direction::North => Direction::East,
                            Direction::East => Direction::South,
                            Direction::South => Direction::West,
                            Direction::West => Direction::North,
                        }
                    }
                    Instruction::TurnLeft => {
                        direction = match direction {
                            Direction::North => Direction::West,
                            Direction::East => Direction::North,
                            Direction::South => Direction::East,
                            Direction::West => Direction::South,
                        }
                    }
                    Instruction::Walk(count) => {
                        for _ in 0..count {
                            position = match direction {
                                Direction::North => Point { y: position.y + 1, ..position },
                                Direction::East => Point { x: position.x + 1, ..position },
                                Direction::South => Point { y: position.y - 1, ..position },
                                Direction::West => Point { x: position.x - 1, ..position },
                            };
                            if first_position_visited_twice.is_none() && !visited.insert(position) {
                                first_position_visited_twice = Some(position);
                            }
                        }
                    }
                }
            }
            (position, first_position_visited_twice)
        }

        /// Borrow a reference to the Traveler current position.
        pub fn position(&self) -> &Point {
            &self.position
        }
    }
}


use no_time_for_a_taxicab::*;

fn main() {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("no input given");
    let document: RecruitingDocument = input.parse().expect("bad input");
    let me = Traveler::airdrop_at(*document.starting_point());
    let easter_bunny_hq_positions = me.follow(&document);
    println!("Easter Bunny Headquarters distance: {}",
             easter_bunny_hq_positions.0.snake_distance(me.position()));
    if let Some(real_hq_position) = easter_bunny_hq_positions.1 {
        println!("Easter Bunny Headquarters distance (after careful read): {}",
                 real_hq_position.snake_distance(me.position()));
    }
}


#[test]
fn part1_first_example() {
    let document: RecruitingDocument = "R2, L3".parse().unwrap();
    let me = Traveler::airdrop_at(*document.starting_point());
    assert_eq!(me.follow(&document).0.snake_distance(me.position()), 5);
}

#[test]
fn part1_second_example() {
    let document: RecruitingDocument = "R2, R2, R2".parse().unwrap();
    let me = Traveler::airdrop_at(*document.starting_point());
    assert_eq!(me.follow(&document).0.snake_distance(me.position()), 2);
}

#[test]
fn part1_third_example() {
    let document: RecruitingDocument = "R5, L5, R5, R3".parse().unwrap();
    let me = Traveler::airdrop_at(*document.starting_point());
    assert_eq!(me.follow(&document).0.snake_distance(me.position()), 12);
}

#[test]
fn part2_single_example() {
    let document: RecruitingDocument = "R8, R4, R4, R8".parse().unwrap();
    let me = Traveler::airdrop_at(*document.starting_point());
    assert_eq!(me.follow(&document).1.unwrap().snake_distance(&me.position()), 4);
}
