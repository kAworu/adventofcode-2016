extern crate regex;
extern crate rand;

use std::collections::HashSet;
use rand::Rng;
use regex::Regex;

#[derive(Copy, Clone, Debug)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone, Debug)]
enum Instruction {
    TurnRight,
    TurnLeft,
    Walk(i32),
}

#[derive(Debug)]
struct RecruitingDocument {
    starting_position: Position,
    initial_direction: Direction,
    instructions: Vec<Instruction>,
}

#[derive(Debug)]
struct Traveler {
    position: Position,
}


impl Position {
    fn random() -> Position {
        let mut rng = rand::thread_rng();
        Position {
            x: rng.gen::<i16>() as i32,
            y: rng.gen::<i16>() as i32,
        }
    }
    fn snake_distance(&self, other: &Self) -> u32 {
        (self.x - other.x).abs() as u32 + (self.y - other.y).abs() as u32
    }
}

impl RecruitingDocument {
    fn parse(input: &str) -> RecruitingDocument {
        let mut instructions = Vec::new();
        let re = Regex::new(r"(R|L)(\d+)").unwrap();
        for cap in re.captures_iter(input) {
            instructions.push(if cap.at(1).unwrap() == "R" {
                Instruction::TurnRight
            } else {
                Instruction::TurnLeft
            });
            let count: i32 = cap.at(2).unwrap().parse().unwrap();
            instructions.push(Instruction::Walk(count));
        }
        RecruitingDocument {
            starting_position: Position::random(),
            initial_direction: Direction::North,
            instructions: instructions,
        }
    }
}

impl Traveler {
    fn airdrop_at(landing_position: Position) -> Traveler {
        Traveler { position: landing_position }
    }
    fn follow_all_pages(&self, document: &RecruitingDocument) -> Option<Position> {
        let (mut position, mut direction) = (self.position, document.initial_direction);
        let mut visited = HashSet::new();
        visited.insert(position);
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
                            Direction::North => Position { y: position.y + 1, ..position },
                            Direction::East => Position { x: position.x + 1, ..position },
                            Direction::South => Position { y: position.y - 1, ..position },
                            Direction::West => Position { x: position.x - 1, ..position },
                        };
                        if !visited.insert(position) {
                            // we've been here before
                            return Some(position);
                        }
                    }
                }
            }
        }
        None
    }
}


fn main() {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("no input given");
    let document = RecruitingDocument::parse(&input);
    let me = Traveler::airdrop_at(document.starting_position);
    let easter_bunny_hq = me.follow_all_pages(&document).expect("forged recruiting document!");
    println!("Easter Bunny Headquarters distance: {}",
             easter_bunny_hq.snake_distance(&me.position));
}


#[test]
fn example() {
    let document = RecruitingDocument::parse("R8, R4, R4, R8");
    let me = Traveler::airdrop_at(document.starting_position);
    assert_eq!(me.follow_all_pages(&document).map(|pos| pos.snake_distance(&me.position)),
               Some(4));
}
