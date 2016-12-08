mod bathroom_security {
    use std::str::FromStr;
    use std::collections::{HashMap, HashSet};

    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[derive(Copy, Clone, Debug)]
    enum Direction {
        Up,
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
        buttons: HashMap<Point, KeypadButton>,
        current_button_position: Option<Point>,
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


    impl FromStr for Direction {
        type Err = String;

        fn from_str(s: &str) -> Result<Direction, String> {
            match s {
                "U" => Ok(Direction::Up),
                "R" => Ok(Direction::Right),
                "D" => Ok(Direction::Down),
                "L" => Ok(Direction::Left),
                _ => Err(format!("{}: unrecognized direction", s))
            }
        }
    }

    impl FromStr for Keypad {
        type Err = String;

        fn from_str(s: &str) -> Result<Keypad, String> {
            let mut unique: HashSet<KeypadButton> = HashSet::new();
            let mut buttons: HashMap<Point, KeypadButton> = HashMap::new();
            for (y, line) in s.lines().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    if c == ' ' {
                        continue;
                    }
                    let button = KeypadButton(c);
                    if !unique.insert(button) {
                        return Err(format!("{:?}: already exist", button));
                    }
                    let position = Point { x: x as i32, y: y as i32 };
                    buttons.insert(position, button);
                }
            }
            Ok(Keypad {
                buttons: buttons,
                current_button_position: None,
                pressed: Vec::new(),
            })
        }
    }

    impl Keypad {
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
                            Direction::Up => Point { y: position.y - 1, ..position },
                            Direction::Right => Point { x: position.x + 1, ..position },
                            Direction::Down => Point { y: position.y + 1, ..position },
                            Direction::Left => Point { x: position.x - 1, ..position },
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

    impl FromStr for BathroomDocument {
        type Err = String;

        fn from_str(s: &str) -> Result<BathroomDocument, String> {
            let mut instructions = Vec::new();
            for line in s.lines() {
                for i in 0..line.len() {
                    let direction: Direction = try!(line[i..i + 1].parse());
                    instructions.push(KeypadAction::Move(direction));
                }
                instructions.push(KeypadAction::Press);
            }
            Ok(BathroomDocument {
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

fn expected_bathroom_keypad() -> Keypad {
"
123
456
789
".parse().unwrap()
}

fn bathroom_actual_keypad() -> Keypad {
"
  1
 234
56789
 ABC
  D
".parse().unwrap()
}

fn main() {
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");
    let document: BathroomDocument = input.parse().unwrap();
    let mut keypad = expected_bathroom_keypad();
    let code = BathroomCodeResolver::find_code(&document, &mut keypad).unwrap();
    println!("the bathroom code is {}", code);
    let mut keypad = bathroom_actual_keypad();
    let code = BathroomCodeResolver::find_code(&document, &mut keypad).unwrap();
    println!("wait no actually the bathroom code is {}", code);
}


#[test]
fn part1_example() {
    let document: BathroomDocument = "ULL\nRRDDD\nLURDL\nUUUUD".parse().unwrap();
    let mut keypad = expected_bathroom_keypad();
    let code = BathroomCodeResolver::find_code(&document, &mut keypad);
    assert_eq!(code, Ok("1985".to_string()));
}

#[test]
fn part2_example() {
    let document: BathroomDocument = "ULL\nRRDDD\nLURDL\nUUUUD".parse().unwrap();
    let mut keypad = bathroom_actual_keypad();
    let code = BathroomCodeResolver::find_code(&document, &mut keypad);
    assert_eq!(code, Ok("5DB3".to_string()));
}
