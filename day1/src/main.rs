extern crate rand;

mod no_time_for_a_taxicab {
    use std::str::FromStr;
    use ::std::collections::HashSet;
    use ::rand::Rng;

    #[derive(Copy, Clone, Debug)]
    enum Direction {
        North,
        East,
        South,
        West,
    }

    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    pub struct Point {
        x: i32,
        y: i32,
    }

    #[derive(Copy, Clone, Debug)]
    enum Instruction {
        TurnRight,
        TurnLeft,
        Walk(i32), // NOTE: i32 allow us walk backward
    }

    #[derive(Debug)]
    pub struct RecruitingDocument {
        starting_position: Point,
        initial_direction: Direction,
        instructions: Vec<Instruction>,
    }

    #[derive(Debug)]
    pub struct Traveler {
        position: Point,
    }


    impl Point {
        pub fn random() -> Point {
            let mut rng = ::rand::thread_rng();
            Point {
                x: rng.gen::<i16>() as i32,
                y: rng.gen::<i16>() as i32,
            }
        }
        pub fn snake_distance(&self, other: &Self) -> u32 {
            (self.x - other.x).abs() as u32 + (self.y - other.y).abs() as u32
        }
    }

    impl FromStr for Instruction {
        type Err = String;

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

    impl RecruitingDocument {
        pub fn starting_position(&self) -> &Point {
            &self.starting_position
        }
    }

    impl FromStr for RecruitingDocument {
        type Err = String;

        fn from_str(s: &str) -> Result<RecruitingDocument, String> {
            let tokens: Vec<&str> = s.split(',').map(|s| s.trim()).collect();
            let mut instructions = Vec::new();
            for token in tokens.iter() {
                if token.len() < 2 {
                    return Err(format!("{}: unrecognized instruction", token));
                }
                let direction: Instruction = try!(token[..1].parse());
                let stepcount: Instruction = try!(token[1..].parse());
                instructions.push(direction);
                instructions.push(stepcount);
            }
            Ok(RecruitingDocument {
                starting_position: Point::random(),
                initial_direction: Direction::North,
                instructions: instructions,
            })
        }
    }

    impl Traveler {
        pub fn airdrop_at(landing_position: Point) -> Traveler {
            Traveler { position: landing_position }
        }
        pub fn follow(&self, document: &RecruitingDocument) -> (Point, Option<Point>) {
            let (mut position, mut direction) = (self.position, document.initial_direction);
            let mut visited = HashSet::new();
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
                            if !visited.insert(position) && first_position_visited_twice.is_none() {
                                // we've been here before
                                first_position_visited_twice = Some(position);
                            }
                        }
                    }
                }
            }
            (position, first_position_visited_twice)
        }
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
    let me = Traveler::airdrop_at(*document.starting_position());
    let easter_bunny_hq_positions = me.follow(&document);
    println!("Easter Bunny Headquarters distance: {}",
             easter_bunny_hq_positions.0.snake_distance(me.position()));
    if let Some(position) = easter_bunny_hq_positions.1 {
        println!("Easter Bunny Headquarters distance (after careful read): {}",
                 position.snake_distance(me.position()));
    }
}


#[test]
fn part1_first_example() {
    let document: RecruitingDocument = "R2, L3".parse().unwrap();
    let me = Traveler::airdrop_at(*document.starting_position());
    assert_eq!(me.follow(&document).0.snake_distance(me.position()), 5);
}

#[test]
fn part1_second_example() {
    let document: RecruitingDocument = "R2, R2, R2".parse().unwrap();
    let me = Traveler::airdrop_at(*document.starting_position());
    assert_eq!(me.follow(&document).0.snake_distance(me.position()), 2);
}

#[test]
fn part1_third_example() {
    let document: RecruitingDocument = "R5, L5, R5, R3".parse().unwrap();
    let me = Traveler::airdrop_at(*document.starting_position());
    assert_eq!(me.follow(&document).0.snake_distance(me.position()), 12);
}

#[test]
fn part2_single_example() {
    let document: RecruitingDocument = "R8, R4, R4, R8".parse().unwrap();
    let me = Traveler::airdrop_at(*document.starting_position());
    assert_eq!(me.follow(&document).1.unwrap().snake_distance(&me.position()), 4);
}
