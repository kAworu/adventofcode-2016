extern crate openssl;

mod how_about_a_nice_game_of_chess {
    mod hashing {
        use ::openssl::hash::{Hasher, MessageDigest};

        /// Iterator over the interesting hashes of door_id starting at index zero.
        pub struct InterestingHashFinder<'a> {
            door_id: &'a [u8],
            index: u64,
            hasher: Hasher,
        }

        impl<'a> InterestingHashFinder<'a> {
            /// Create a new `InterestingHashFinder` for a given door.
            pub fn new(door_id: &'a str) -> Option<InterestingHashFinder<'a>> {
                let mdigest = MessageDigest::md5();
                let hasher  = Hasher::new(mdigest).ok()?;
                Some(InterestingHashFinder {
                    door_id: door_id.as_bytes(),
                    index: 0,
                    hasher: hasher,
                })
            }
        }

        impl<'a> Iterator for InterestingHashFinder<'a> {
            type Item = String;

            /// Find the next interesting hash in the index sequence.
            ///
            /// > A hash indicates the next character in the password if its hexadecimal representation
            /// > starts with five zeroes.
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    self.hasher.update(self.door_id).ok()?;
                    self.hasher.update(self.index.to_string().as_bytes()).ok()?;
                    // NOTE: finish() will reset the hasher state so we can reuse it later on.
                    let hash = self.hasher.finish().ok()?;
                    self.index += 1;
                    // Since one byte is two characters in hex representation, we test the first two
                    // byte and the most significants 4 bits ("high part") of the third.
                    if (hash[0] | hash[1] | (hash[2] & 0xf0)) == 0 {
                        let hex = hash.iter().map(|byte| format!("{:02x}", byte)).collect();
                        return Some(hex);
                    }
                }
            }
        }
    }

    /// The password character count.
    const PASSWORD_LEN: usize = 8;
    const UNKNOWN_CHAR: char = '_';

    /// Represent a `SecurityDoor` password
    #[derive(Debug)]
    pub struct Password {
        characters: [char; PASSWORD_LEN],
    }

    impl Password {
        /// Create a new (completely unknown) password
        fn new() -> Password {
            Password {
                characters: [UNKNOWN_CHAR; PASSWORD_LEN],
            }
        }

        /// Returns true if all characters are known in self, false otherwise.
        pub fn is_known(&self) -> bool {
            self.characters.iter().all(|&ch| ch != UNKNOWN_CHAR)
        }

        /// Convert the underlying characters array of self into a `String`
        pub fn to_string(&self) -> String {
            self.characters.iter().map(|&ch| ch).collect()
        }

    }

    impl ::std::fmt::Display for Password {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "{}", self.to_string())
        }
    }

    /// Represent a security door designed by Easter Bunny engineers.
    #[derive(Debug)]
    pub struct SecurityDoor {
        door_id: String,
    }

    impl SecurityDoor {
        /// Create a new `SecurityDoor` given a door ID.
        pub fn new(door_id: &str) -> SecurityDoor {
            SecurityDoor { door_id: door_id.to_string() }
        }

        /// Generate both passwords (for the first and the second door) according to the Easter
        /// Bunny engineers questionable algorithm.
        ///
        /// The cracking process will continue as long as the given `progress` function return
        /// `true`.
        ///
        /// # Errors
        ///
        /// When the password generation failed.
        pub fn crack<T>(&self, progress: T) -> Result<(Password, Password), String>
                where T: Fn(&Password, &Password) -> bool {
            let mut passwords = (Password::new(), Password::new());
            let mut generator = hashing::InterestingHashFinder::new(&self.door_id).ok_or("OpenSSL error")?;
            while progress(&passwords.0, &passwords.1) {
                let hash_str = generator.next().ok_or("Password generation failure")?;
                let sixth    = hash_str.chars().nth(5).ok_or("Password generation error")?;
                let seventh  = hash_str.chars().nth(6).ok_or("Password generation error")?;
                // First door password:
                // > […] the sixth character in the hash is the next character of the password.
                let position = passwords.0.characters.iter().position(|&ch| ch == UNKNOWN_CHAR);
                if let Some(index) = position {
                    passwords.0.characters[index] = sixth;
                }
                // Second door password:
                // > […] the sixth character represents the position (0-7), and the seventh
                // > character is the character to put in that position.
                // > […] Use only the first result for each position, and ignore invalid positions.
                let index = (sixth as u8 - '0' as u8) as usize;
                if index < PASSWORD_LEN && passwords.1.characters[index] == UNKNOWN_CHAR {
                    passwords.1.characters[index] = seventh;
                }
            }
            Ok(passwords)
        }
    }
}


use ::std::io::Write;
use how_about_a_nice_game_of_chess::*;

fn main() {
    // acquire data from stdin, we only need the first line.
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("no input given");

    println!("\rCracking both passwords:");
    let door = SecurityDoor::new(input.trim());
    door.crack(|ref first, ref second| {
        print!("\rFirst door: {}, Second door: {}", first, second);
        // .ok() to ignore the returned Result.
        std::io::stdout().flush().ok();
        // continue while either password is not known yet.
        !first.is_known() || !second.is_known()
    }).ok(); // same .ok() trick as for flushing stdout.
    println!("");
}

#[test]
fn part1_example() {
    let door = SecurityDoor::new("abc");
    let password = door.crack(|ref first, _| !first.is_known()).unwrap().0;
    assert_eq!(password.to_string(), "18f47a30".to_string());
}

#[test]
fn part2_example() {
    let door = SecurityDoor::new("abc");
    let password = door.crack(|_,ref second| !second.is_known()).unwrap().1;
    assert_eq!(password.to_string(), "05ace8e3".to_string());
}
