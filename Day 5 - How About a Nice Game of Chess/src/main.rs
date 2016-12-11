#[macro_use]
extern crate try_opt;
extern crate openssl;

mod how_about_a_nice_game_of_chess {
    use ::openssl::hash;

    /// Iterator over a given password characters
    struct PasswordGenerator<'a> {
        door_id: &'a [u8],
        index: u64,
        hasher: hash::Hasher,
    }

    impl<'a> PasswordGenerator<'a> {
        /// Create a new `PasswordGenerator` for a given door.
        fn new(door_id: &'a str) -> Option<PasswordGenerator<'a>> {
            let mdigest = hash::MessageDigest::md5();
            hash::Hasher::new(mdigest).ok().map(|hasher| {
                PasswordGenerator {
                    door_id: door_id.as_bytes(),
                    index: 0,
                    hasher: hasher,
                }
            })
        }
    }

    impl<'a> Iterator for PasswordGenerator<'a> {
        type Item = char;

        /// Find the next character of the password.
        ///
        /// > A hash indicates the next character in the password if its hexadecimal representation
        /// > starts with five zeroes. If it does, the sixth character in the hash is the next
        /// > character of the password.
        fn next(&mut self) -> Option<Self::Item> {
            loop {
                try_opt!(self.hasher.update(self.door_id).ok());
                try_opt!(self.hasher.update(self.index.to_string().as_bytes()).ok());
                // NOTE: .finish() will reset the hasher state so we can reuse it later on.
                let hash = try_opt!(self.hasher.finish().ok());
                self.index += 1;
                // Since one byte is two characters in hex representation, we test the first two
                // byte and the most significants 4 bits ("high part") of the third.
                if (hash[0] | hash[1] | (hash[2] & 0xf0)) == 0 {
                    // the sixth character is the least significants 4 bits ("low part") of the
                    // third byte.
                    return format!("{:x}", hash[2] & 0xf).pop();
                }
            }
        }
    }

    /// Represent a security door designed by Easter Bunny engineers.
    pub struct SecurityDoor {
        door_id: String,
        password_len: usize,
    }

    impl SecurityDoor {
        /// Create a new `SecurityDoor` given a door ID.
        pub fn new(door_id: &str) -> SecurityDoor {
            SecurityDoor {
                door_id: door_id.to_string(),
                password_len: 8,
            }
        }

        /// Generate the password for this door according to the Easter Bunny engineers
        /// questionable algorithm.
        ///
        /// # Errors
        ///
        /// When the password generation failed.
        pub fn password(&self) -> Result<String, String> {
            if let Some(gen) = PasswordGenerator::new(&self.door_id) {
                let password: String = gen.take(self.password_len).collect();
                if password.len() == self.password_len {
                    return Ok(password);
                }
            }
            return Err("OpenSSL error".to_string());
        }

        /// Returns this `SecurityDoor` door ID.
        pub fn door_id(&self) -> &str {
            &self.door_id
        }
    }
}


use how_about_a_nice_game_of_chess::*;

fn main() {
    // acquire data from stdin, we only need the first line.
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("no input given");

    let door = SecurityDoor::new(input.trim());
    println!("The password for the door(ID={}) is: {}", door.door_id(), door.password().unwrap());
}

#[test]
fn part1_example() {
    let door = SecurityDoor::new("abc");
    assert_eq!(door.password().unwrap(), "18f47a30".to_string());
}
