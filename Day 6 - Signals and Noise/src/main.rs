mod signals_and_noise {
    use ::std::collections::HashMap;
    use ::std::ops::{Deref, DerefMut};
    use ::std::str::FromStr;

    /// Represent characters frequency counters for a given message position.
    #[derive(Debug)]
    struct CharFreq(HashMap<char, u32>);

    impl CharFreq {
        /// Create a new `CharFreq`
        fn new() -> CharFreq {
            CharFreq(HashMap::new())
        }

        /// Returns the character having the maximum frequency.
        ///
        /// If many characters are tied for the maximum frequency, the return value is one of them
        /// choosen arbitrarily. If self is empty, return `None`.
        fn most_frequent_character(&self) -> Option<char> {
            // build a vector of tuple (char, frequency) from the hash (key, value) so we can sort
            // our results.
            let mut vec: Vec<_> = self.iter().collect();
            // compare by the frequency (value) in the descending order (i.e. the most frequent
            // first), hence "b cmp a".
            vec.sort_by(|&(_, freqa), &(_, freqb)| freqb.cmp(&freqa));
            // map to the char, we don't need the frequency anymore
            vec.into_iter().map(|(&ch, _)| ch).next()
        }
    }

    impl Deref for CharFreq {
        type Target = HashMap<char, u32>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for CharFreq {
        fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target {
            &mut self.0
        }
    }


    /// Represents an error corrector device used to communicate with Santa when the signal is poor
    /// or jammed.
    #[derive(Debug)]
    pub struct ErrorCorrector(Vec<CharFreq>);

    impl ErrorCorrector {
        /// Create a new `ErrorCorrector`
        fn new() -> ErrorCorrector {
            ErrorCorrector(Vec::new())
        }

        /// Register a given character `ch` at the position `index`.
        fn register(&mut self, ch:char, index: usize) {
            let ref mut vec = self.0;
            // ensure to have a CharFreq at self.0[index]
            while vec.len() <= index {
                vec.push(CharFreq::new());
            }
            *vec[index].entry(ch).or_insert(0) += 1;
        }

        /// Compute and return the error-corrected message version.
        pub fn message(&self) -> String {
            self.0.iter().filter_map(|cfreq| cfreq.most_frequent_character()).collect()
        }
    }

    impl FromStr for ErrorCorrector {
        type Err = ();

        fn from_str(s: &str) -> Result<ErrorCorrector, Self::Err> {
            let mut ec = ErrorCorrector::new();
            for line in s.lines() {
                for (index, ch) in line.chars().enumerate() {
                    ec.register(ch, index);
                }
            }
            Ok(ec)
        }
    }
}


use std::io::Read;
use signals_and_noise::*;

fn main() {
    // acquire data from stdin.
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");

    let ec: ErrorCorrector = input.parse().unwrap();
    println!("The error-corrected version of the message is: {}", ec.message());
}


#[test]
fn part1_example() {
    let message = "\
eedadn
drvtee
eandsr
raavrd
atevrs
tsrnev
sdttsa
rasrtv
nssdts
ntnada
svetve
tesnvt
vntsnd
vrdear
dvrsen
enarar";
    let ec: ErrorCorrector = message.parse().unwrap();
    assert_eq!(ec.message(), "easter".to_string());
}
