// XXX: as of December 2016 the `pattern` API is unstable, see #27721
#![feature(pattern)]

mod internet_protocol_version_7 {
    use ::std::collections::HashSet;
    use ::std::iter::{Enumerate, Map};
    use ::std::slice::Windows;
    use ::std::str::{FromStr, Matches};
    use ::std::str::pattern::{Pattern, Searcher, SearchStep};

    /// A `Searcher` matching ABBA patterns.
    struct AbbaSearcher<'a> {
        haystack: &'a str,
        it: Enumerate<Windows<'a, u8>>,
    }

    impl<'a> AbbaSearcher<'a> {
        /// Create a new `AbbaSearcher`.
        fn new(haystack: &'a str) -> AbbaSearcher<'a> {
            AbbaSearcher {
                haystack: haystack,
                it: haystack.as_bytes().windows(4).enumerate(),
            }
        }
    }

    unsafe impl<'a> Searcher<'a> for AbbaSearcher<'a> {
        fn haystack(&self) -> &'a str {
            self.haystack
        }

        fn next(&mut self) -> SearchStep {
            if let Some((i, slice)) = self.it.next() {
                let (a, b, c, d) = (slice[0], slice[1], slice[2], slice[3]);
                // check for an ABBA pattern in `abcd`.
                if a == d && b == c && a != b {
                    SearchStep::Match(i, i + 4)
                } else {
                    SearchStep::Reject(i, i + 4)
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
        it: Enumerate<Windows<'a, u8>>,
    }

    impl<'a> BabSearcher<'a> {
        /// Create a new `BabSearcher`.
        fn new(haystack: &'a str) -> BabSearcher<'a> {
            BabSearcher {
                haystack: haystack,
                it: haystack.as_bytes().windows(3).enumerate(),
            }
        }
    }

    unsafe impl<'a> Searcher<'a> for BabSearcher<'a> {
        fn haystack(&self) -> &'a str {
            self.haystack
        }

        fn next(&mut self) -> SearchStep {
            if let Some((i, slice)) = self.it.next() {
                let (x, y, z) = (slice[0], slice[1], slice[2]);
                if x == z && x != y {
                    SearchStep::Match(i, i + 3)
                } else {
                    SearchStep::Reject(i, i + 3)
                }
            } else {
                SearchStep::Done
            }
        }
    }

    /// `Pattern` associated with `BabSearcher`.
    struct BabPattern { }

    impl BabPattern {
        /// Create a new `BabPattern` matching all ABA/BAB sequences.
        fn all() -> BabPattern {
            BabPattern { }
        }
    }

    impl<'a> Pattern<'a> for BabPattern {
        type Searcher = BabSearcher<'a>;

        fn into_searcher(self, haystack: &'a str) -> BabSearcher<'a> {
            BabSearcher::new(haystack)
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

        /// Returns an iterator over all the `Bab` patterns contained in self.
        fn bab(&self) -> Map<Matches<BabPattern>, fn(&str) -> Bab>
        {
            // https://www.reddit.com/r/rust/comments/31x7jj/returning_iterators_from_a_function/
            // helped me a lot here.
            fn parse(s: &str) -> Bab {
                s.parse().unwrap()
            }
            self.number.matches(BabPattern::all()).map(parse)
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
            let mut babset = HashSet::new();
            for snet in supernets {
                for aba in snet.bab() {
                    babset.insert(aba.inverse());
                }
            }
            // If we did not find any ABA we're done.
            if babset.is_empty() {
                return false;
            }
            // look through our hypernet for the first BAB match.
            hypernets.any(|seg| {
                seg.bab().any(|bab| babset.contains(&bab))
            })
        }
    }

    /// The hypernet start/stop markers in an `Ipv7Addr`.
    const HYPERNET_START: char = '[';
    const HYPERNET_STOP:  char = ']';

    impl FromStr for Ipv7Addr {
        type Err = String;

        fn from_str(s: &str) -> Result<Ipv7Addr, String> {
            let mut segments = Vec::new();
            let mut start = 0;
            let mut target = HYPERNET_START;
            for (i, c) in s.chars().enumerate() {
                if c == target {
                    segments.push(Segment {
                        hypernet: (target == HYPERNET_STOP),
                        number: s[start..i].to_string()
                    });
                    // update state for the next segment
                    start = i + 1;
                    target = if target == HYPERNET_START {
                        HYPERNET_STOP
                    } else {
                        HYPERNET_START
                    };
                }
            }
            // trailing supernet handling
            if start < s.len() - 1 {
                segments.push(Segment {
                    hypernet: false,
                    number: s[start..s.len()].to_string()
                });
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
