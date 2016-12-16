#![feature(pattern)]

mod internet_protocol_version_7 {
    use ::std::collections::VecDeque;
    use ::std::iter::{Peekable, Enumerate, Skip};
    use ::std::str::pattern::{Pattern, Searcher, SearchStep};
    use ::std::str::{Chars, FromStr};

    const HYPERNET_START: char = '[';
    const HYPERNET_STOP:  char = ']';

    struct SubnetSearcher<'a> {
        haystack: &'a str,
        iter: Peekable<Enumerate<Chars<'a>>>,
    }

    impl<'a> SubnetSearcher<'a> {
        fn new(haystack: &'a str) -> SubnetSearcher<'a> {
            SubnetSearcher {
                haystack: haystack,
                iter: haystack.chars().enumerate().peekable(),
            }
        }
    }

    unsafe impl<'a> Searcher<'a> for SubnetSearcher<'a> {
        fn haystack(&self) -> &'a str {
            self.haystack
        }
        fn next(&mut self) -> SearchStep {
            match self.iter.next() {
                None => SearchStep::Done,
                Some((start, lead)) if lead == HYPERNET_START => { // hypernet
                    while let Some((pos, ch)) = self.iter.next() {
                        if ch == HYPERNET_STOP {
                            return SearchStep::Match(start, pos + 1);
                        }
                    }
                    SearchStep::Reject(start, self.haystack.len())
                },
                Some((start, _)) => { // !hypernet
                    while let Some(&(pos, ch)) = self.iter.peek() {
                        if ch == HYPERNET_START {
                            return SearchStep::Match(start, pos);
                        }
                        self.iter.next();
                    }
                    SearchStep::Match(start, self.haystack.len())
                },
            }
        }
    }

    struct SubnetPattern { }

    impl SubnetPattern {
        fn new() -> SubnetPattern {
            SubnetPattern { }
        }
    }

    impl<'a> Pattern<'a> for SubnetPattern {
        type Searcher = SubnetSearcher<'a>;

        fn into_searcher(self, haystack: &'a str) -> SubnetSearcher<'a> {
            SubnetSearcher::new(haystack)
        }
    }

    struct AbbaSearcher<'a> {
        haystack: &'a str,
        iter: Skip<Enumerate<Chars<'a>>>,
        last3: VecDeque<char>,
    }

    impl<'a> AbbaSearcher<'a> {
        fn new(haystack: &'a str) -> AbbaSearcher<'a> {
            let last3: VecDeque<_> = haystack.chars().take(3).collect();
            let iter = haystack.chars().enumerate().skip(3);
            AbbaSearcher {
                haystack: haystack,
                iter: iter,
                last3: last3,
            }
        }
    }

    unsafe impl<'a> Searcher<'a> for AbbaSearcher<'a> {
        fn haystack(&self) -> &'a str {
            self.haystack
        }
        fn next(&mut self) -> SearchStep {
            if let Some((index, current)) = self.iter.next() {
                let (one, two, three) = (self.last3[0], self.last3[1], self.last3[2]);
                let four = current;
                self.last3.pop_front();
                self.last3.push_back(four);
                if one == four && two == three && one != two {
                    SearchStep::Match(index - 3, index + 1)
                } else {
                    SearchStep::Reject(index - 3, index + 1)
                }
            } else {
                SearchStep::Done
            }
        }
    }

    struct AbbaPattern { }

    impl AbbaPattern {
        fn new() -> AbbaPattern {
            AbbaPattern { }
        }
    }

    impl<'a> Pattern<'a> for AbbaPattern {
        type Searcher = AbbaSearcher<'a>;

        fn into_searcher(self, haystack: &'a str) -> AbbaSearcher<'a> {
            AbbaSearcher::new(haystack)
        }
    }

    #[derive(Debug)]
    struct Subnet {
        is_hypernet: bool,
        number: String,
    }

    impl Subnet {
        fn is_hypernet(&self) -> bool {
            self.is_hypernet
        }
        fn has_abba(&self) -> bool {
            self.number.matches(AbbaPattern::new()).take(1).next().is_some()
        }
    }

    impl FromStr for Subnet {
        type Err = String;

        fn from_str(s: &str) -> Result<Subnet, String> {
            if s.is_empty() {
                Err("empty subnet string".to_string())
            } else if s.starts_with(HYPERNET_START) && s.ends_with(HYPERNET_STOP) {
                Ok(Subnet { is_hypernet: true,  number: s[1..(s.len() - 1)].to_string() })
            } else {
                Ok(Subnet { is_hypernet: false, number: s.to_string() })
            }
        }
    }

    #[derive(Debug)]
    pub struct Ipv7Addr {
        subnets: Vec<Subnet>,
    }

    impl Ipv7Addr {
        pub fn has_tls_support(&self) -> bool {
            // we have four cases to consider:
            // 1. one  of our hypernet has ABBA and one of our non-hypernet has ABBA
            // 2. one  of our hypernet has ABBA and no  of our non-hypernet has ABBA
            // 3. none of our hypernet has ABBA and one of our non-hypernet has ABBA
            // 4. none of our hypernet has ABBA and no  of our non-hypernet has ABBA
            !self.subnets.iter().any(|subnet|  subnet.is_hypernet() && subnet.has_abba()) &&
             self.subnets.iter().any(|subnet| !subnet.is_hypernet() && subnet.has_abba())
        }
    }

    impl FromStr for Ipv7Addr {
        type Err = String;

        fn from_str(s: &str) -> Result<Ipv7Addr, String> {
            let mut subnets = Vec::new();
            for sub in s.matches(SubnetPattern::new()) {
                subnets.push(sub.parse()?);
            }
            Ok(Ipv7Addr { subnets: subnets })
        }
    }
}


use std::io::Read;
use internet_protocol_version_7::*;

fn main() {
    // acquire data from stdin.
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");

    let xs: Vec<Ipv7Addr> = input.lines().map(|line| line.parse().unwrap()).collect();
    println!("{}", xs.into_iter().filter(|ip| ip.has_tls_support()).count());
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
