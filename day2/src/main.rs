mod bathroom_security {
    use std::collections::{HashMap, HashSet};

    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    struct Position {
        x: i32,
        y: i32,
    }

    #[derive(Copy, Clone, Debug)]
    enum Direction {
        Up = 0,
        Right,
        Down,
        Left,
    }

    #[derive(Copy, Clone, Debug)]
    enum KeypadAction {
        Move(Direction),
        Press,
    }

    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    struct KeypadButton(char);

    #[derive(Debug)]
    pub struct Keypad {
        buttons: HashMap<Position, KeypadButton>,
        current_button_position: Option<Position>,
        pressed: Vec<KeypadButton>,
    }

    pub enum KeypadError {
        ButtonNotFound,
    }

    #[derive(Debug)]
    pub struct BathroomDocument {
        start: KeypadButton,
        instructions: Vec<KeypadAction>,
    }

    pub struct BathroomCodeResolver {
    }


    impl Keypad {
        pub fn new(desc: &str) -> Option<Keypad> {
            let mut unique: HashSet<KeypadButton> = HashSet::new();
            let mut buttons: HashMap<Position, KeypadButton> = HashMap::new();
            for (y, line) in desc.lines().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    if c == ' ' {
                        continue;
                    }
                    let button = KeypadButton(c);
                    if !unique.insert(button) {
                        return None;
                    }
                    let position = Position { x: x as i32, y: y as i32 };
                    buttons.insert(position, button);
                }
            }
            Some(Keypad {
                buttons: buttons,
                current_button_position: None,
                pressed: Vec::new(),
            })
        }
        fn current_button_is(&mut self, target: KeypadButton) -> Result<(), KeypadError> {
            for (position, button) in self.buttons.iter() {
                if *button == target {
                    self.current_button_position = Some(*position);
                    return Ok(());
                }
            }
            Err(KeypadError::ButtonNotFound)
        }
        // noop if current_button_position is None
        fn perform(&mut self, action: KeypadAction) {
            if let Some(position) = self.current_button_position {
                match action {
                    KeypadAction::Press => {
                        let to_press = self.buttons.get(&position).unwrap();
                        self.pressed.push(*to_press);
                    }
                    KeypadAction::Move(direction) => {
                        let next_position = match direction {
                            Direction::Up => Position { y: position.y - 1, ..position },
                            Direction::Right => Position { x: position.x + 1, ..position },
                            Direction::Down => Position { y: position.y + 1, ..position },
                            Direction::Left => Position { x: position.x - 1, ..position },
                        };
                        if self.buttons.contains_key(&next_position) {
                            self.current_button_position = Some(next_position);
                        }
                    }
                }
            }
        }
        fn input_sequence(&self) -> &Vec<KeypadButton> {
            &self.pressed
        }
    }

    impl BathroomDocument {
        pub fn parse(input: &str) -> Option<BathroomDocument> {
            let mut instructions = Vec::new();
            for line in input.lines() {
                for c in line.chars() {
                    instructions.push(match c {
                        'U' => KeypadAction::Move(Direction::Up),
                        'R' => KeypadAction::Move(Direction::Right),
                        'D' => KeypadAction::Move(Direction::Down),
                        'L' => KeypadAction::Move(Direction::Left),
                        _ => return None,
                    });
                }
                instructions.push(KeypadAction::Press);
            }
            Some(BathroomDocument {
                start: KeypadButton('5'),
                instructions: instructions,
            })
        }
    }

    impl BathroomCodeResolver {
        pub fn find_code(document: &BathroomDocument,
                         keypad: &mut Keypad)
                         -> Result<String, String> {
            match keypad.current_button_is(document.start) {
                Err(KeypadError::ButtonNotFound) => {
                    Err("document starting button doesn't exist in the keypad".to_string())
                }
                Ok(_) => {
                    for action in &document.instructions {
                        keypad.perform(*action);
                    }
                    let seq = keypad.input_sequence();
                    let code = seq.iter()
                        .map(|button| {
                            let KeypadButton(c) = *button;
                            c
                        })
                        .collect();
                    Ok(code)
                }
            }
        }
    }
}


use std::io::Read;
use bathroom_security::*;

fn main() {
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");
    let document = BathroomDocument::parse(&input).expect("bad bathroom document");
    let mut keypad = Keypad::new("
123
456
789
").unwrap();
    let code = BathroomCodeResolver::find_code(&document, &mut keypad).unwrap();
    println!("the bathroom code is {}", code);
    let mut keypad = Keypad::new("
  1
 234
56789
 ABC
  D
").unwrap();
    let code = BathroomCodeResolver::find_code(&document, &mut keypad).unwrap();
    println!("wait no actually the bathroom code is {}", code);
}


#[test]
fn part1_example() {
    let document = BathroomDocument::parse("ULL\nRRDDD\nLURDL\nUUUUD").unwrap();
    let mut keypad = Keypad::new("
123
456
789
").unwrap();
    let code = BathroomCodeResolver::find_code(&document, &mut keypad);
    assert_eq!(code, Ok("1985".to_string()));
}

#[test]
fn part2_example() {
    let document = BathroomDocument::parse("ULL\nRRDDD\nLURDL\nUUUUD").unwrap();
    let mut keypad = Keypad::new("
  1
 234
56789
 ABC
  D
").unwrap();
    let code = BathroomCodeResolver::find_code(&document, &mut keypad);
    assert_eq!(code, Ok("5DB3".to_string()));
}
