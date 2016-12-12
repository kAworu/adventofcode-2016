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
            // compare by the frequency (value) in the descending order (i.e. the most frequent
            // first), hence "b cmp a".
            self.first_char_sort_by_freq(|a, b| b.cmp(&a))
        }

        /// Returns the character having the minimum frequency.
        ///
        /// If many characters are tied for the minimum frequency, the return value is one of them
        /// choosen arbitrarily. If self is empty, return `None`.
        fn least_frequent_character(&self) -> Option<char> {
            // compare by the frequency (value) in the ascending order (i.e. the least frequent
            // first), hence "a cmp b".
            self.first_char_sort_by_freq(|a, b| a.cmp(&b))
        }

        /// Returns the first character of self sorted by a given `cmp` comparison function on the
        /// frequency.
        fn first_char_sort_by_freq<F>(&self, mut cmp: F) -> Option<char>
            where F: FnMut(&u32, &u32) -> ::std::cmp::Ordering
        {
            // build a vector of tuple (char, frequency) from the hash (key, value) so we can sort
            // our results.
            let mut vec: Vec<_> = self.iter().collect();
            vec.sort_by(|&(_, freqa), &(_, freqb)| cmp(freqa, freqb));
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
        fn register(&mut self, ch: char, index: usize) {
            let ref mut vec = self.0;
            // ensure to have a CharFreq at self.0[index]
            while vec.len() <= index {
                vec.push(CharFreq::new());
            }
            *vec[index].entry(ch).or_insert(0) += 1;
        }

        /// Compute and return the error-corrected message version using the simple repetition code
        /// protocol.
        pub fn src_message(&self) -> String {
            self.0.iter().filter_map(|cfreq| cfreq.most_frequent_character()).collect()
        }

        /// Compute and return the original message using the modified repetition code protocol.
        pub fn mrc_message(&self) -> String {
            self.0.iter().filter_map(|cfreq| cfreq.least_frequent_character()).collect()
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
    println!("The error-corrected version of the message is: {}",
             ec.src_message());
    println!("The original message is: {}", ec.mrc_message());
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
    assert_eq!(ec.src_message(), "easter".to_string());
}

#[test]
fn part2_example() {
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
    assert_eq!(ec.mrc_message(), "advent".to_string());
}
