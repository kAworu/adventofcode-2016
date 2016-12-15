mod bathroom_security {
    use ::std::collections::HashMap;
    use ::std::fmt::Display;
    use ::std::ops::{Deref, DerefMut};
    use ::std::str::FromStr;

    /// Represent a position on the keypad.
    ///
    /// the 0,0 Point on the keypad is the very top-left corner.
    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    /// Represent a direction on they keypad.
    #[derive(Copy, Clone, Debug)]
    enum Direction {
        Up,
        Right,
        Down,
        Left,
    }

    // NOTE: don't impl From<char> because it can not fail, TryFrom not ready yet.
    impl FromStr for Direction {
        type Err = String;

        /// Parse a string into a `Direction`.
        ///
        /// Expect `s` to be either "U", "R", "D" or "L".
        fn from_str(s: &str) -> Result<Direction, String> {
            match s {
                "U" => Ok(Direction::Up),
                "R" => Ok(Direction::Right),
                "D" => Ok(Direction::Down),
                "L" => Ok(Direction::Left),
                _ => Err(format!("{}: unrecognized direction", s)),
            }
        }
    }

    /// Represent a keypad button, storing its "label" as `char`.
    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    pub struct KeypadButton(char);

    impl Deref for KeypadButton {
        type Target = char;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    /// Represent an input sequence of `KeypadButton`
    ///
    /// Newtype'd so we can to_string() and impl Deref and DerefMut to the underlying Vec.
    #[derive(Debug)]
    pub struct KeypadButtonSequence(Vec<KeypadButton>);

    impl Deref for KeypadButtonSequence {
        type Target = Vec<KeypadButton>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for KeypadButtonSequence {
        fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target {
            &mut self.0
        }
    }

    impl Display for KeypadButtonSequence {
        /// Basically join each `KeypadButton` characters in self into a `String`.
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            let s: String = self.iter().map(|&button| *button).collect();
            write!(f, "{}", s)
        }
    }

    /// Represent a bathroom Keypad.
    #[derive(Debug)]
    pub struct Keypad {
        // NOTE: Point { x: 0, y: 0 } on the keypad is the top-left corner.
        positions_to_buttons: HashMap<Point, KeypadButton>,
        buttons_to_positions: HashMap<KeypadButton, Point>,
        pressed: KeypadButtonSequence,
    }

    impl Keypad {
        /// Returns true if the given `KeypadButton` exist in self, false otherwise.
        fn has_button(&self, button: KeypadButton) -> bool {
            self.buttons_to_positions.contains_key(&button)
        }

        /// Find the button near the given target KeypadButton.
        ///
        /// Returns None if target is not in self or there is no button in the given `Direction`
        /// from target, `Some` button otherwise.
        fn neighbour_of(&self, target: KeypadButton, direction: Direction) -> Option<KeypadButton> {
            self.buttons_to_positions.get(&target).and_then(|&position| {
                let next_position = match direction {
                    Direction::Up => Point { y: position.y - 1, ..position },
                    Direction::Right => Point { x: position.x + 1, ..position },
                    Direction::Down => Point { y: position.y + 1, ..position },
                    Direction::Left => Point { x: position.x - 1, ..position },
                };
                self.positions_to_buttons.get(&next_position).and_then(|&button| Some(button))
            })
        }

        /// Press the given `KeypadButton` on self.
        ///
        /// Returns true if the button could be pressed, false otherwise (the button doesn't
        /// belongs in self).
        fn press(&mut self, target: KeypadButton) -> bool {
            if !self.has_button(target) {
                return false;
            } else {
                self.pressed.push(target);
                true
            }
        }

        /// Borrow a reference to the `Keypad` pressed buttons.
        pub fn input_sequence(&self) -> &KeypadButtonSequence {
            &self.pressed
        }
    }

    impl FromStr for Keypad {
        type Err = String;

        /// Parse a string into a `Keypad`.
        ///
        /// Expect `s` to be a keypad grid representation where ASCII spaces (0x20) are skipped
        /// (but not ignored) zones of the size of a button and all other characters are buttons.
        /// All non-space characters must be unique through the representation.
        ///
        /// # Examples
        ///
        /// A classic keypad (with buttons from 1 to 9 as any sane person would picture)
        /// representation look like this:
        ///
        /// ```text
        /// 123
        /// 456
        /// 789
        /// ```
        ///
        /// A keypad from hell resulting of hundreds of man-hours of bathroom-keypad-design
        /// meetings representation look like this:
        ///
        /// ```text
        ///   1
        ///  234
        /// 56789
        ///  ABC
        ///   D
        /// ```
        fn from_str(s: &str) -> Result<Keypad, String> {
            let mut buttons_to_positions: HashMap<KeypadButton, Point> = HashMap::new();
            let mut positions_to_buttons: HashMap<Point, KeypadButton> = HashMap::new();
            for (y, line) in s.lines().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    // skip if we're on a blank space, it is a non-button position.
                    if c == ' ' {
                        continue;
                    }
                    // NOTE: we want to be able to create `Point` that are beyond the keyboard grid
                    // (off-by-one, see neighbour_of()), hence checking for (x + 1) and (y + 1).
                    if x + 1 > ::std::i32::MAX as usize || y + 1 > ::std::i32::MAX as usize {
                        return Err("insanely big keyboard string representation".to_string());
                    }
                    // (x as i32) and (y as i32) are safe now that we checked against
                    // std::i32::MAX.
                    let position = Point {
                        x: x as i32,
                        y: y as i32,
                    };
                    let button = KeypadButton(c);
                    if buttons_to_positions.insert(button, position).is_some() {
                        return Err(format!("{:?}: already exist", button));
                    }
                    positions_to_buttons.insert(position, button);
                }
            }
            Ok(Keypad {
                positions_to_buttons: positions_to_buttons,
                buttons_to_positions: buttons_to_positions,
                pressed: KeypadButtonSequence(Vec::new()),
            })
        }
    }

    /// Represent an action that can be performed on a keypad.
    #[derive(Copy, Clone, Debug)]
    enum KeypadAction {
        Move(Direction),
        Press,
    }

    /// Represent a bathroom code document found in Easter Bunny Headquarters.
    #[derive(Debug)]
    pub struct BathroomDocument {
        initial_button: KeypadButton,
        instructions: Vec<KeypadAction>,
    }

    impl FromStr for BathroomDocument {
        type Err = String;

        /// Parse a string into a `BathroomDocument`.
        ///
        /// Expect each line from `s` to match `/[URDL]*/`. Only the instructions are parsed, the
        /// starting button is always '5'.
        fn from_str(s: &str) -> Result<BathroomDocument, String> {
            let mut instructions = Vec::new();
            for line in s.lines() {
                // NOTE: loop through the line characters index and not .chars() so we can slice
                // it, because `Direction` are parsed `FromStr`.
                for i in 0..line.len() {
                    let direction: Direction = line[i..i + 1].parse()?;
                    instructions.push(KeypadAction::Move(direction));
                }
                instructions.push(KeypadAction::Press);
            }
            Ok(BathroomDocument {
                initial_button: KeypadButton('5'),
                instructions: instructions,
            })
        }
    }

    /// Represent someone (or something) able to follow the Bathroom Document instructions.
    #[derive(Debug)]
    pub struct Finger<'a> {
        keypad: &'a mut Keypad,
        hovering: KeypadButton,
    }

    impl<'a> Finger<'a> {
        /// Create a new `Finger` hovering the given button on the provided `Keypad`.
        ///
        /// Returns `None` if `button` doesn't exist in the keypad, `Some` new `Finger` object
        /// otherwise.
        fn new(keypad: &'a mut Keypad, button: KeypadButton) -> Option<Finger> {
            if !keypad.has_button(button) {
                return None;
            }
            Some(Finger {
                keypad: keypad,
                hovering: button,
            })
        }

        /// Follow every instructions from the `BathroomDocument` on the given `Keypad`.
        pub fn follow(document: &BathroomDocument, keypad: &'a mut Keypad) {
            if let Some(mut finger) = Finger::new(keypad, document.initial_button) {
                for &action in &document.instructions {
                    finger.perform(action);
                }
            }
        }

        /// Perform the given `KeypadAction` on our keypad.
        ///
        /// Returns the hovering button after the action has resolved.
        fn perform(&mut self, action: KeypadAction) {
            match action {
                KeypadAction::Press => {
                    if !self.keypad.press(self.hovering) {
                        // NOTE: if self.hovering is not in the keypad it is a Finger impl bug.
                        panic!("buggy hovering button handling in Finger");
                    }
                }
                KeypadAction::Move(direction) => {
                    let neighbour = self.keypad.neighbour_of(self.hovering, direction);
                    if let Some(button) = neighbour {
                        self.hovering = button;
                    }
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
"
        .parse()
        .unwrap()
}

fn actual_bathroom_keypad() -> Keypad {
    "
  1
 234
56789
 ABC
  D
"
        .parse()
        .unwrap()
}

fn main() {
    // acquire data from stdin
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");

    // parse the provided document instructions
    let document: BathroomDocument = input.parse().unwrap();

    let mut keypad = expected_bathroom_keypad();
    Finger::follow(&document, &mut keypad);
    println!("the bathroom code is {}",
             keypad.input_sequence().to_string());

    let mut keypad = actual_bathroom_keypad();
    Finger::follow(&document, &mut keypad);
    println!("wait no actually the bathroom code is {}",
             keypad.input_sequence().to_string());
}


#[test]
fn part1_example() {
    let document: BathroomDocument = "ULL\nRRDDD\nLURDL\nUUUUD".parse().unwrap();
    let mut keypad = expected_bathroom_keypad();
    Finger::follow(&document, &mut keypad);
    assert_eq!(keypad.input_sequence().to_string(), "1985".to_string());
}

#[test]
fn part2_example() {
    let document: BathroomDocument = "ULL\nRRDDD\nLURDL\nUUUUD".parse().unwrap();
    let mut keypad = actual_bathroom_keypad();
    Finger::follow(&document, &mut keypad);
    assert_eq!(keypad.input_sequence().to_string(), "5DB3".to_string());
}
