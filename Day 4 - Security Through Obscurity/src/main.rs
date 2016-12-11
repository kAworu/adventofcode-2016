mod security_through_obscurity {
    use ::std::collections::HashMap;
    use ::std::fmt::Display;
    use ::std::str::FromStr;

    // some Room parsing / filtering related helpers

    /// Returns true if the given character is a dash (0x2d), false otherwise.
    fn is_dash(ch: char) -> bool {
        ch == '-'
    }

    /// Returns true if the given character is a letter as defined by the puzzle — i.e. matching
    /// [a-z], false otherwise.
    fn is_ascii_lower(ch: char) -> bool {
        // XXX: unstable see issue #32311
        // ('a'..'z').contains(ch);
        ch >= 'a' && ch <= 'z'
    }

    /// Returns true if the given character is numeric as defined by the puzzle — i.e. matching
    /// [0-9], false otherwise.
    fn is_ascii_digit(ch: char) -> bool {
        // XXX: unstable see issue #32311
        // ('0'..'9').contains(ch);
        ch >= '0' && ch <= '9'
    }

    /// Returns true if the given character is a left square bracket (0x5b), false otherwise.
    fn is_left_square_bracket(ch: char) -> bool {
        ch == '['
    }

    /// Returns true if the given character is a right square bracket (0x5d), false otherwise.
    fn is_right_square_bracket(ch: char) -> bool {
        ch == ']'
    }

    /// Represent a `Room` encrypted name, implement the decryption and checksum logic.
    #[derive(Debug)]
    struct RoomEncryptedName(String);

    impl RoomEncryptedName {
        /// Compute the checksum according to the puzzle definition.
        ///
        /// > [...] the checksum is the five most common letters in the encrypted name, in order,
        /// > with ties broken by alphabetization.
        fn checksum(&self) -> String {
            // compute the frequency for each letter characters in our encrypted_name.
            let mut char_to_freq = HashMap::new();
            for ch in self.0.chars().filter(|&ch| is_ascii_lower(ch)) {
                *char_to_freq.entry(ch).or_insert(0) += 1;
            }
            // build a vector of tuple (char, frequency) from the hash (key, value) so we can sort
            // our results.
            let mut vec: Vec<_> = char_to_freq.into_iter().collect();
            vec.sort_by(|&(cha, freqa), &(chb, freqb)| {
                // compare by the frequency (value) in the descending order (i.e. the most frequent
                // first), hence "b cmp a".
                match freqb.cmp(&freqa) {
                    // if a and b have the same frequency: "fallback" to the alphabetic
                    // (ascending) order, hence "a cmp b" this time.
                    ::std::cmp::Ordering::Equal => cha.cmp(&chb),
                    less_or_greater             => less_or_greater,
                }
            });

            vec.into_iter()
                .map(|(ch, _)| ch) // map to the char, we don't need the frequency anymore
                .take(5) // the checksum is *the five* most common letters
                .collect()
        }

        /// Decrypt self using the given key.
        ///
        /// Returns a decrypted representation of self.
        // NOTE: Only dash and lower letters will be decrypted, other characters will be replaced
        // by `?` (i.e. 0x3f). The puzzle `Room` encrypted names only contains dash and lower
        // letters but this invariant is enforced at the `Room` level.
        fn decrypt(&self, key: u32) -> String {
            // NOTE: % is the reminder operator in Rust, no modulus operator in the stdlib.
            let mod26 = |x| (x % 26) as u8;
            let char_to_enc = |ch| ch as u32 - 'a' as u32;
            let dec_to_char = |dec| char::from('a' as u8 + dec);
            let shift = mod26(key) as u32; // as u32 because we'll use it as mod26() input
            self.0.chars().map(|ch| {
                if is_dash(ch) {
                    ' '
                } else if is_ascii_lower(ch) {
                    let enc = char_to_enc(ch);
                    let dec = mod26(enc + shift);
                    dec_to_char(dec)
                } else { // unexpected
                    '?'
                }
            }).collect()
        }
    }

    /// Represent a room from the list at the information kiosk
    #[derive(Debug)]
    pub struct Room {
        encrypted_name: RoomEncryptedName,
        sector_id: u32,
        checksum: String,
    }

    impl Room {
        /// Returns true if a room is real (i.e. if its checksum is correct), false otherwise.
        pub fn is_real(&self) -> bool {
            self.encrypted_name.checksum() == self.checksum
        }

        /// Returns true if a room is not real (i.e. if its checksum is incorrect), false
        /// otherwise.
        pub fn is_decoy(&self) -> bool {
            !self.is_real()
        }

        /// Returns the `Room` sector_id.
        pub fn sector_id(&self) -> u32 {
            self.sector_id
        }

        /// Returns the decrypted `Room` name.
        pub fn name(&self) -> String {
            self.encrypted_name.decrypt(self.sector_id)
        }
    }

    impl FromStr for Room {
        type Err = String;

        /// Parse a string into a `Room`.
        ///
        /// > Each room consists of an encrypted name (lowercase letters separated by dashes) >
        /// followed by a dash, a sector ID, and a checksum in square brackets.
        ///
        /// # Examples
        ///
        /// `aaaaa-bbb-z-y-x-123[abxyz]`
        /// `a-b-c-d-e-f-g-h-987[abcde]`
        /// `not-a-real-room-404[oarel]`
        /// `totally-real-room-200[decoy]`
        // We could just /^([a-z]+(?:-[a-z]+)*)-(\d+)\[[a-z]+\]$/ but meh
        fn from_str(s: &str) -> Result<Room, String> {
            let parse_error_for = |part, x| {
                match x {
                    Some(ch) => Err(format!("unexpected `{}` while parsing {}", ch, part)),
                    None     => Err(format!("parsing {} failed", part)),
                }
            };
            let mut iter = s.chars().peekable();
            let mut encrypted_name = String::with_capacity(s.len());
            let mut sector_id      = String::with_capacity(s.len());
            let mut checksum       = String::with_capacity(s.len());
            // parse the encrypted name
            loop {
                match iter.next() {
                    Some(ch) if is_ascii_lower(ch) => encrypted_name.push(ch),
                    Some(ch) if is_dash(ch) => match iter.peek() {
                        // we don't accept encrypted name beginning with a dash
                        _ if encrypted_name.len() == 0 => return parse_error_for("encrypted name", Some(ch)),
                        // if the next character is numeric then this dash (ch) is the delimiter
                        // between the encrypted name and sector ID.
                        Some(&next) if is_ascii_digit(next) => break,
                        // the encrypted name may contains dash but then we require the next
                        // character to be a letter
                        Some(&next) if is_ascii_lower(next) => encrypted_name.push(ch),
                        // this is unexpected, but we'll handle it at the next iteration.
                        _ => continue,
                    },
                    x => return parse_error_for("encrypted name", x)
                }
            }
            // parse the sector ID
            loop {
                match iter.next() {
                    Some(ch) if is_ascii_digit(ch) => sector_id.push(ch),
                    Some(ch) if is_left_square_bracket(ch) => break,
                    x => return parse_error_for("sector ID", x)
                }
            }
            // parse the checksum
            loop {
                match iter.next() {
                    Some(ch) if is_ascii_lower(ch) => checksum.push(ch),
                    Some(ch) if is_right_square_bracket(ch) => break,
                    x => return parse_error_for("checksum", x)
                }
            }
            // we're done parsing, don't allow the input to have more characters.
            if iter.peek().is_some() {
                return parse_error_for("room", iter.next());
            }
            Ok(Room {
                encrypted_name: RoomEncryptedName(encrypted_name),
                sector_id: sector_id.parse().unwrap(),
                checksum: checksum,
            })
        }
    }

    impl Display for Room {
        /// Reconstruct a string from `Room`
        ///
        /// see from_str() for the format.
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "{}-{}[{}]", self.encrypted_name.0, self.sector_id, self.checksum)
        }
    }

}


use std::io::Read;
use security_through_obscurity::*;

fn main() {
    // acquire data from stdin
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");

    // parse all the rooms, one per line of input.
    let mut rooms = Vec::new();
    for line in input.lines() {
        let room: Room = line.parse().expect("bad input");
        rooms.push(room);
    }

    let real_rooms: Vec<_> = rooms.iter().filter(|&r| r.is_real()).collect();
    // compute the sum of the real room's sector ID
    let sum: u32 = real_rooms.iter().map(|r| r.sector_id()).sum();
    println!("The sum of the sector IDs of the real rooms is {}", sum);

    // find the target!
    for room in &real_rooms {
        let name = room.name();
        if name.contains("northpole") && name.contains("storage") {
            println!("{} #{}", room.name(), room.sector_id());
        }
    }
}


#[test]
fn part1_first_example() {
    let room: Room = "aaaaa-bbb-z-y-x-123[abxyz]".parse().unwrap();
    println!("{:?}", room);
    assert!(room.is_real());
}

#[test]
fn part1_second_example() {
    let room: Room = "a-b-c-d-e-f-g-h-987[abcde]".parse().unwrap();
    println!("{:?}", room);
    assert!(room.is_real());
}

#[test]
fn part1_third_example() {
    let room: Room = "not-a-real-room-404[oarel]".parse().unwrap();
    println!("{:?}", room);
    assert!(room.is_real());
}

#[test]
fn part1_fourth_example() {
    let room: Room = "totally-real-room-200[decoy]".parse().unwrap();
    println!("{:?}", room);
    assert!(room.is_decoy());
}

#[test]
fn part2_example() {
    let room: Room = "qzmt-zixmtkozy-ivhz-343[incomplete]".parse().unwrap();
    assert_eq!(room.name(), "very encrypted name");
}
