use std::io::Read;


#[derive(Copy, Clone, Debug)]
enum Move {
    Up,
    Right,
    Down,
    Left,
    Push,
}

#[derive(Debug)]
struct BathroomDocument {
    starting_button: u8,
    instructions: Vec<Move>,
}

#[derive(Debug)]
struct Keypad9 {
    button: u8,
    pressed: Vec<u8>,
}

struct BathroomCodeResolver {
}


impl BathroomDocument {
    fn parse(input: &str) -> Option<BathroomDocument> {
        let mut instructions = Vec::new();
        for line in input.lines() {
            for c in line.chars() {
                let optmv = match c {
                    'U' => Some(Move::Up),
                    'R' => Some(Move::Right),
                    'D' => Some(Move::Down),
                    'L' => Some(Move::Left),
                    _ => None,
                };
                if let Some(mv) = optmv {
                    instructions.push(mv);
                } else {
                    return None;
                }
            }
            instructions.push(Move::Push);
        }
        Some(BathroomDocument {
            starting_button: 5,
            instructions: instructions,
        })
    }
}

impl Keypad9 {
    fn new(starting_button: u8) -> Option<Keypad9> {
        let (min, max) = (1, 9);
        if starting_button < min || starting_button > max {
            None
        } else {
            Some(Keypad9 {
                button: starting_button,
                pressed: Vec::new(),
            })
        }
    }
    fn perform(&mut self, mv: Move) -> () {
        match mv {
            Move::Up if self.button > 3 => self.button -= 3,
            Move::Down if self.button <= (3 * (3 - 1)) => self.button += 3,
            Move::Right if (self.button % 3) != 0 => self.button += 1,
            Move::Left if (self.button % 3) != 1 => self.button -= 1,
            Move::Push => self.pressed.push(self.button),
            _ => (),
        }
    }
}

impl BathroomCodeResolver {
    fn find_code(document: &BathroomDocument) -> String {
        let mut keypad = Keypad9::new(document.starting_button).expect("bad bathroom document");
        for mv in &document.instructions {
            keypad.perform(*mv);
        }
        keypad.pressed.iter().map(|i| i.to_string()).collect::<Vec<String>>().join("")
    }
}


fn main() {
    let mut input = String::new();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut input).expect("no input given");
    let document = BathroomDocument::parse(&input).expect("bad bathroom document");
    let code = BathroomCodeResolver::find_code(&document);
    println!("the bathroom code is {}", code);
}


#[test]
fn example() {
    let document = BathroomDocument::parse("ULL\nRRDDD\nLURDL\nUUUUD").unwrap();
    let code = BathroomCodeResolver::find_code(&document);
    assert_eq!(code, "1985");
}
