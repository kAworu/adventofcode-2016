// XXX: as of December 2016 the `pattern` API is unstable, see #27721
#![feature(pattern)]

mod internet_protocol_version_7 {
    use ::std::collections::{VecDeque, HashSet};
    use ::std::iter::{Peekable, Enumerate, Skip, FromIterator};
    use ::std::str::pattern::{Pattern, Searcher, SearchStep};
    use ::std::str::{Chars, FromStr};

    /// The hypernet start/stop markers in an `Ipv7Addr`.
    const HYPERNET_START: char = '[';
    const HYPERNET_STOP:  char = ']';

    /// A `Searcher` matching both `Ipv7Addr` `Segments` types (i.e. hypernet and supernet).
    struct SegmentSearcher<'a> {
        haystack: &'a str,
        iter: Peekable<Enumerate<Chars<'a>>>,
    }

    impl<'a> SegmentSearcher<'a> {
        /// Create a new `SegmentSearcher`.
        fn new(haystack: &'a str) -> SegmentSearcher<'a> {
            SegmentSearcher {
                haystack: haystack,
                iter: haystack.chars().enumerate().peekable(),
            }
        }

        /// Matches a hypernet Segment, something like `[foo]`.
        ///
        /// # Panic
        ///
        /// when the next char is not the `HYPERNET_START` marker.
        fn hypernet(&mut self) -> SearchStep {
            // sanity check: look for the hypernet start marker.
            let (start, lead) = self.iter.next().unwrap();
            if lead != HYPERNET_START {
                panic!(format!("expected {} (HYPERNET_START), got {}", HYPERNET_START, lead));
            }
            // find the matching hypernet stop marker.
            if let Some((stop, _)) = self.iter.find(|&(_, ch)| ch == HYPERNET_STOP) {
                // NOTE: (stop + 1) to include the HYPERNET_STOP in the match.
                SearchStep::Match(start, stop + 1)
            } else {
                SearchStep::Reject(start, self.haystack.len())
            }
        }

        /// Matches a supernet Segment, something like `foo`.
        ///
        /// # Panic
        ///
        /// when the next char is the `HYPERNET_START` marker.
        fn supernet(&mut self) -> SearchStep {
            // sanity check: look for something else than the hypernet start marker.
            let (start, lead) = self.iter.next().unwrap();
            if lead == HYPERNET_START {
                panic!(format!("unexpected {} (HYPERNET_START)", HYPERNET_START));
            }
            loop {
                // NOTE: Because we don't want to "eat" the next HYPERNET_START marker from our
                // iterator, we have to use peek() and "find by hand".
                match self.iter.peek() {
                    None => {
                        // we are at the end of the haystack, return a "full" match.
                        return SearchStep::Match(start, self.haystack.len());
                    }
                    Some(&(pos, ch)) if ch == HYPERNET_START => {
                        // we found a HYPERNET_START marker, match until the character just before
                        // it.
                        return SearchStep::Match(start, pos);
                    },
                    Some(_) => {
                        // this char is ours, check the next.
                        self.iter.next();
                    }
                }
            }
        }
    }

    unsafe impl<'a> Searcher<'a> for SegmentSearcher<'a> {
        fn haystack(&self) -> &'a str {
            self.haystack
        }

        fn next(&mut self) -> SearchStep {
            match self.iter.peek() {
                Some(&(_, lead)) if lead == HYPERNET_START => self.hypernet(),
                Some(_) => self.supernet(),
                None => SearchStep::Done,
            }
        }
    }

    /// `Pattern` associated with `SegmentSearcher`.
    struct SegmentPattern { }

    impl SegmentPattern {
        /// Create a new `SegmentPattern` matching both hypernet and supernet segments.
        fn all() -> SegmentPattern {
            SegmentPattern { }
        }
    }

    impl<'a> Pattern<'a> for SegmentPattern {
        type Searcher = SegmentSearcher<'a>;

        fn into_searcher(self, haystack: &'a str) -> SegmentSearcher<'a> {
            SegmentSearcher::new(haystack)
        }
    }

    /// A `Searcher` matching ABBA patterns.
    struct AbbaSearcher<'a> {
        haystack: &'a str,
        iter: Skip<Enumerate<Chars<'a>>>,
        prev3: VecDeque<char>,
    }

    impl<'a> AbbaSearcher<'a> {
        /// Create a new `AbbaSearcher`.
        fn new(haystack: &'a str) -> AbbaSearcher<'a> {
            // NOTE: we want to analyze the haystack characters within the context of a potential
            // ABBA pattern `abcd` of 4 character long. To do so we setup `prev3` to have the first
            // three characters from the haystack and `iter` so that `iter.next()` will yield the
            // fourth character the first time it is called.
            let prev3: VecDeque<_> = haystack.chars().take(3).collect();
            let iter = haystack.chars().enumerate().skip(3);
            AbbaSearcher {
                haystack: haystack,
                iter: iter,
                prev3: prev3,
            }
        }
    }

    unsafe impl<'a> Searcher<'a> for AbbaSearcher<'a> {
        fn haystack(&self) -> &'a str {
            self.haystack
        }

        fn next(&mut self) -> SearchStep {
            // we're looking for an ABBA pattern in a "slice" `abcd` of the haystack. At that point
            // the first three characters `abc` are remembered (in order) in `self.prev3` and
            // `self.iter.next()` will yield the fourth `d` character.
            if let Some((index_of_d, d)) = self.iter.next() {
                // At that point self.prev3 is guaranteed to contains `abc`.
                let (a, b, c) = (self.prev3[0], self.prev3[1], self.prev3[2]);
                let index_of_a = index_of_d - 3;
                // setup `self.prev3` to contains `bcd`, the next iteration's `abc`.
                self.prev3.pop_front();
                self.prev3.push_back(d);
                // check for an ABBA pattern in `abcd`.
                if a == d && b == c && a != b {
                    SearchStep::Match(index_of_a, index_of_d + 1)
                } else {
                    SearchStep::Reject(index_of_a, index_of_d + 1)
                }
            } else {
                SearchStep::Done
            }
        }
    }

    /// `Pattern` associated with `AbbaSearcher`.
    struct AbbaPattern { }

    impl AbbaPattern {
        /// Create a new `AbbaPattern` matching all ABBA sequences.
        fn all() -> AbbaPattern {
            AbbaPattern { }
        }
    }

    impl<'a> Pattern<'a> for AbbaPattern {
        type Searcher = AbbaSearcher<'a>;

        fn into_searcher(self, haystack: &'a str) -> AbbaSearcher<'a> {
            AbbaSearcher::new(haystack)
        }
    }

    /// Represents an ABA/BAB pattern.
    // We use `Bab` because `Aba` would be too easy to confuse with `Abba`.
    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    struct Bab {
        b: char, // NOTE: the first and third character
        a: char, // NOTE: the second character
    }

    impl Bab {
        /// returns the logical inverse of self (eg. 'aba' when self is 'bab').
        fn inverse(&self) -> Bab {
            Bab { b: self.a, a: self.b }
        }
    }

    impl FromStr for Bab {
        type Err = String;

        fn from_str(s: &str) -> Result<Bab, String> {
            if s.len() != 3 {
                return Err("empty ABA/BAB string".to_string());
            }
            let mut it = s.chars();
            let (b, a, b2) = (it.next().unwrap(), it.next().unwrap(), it.next().unwrap());
            if b != b2 {
                return Err("non-ABA/BAB string".to_string());
            }
            Ok(Bab { b: b, a: a })
        }
    }

    /// Represents a `Searcher` matching ABA/BAB patterns.
    struct BabSearcher<'a> {
        haystack: &'a str,
        iter: Skip<Enumerate<Chars<'a>>>,
        prev2: VecDeque<char>,
        match_only: Option<&'a HashSet<Bab>>,
    }

    impl<'a> BabSearcher<'a> {
        /// Create a new `BabSearcher`.
        fn new(haystack: &'a str, match_only: Option<&'a HashSet<Bab>>) -> BabSearcher<'a> {
            // NOTE: we want to analyze the haystack characters within the context of a potential
            // ABA/BAB pattern `xyz` of 3 character long. To do so we setup `prev2` to have the
            // first two characters from the haystack and `iter` so that `iter.next()` will yield
            // the third character the first time it is called.
            let prev2: VecDeque<_> = haystack.chars().take(2).collect();
            let iter = haystack.chars().enumerate().skip(2);
            BabSearcher {
                haystack: haystack,
                iter: iter,
                prev2: prev2,
                match_only: match_only,
            }
        }
    }

    unsafe impl<'a> Searcher<'a> for BabSearcher<'a> {
        fn haystack(&self) -> &'a str {
            self.haystack
        }

        fn next(&mut self) -> SearchStep {
            // we're looking for an ABA/BAB pattern in a "slice" `xyz` of the haystack. At that
            // point the first two characters `xy` are remembered (in order) in `self.prev2` and
            // `self.iter.next()` will yield the third `z` character.
            if let Some((index_of_z, z)) = self.iter.next() {
                // At that point self.prev2 is guaranteed to contains `xy`.
                let (x, y) = (self.prev2[0], self.prev2[1]);
                let index_of_x = index_of_z - 2;
                // setup `self.prev2` to contains `yz`, the next iteration's `xy`.
                self.prev2.pop_front();
                self.prev2.push_back(z);
                // check for an ABA/BAB pattern in `xyz`.
                if x == z && x != y {
                    // if we have a match_only, reject unless our freshly found ABA/BAB pattern is
                    // in the set.
                    if let Some(set) = self.match_only {
                        if !set.contains(&Bab { b: x, a: y }) {
                            return SearchStep::Reject(index_of_x, index_of_z + 1);
                        }
                    }
                    SearchStep::Match(index_of_x, index_of_z + 1)
                } else {
                    SearchStep::Reject(index_of_x, index_of_z + 1)
                }
            } else {
                SearchStep::Done
            }
        }
    }

    /// `Pattern` associated with `BabSearcher`.
    struct BabPattern<'a> {
        match_only: Option<&'a HashSet<Bab>>
    }

    impl<'a> BabPattern<'a> {
        /// Create a new `BabPattern` matching all ABA/BAB sequences.
        fn all() -> BabPattern<'a> {
            BabPattern { match_only: None }
        }

        /// Create a new `BabPattern` matching only the ABA/BAB sequences contained in the given
        /// `babset`.
        fn matching(babset: &'a HashSet<Bab>) -> BabPattern<'a> {
            BabPattern { match_only: Some(babset) }
        }
    }

    impl<'a> Pattern<'a> for BabPattern<'a> {
        type Searcher = BabSearcher<'a>;

        fn into_searcher(self, haystack: &'a str) -> BabSearcher<'a> {
            BabSearcher::new(haystack, self.match_only)
        }
    }

    /// Represents an `Ipv7Addr` "segment", either an hypernet or a supernet.
    #[derive(Debug)]
    struct Segment {
        /// `true` if this `Segment` is hypernet, false otherwise (supernet).
        hypernet: bool,
        number: String,
    }

    impl Segment {
        /// Returns `true` if self is a hypernet segment, `false` otherwise.
        fn is_hypernet(&self) -> bool {
            self.hypernet
        }

        /// Returns `true` if self is a supernet segment, `false` otherwise.
        fn is_supernet(&self) -> bool {
            !self.hypernet
        }

        /// Returns `true` if self contains an ABBA pattern, `false` otherwise.
        fn has_abba(&self) -> bool {
            // XXX: could be cached because matching is costly, but we only call it once per
            // `Segment` so that's ok for now.
            self.number.matches(AbbaPattern::all()).next().is_some()
        }

        /// Returns all the unique ABA/BAB patterns contained in self.
        fn babset(&self) -> HashSet<Bab> {
            self.number.matches(BabPattern::all()).map(|s| s.parse().unwrap()).collect()
        }

        /// Returns `true` if self contains any ABA/BAB patterns from the given `babset`, `false`
        /// otherwise.
        fn has_any_bab(&self, babset: &HashSet<Bab>) -> bool {
            self.number.matches(BabPattern::matching(babset)).next().is_some()
        }
    }

    impl FromStr for Segment {
        type Err = String;

        fn from_str(s: &str) -> Result<Segment, String> {
            if s.is_empty() {
                Err("empty segment string".to_string())
            } else if s.starts_with(HYPERNET_START) && s.ends_with(HYPERNET_STOP) {
                let number = s[1..(s.len() - 1)].to_string(); // "trim" both markers
                Ok(Segment { hypernet: true,  number: number })
            } else {
                let number = s.to_string();
                Ok(Segment { hypernet: false, number: number })
            }
        }
    }

    /// Represents an IPv7 from the local network of Easter Bunny HQ.
    #[derive(Debug)]
    pub struct Ipv7Addr {
        segments: Vec<Segment>,
    }

    impl Ipv7Addr {
        /// Returns `true` if self has TLS (transport-layer snooping) support, `false` otherwise.
        ///
        /// > An IP supports TLS if it has an Autonomous Bridge Bypass Annotation, or ABBA […]
        /// > However, the IP also must not have an ABBA within any hypernet sequences […]
        pub fn has_tls_support(&self) -> bool {
            // we have four cases to consider:
            //
            // 1. one  of our hypernet segments has ABBA and one  of our supernet segments has ABBA
            // 2. one  of our hypernet segments has ABBA and none of our supernet segments has ABBA
            // 3. none of our hypernet segments has ABBA and one  of our supernet segments has ABBA
            // 4. none of our hypernet segments has ABBA and none of our supernet segments has ABBA
            //
            // Of the four cases only one, namely #3, is a success (i.e. has TLS support). #1 and
            // #2 fail because of one of our hypernet segment has ABBA and #4 fail because of the
            // lack of any supernet segment with ABBA.
            //
            // Here we're considering the analyze order between our hypernet segments first vs our
            // supernet segments first. Since we don't have any clue and to simplify our reasoning
            // we consider that having ABBA is equally likely in a hypernet segment and a supernet
            // segment of the same length.
            //
            // Intuitively, we find that analyzing our hypernet segments first should be faster
            // because we can "shortcut" (i.e. skip analyzing our supernet segments) in cases #1
            // and #2 as soon as the first hypernet segment with ABBA is found. If we analyze our
            // supernet segments first we can "shortcut" in cases #2 and #4 but only after having
            // analyzing all of them.
            let mut hypernets = self.segments.iter().filter(|&seg| seg.is_hypernet());
            let mut supernets = self.segments.iter().filter(|&seg| seg.is_supernet());
            !hypernets.any(|seg| seg.has_abba()) && supernets.any(|seg| seg.has_abba())
        }

        /// Returns `true` if self has SSL (super-secret listening) support, `false` otherwise.
        ///
        /// > An IP supports SSL if it has an Area-Broadcast Accessor, or ABA, anywhere in the
        /// > supernet sequences (outside any square bracketed sections), and a corresponding Byte
        /// > Allocation Block, or BAB, anywhere in the hypernet sequences.
        pub fn has_ssl_support(&self) -> bool {
            let mut hypernets = self.segments.iter().filter(|&seg| seg.is_hypernet());
            let     supernets = self.segments.iter().filter(|&seg| seg.is_supernet());
            // collect from all the Area-Broadcast Accessor from the supernet sequences.
            let abaset = supernets.fold(HashSet::new(), |mut acc, ref seg| {
                acc.extend(seg.babset());
                acc
            });
            // If we did not find any ABA we're done.
            if abaset.is_empty() {
                return false;
            }
            // build a BAB set from the ABA, "inversing" every elements.
            let babset = HashSet::from_iter(abaset.iter().map(|aba| aba.inverse()));
            // search through all our hypernet segments for one having a matching Byte Allocation
            // Block.
            hypernets.any(|seg| seg.has_any_bab(&babset))
        }
    }

    impl FromStr for Ipv7Addr {
        type Err = String;

        fn from_str(s: &str) -> Result<Ipv7Addr, String> {
            let mut segments = Vec::new();
            for sub_str in s.matches(SegmentPattern::all()) {
                segments.push(sub_str.parse()?);
            }
            Ok(Ipv7Addr { segments: segments })
        }
    }
}


use std::io::Read;
use internet_protocol_version_7::*;

fn main() {
    // Acquire data from stdin.
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");

    // Parse one Ipv7Addr per line of input.
    let ips: Vec<Ipv7Addr> = input.lines().map(|line| line.parse().unwrap()).collect();

    // Compute and report the number of `Ipv7Addr` supporting transport-layer snooping.
    let tls_supporting_count = ips.iter().filter(|ip| ip.has_tls_support()).count();
    println!("Found {} IPv7 with TLS (transport-layer snooping) support.",
        tls_supporting_count);

    // Compute and report the number of `Ipv7Addr` supporting super-secret listening.
    let ssl_supporting_count = ips.iter().filter(|ip| ip.has_ssl_support()).count();
    println!("Found {} IPv7 with SSL (super-secret listening) support.",
        ssl_supporting_count);
}

#[test]
fn part1_first_example() {
    let ip: Ipv7Addr = "abba[mnop]qrst".parse().unwrap();
    println!("{:?}", ip);
    assert!(ip.has_tls_support());
}

#[test]
fn part1_second_example() {
    let ip: Ipv7Addr = "abcd[bddb]xyyx".parse().unwrap();
    println!("{:?}", ip);
    assert!(!ip.has_tls_support());
}

#[test]
fn part1_third_example() {
    let ip: Ipv7Addr = "aaaa[qwer]tyui".parse().unwrap();
    println!("{:?}", ip);
    assert!(!ip.has_tls_support());
}

#[test]
fn part1_fourth_example() {
    let ip: Ipv7Addr = "ioxxoj[asdfgh]zxcvbn".parse().unwrap();
    println!("{:?}", ip);
    assert!(ip.has_tls_support());
}

#[test]
fn part2_first_example() {
    let ip: Ipv7Addr = "aba[bab]xyz".parse().unwrap();
    println!("{:?}", ip);
    assert!(ip.has_ssl_support());
}

#[test]
fn part2_second_example() {
    let ip: Ipv7Addr = "xyx[xyx]xyx".parse().unwrap();
    println!("{:?}", ip);
    assert!(!ip.has_ssl_support());
}

#[test]
fn part2_third_example() {
    let ip: Ipv7Addr = "aaa[kek]eke".parse().unwrap();
    println!("{:?}", ip);
    assert!(ip.has_ssl_support());
}

#[test]
fn part2_fourth_example() {
    let ip: Ipv7Addr = "zazbz[bzb]cdb".parse().unwrap();
    println!("{:?}", ip);
    assert!(ip.has_ssl_support());
}
