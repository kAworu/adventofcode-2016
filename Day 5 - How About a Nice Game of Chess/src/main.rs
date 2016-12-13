#[macro_use]
extern crate try_opt;
extern crate openssl;

mod how_about_a_nice_game_of_chess {
    use ::openssl::hash;

    /// Iterator over the interesting hashes of door_id starting at index zero.
    struct InterestingHashFinder<'a> {
        door_id: &'a [u8],
        index: u64,
        hasher: hash::Hasher,
    }

    impl<'a> InterestingHashFinder<'a> {
        /// Create a new `InterestingHashFinder` for a given door.
        fn new(door_id: &'a str) -> Option<InterestingHashFinder<'a>> {
            let mdigest = hash::MessageDigest::md5();
            let hasher = try_opt!(hash::Hasher::new(mdigest).ok());
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
            'critical: loop {
                try_opt!(self.hasher.update(self.door_id).ok());
                try_opt!(self.hasher.update(self.index.to_string().as_bytes()).ok());
                // NOTE: finish() will reset the hasher state so we can reuse it later on.
                let hash = try_opt!(self.hasher.finish().ok());
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

    /// The password character count.
    const PASSWORD_LEN: usize = 8;

    /// Represent a security door designed by Easter Bunny engineers.
    #[derive(Debug)]
    pub struct SecurityDoor {
        door_id: String,
    }

    // FIXME: merge first_password() and second_password() into one password fn returning
    // Result<(String, String), String> that avoid to iter twice over "the same"
    // InterestingHashFinder.
    impl SecurityDoor {
        /// Create a new `SecurityDoor` given a door ID.
        pub fn new(door_id: &str) -> SecurityDoor {
            SecurityDoor { door_id: door_id.to_string() }
        }

        /// Generate the password for the first door according to the Easter Bunny engineers
        /// questionable algorithm.
        ///
        /// # Errors
        ///
        /// When the password generation failed.
        pub fn first_password(&self) -> Result<String, String> {
            if let Some(gen) = InterestingHashFinder::new(&self.door_id) {
                let password: String = gen.take(PASSWORD_LEN)
                    .filter_map(|hash_str| hash_str.chars().nth(5))
                    .collect();
                if password.len() == PASSWORD_LEN {
                    return Ok(password);
                }
            }
            return Err("OpenSSL error".to_string());
        }

        /// Generate the password for the second door according to the Easter Bunny engineers
        /// (still) questionable second (out-of-order) algorithm.
        ///
        /// # Errors
        ///
        /// When the password generation failed.
        pub fn second_password(&self) -> Result<String, String> {
            let mut gen = try!(InterestingHashFinder::new(&self.door_id).ok_or("OpenSSL error"));
            let mut password = ['?'; PASSWORD_LEN];
            while password.iter().any(|&ch| ch == '?') {
                let hash_str = try!(gen.next().ok_or("Password generation failure"));
                let sixth    = try!(hash_str.chars().nth(5).ok_or("Password generation error"));
                let seventh  = try!(hash_str.chars().nth(6).ok_or("Password generation error"));
                let position = (sixth as u8 - '0' as u8) as usize;
                if position < PASSWORD_LEN && password[position] == '?' {
                    password[position] = seventh;
                }
            }
            Ok(password.iter().map(|&ch| ch).collect())
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
    println!("The password of the first door(ID={}) is: {}",
             door.door_id(),
             door.first_password().unwrap());
    println!("The password of the second door(ID={}) is: {}",
             door.door_id(),
             door.second_password().unwrap());
}

#[test]
fn part1_example() {
    let door = SecurityDoor::new("abc");
    assert_eq!(door.first_password().unwrap(), "18f47a30".to_string());
}

#[test]
fn part2_example() {
    let door = SecurityDoor::new("abc");
    assert_eq!(door.second_password().unwrap(), "05ace8e3".to_string());
}
