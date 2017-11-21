#[macro_use]
extern crate lazy_static;
extern crate regex;

mod balance_bots {
    use ::regex::Regex;
    use ::std::collections::HashMap;
    use ::std::str::FromStr;

    /// Used to identify robots and bins.
    pub type Id = u32;

    /// `Microchip` numbers.
    pub type Value = u32;

    /// Represents a microchip of a given value.
    #[derive(Hash, Eq, PartialEq, PartialOrd, Copy, Clone, Debug)]
    pub struct Microchip(pub Value);

    /// Used to make the distinction between lower-value and higher-value microchip.
    #[derive(Hash, Eq, PartialEq, PartialOrd, Copy, Clone, Debug)]
    enum MicrochipWeight {
        Higher,
        Lower,
    }

    /// A couple of microchips. This along `MicrochipWeight` are useful because robots handle
    /// microchips by pair caring about which one is the lower-value and high-value.
    #[derive(Hash, Eq, PartialEq, PartialOrd, Copy, Clone, Debug)]
    struct Microchip2 {
        low: Microchip,
        high: Microchip,
    }

    impl Microchip2 {
        /// Create a new pair of microchip. `a` and `b` can be given in any order, that is:
        /// Microchip2::new(a, b) == Microchip2::new(b, a)
        fn new(a: Microchip, b: Microchip) -> Microchip2 {
            let (low, high) = if a > b { (b, a) } else { (a, b) };
            Microchip2 { low, high }
        }
    }

    /// Identify an microchip donation output, either a robot or an output bin.
    #[derive(Eq, PartialEq, PartialOrd, Copy, Clone, Debug)]
    pub enum Output {
        Robot(Id),
        Bin(Id),
    }

    /// Used to make a link from an output to their input. An input can be:
    /// 1. a robot making a `Donation` of its lower-value microchip,
    /// 2. a robot making a `Donation` of its higher-value microchip,
    /// 3. an `Input` bin giving its sole microchip.
    #[derive(Copy, Clone, Debug)]
    enum Gift {
        Donation {
            from_robot_id: Id,
            weight: MicrochipWeight,
        },
        Input {
            chip: Microchip,
        }
    }

    /// Represents a robot from the factory.
    #[derive(Debug)]
    struct Robot {
        id: Id,
        // Its two inputs, each are either another robot's `Donation` or an `Input` bin.
        from: (Gift, Gift),
        // the output to which this robot donate its lower-value microchip
        low_to:  Output,
        // the output to which this robot donate its higher-value microchip
        high_to: Output,
    }

    impl Robot {
        /// Returns `true` if this robot has taken the `target` microchip
        /// **directly from an input bin**, `false` otherwise.
        fn is_initially_holding(&self, target: Microchip) -> bool {
            match self.from {
                (Gift::Input { chip }, _) if target == chip => true,
                (_, Gift::Input { chip }) if target == chip => true,
                _ => false,
            }
        }
    }

    /// Represent an output bin.
    #[derive(Debug)]
    struct Bin {
        id: Id,
        // NOTE: technically this bin could get its microchip from an input bin.
        from: Gift,
    }

    /// An instruction from the local control computer.
    #[derive(Copy, Clone, Debug)]
    pub enum Instruction {
        // value `chip` goes to bot `robot_id`
        Take { chip: Microchip, robot_id: Id },
        // bot `robot_id` gives low to `low` and high to `high`
        Donate { robot_id: Id, low: Output, high: Output },
    }

    impl FromStr for Instruction {
        type Err = String;

        /// Parse an `Instruction`.
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            lazy_static! {
                static ref TAKE: Regex = Regex::new(
                    r"value (?P<value>\d+) goes to bot (?P<rid>\d+)"
                ).unwrap();
                static ref DONATE: Regex = Regex::new(
                    r"bot (?P<rid>\d+) gives low to (?P<l>bot|output) (?P<lid>\d+) and high to (?P<h>bot|output) (?P<hid>\d+)"
                ).unwrap();
            }
            if let Some(caps) = TAKE.captures(s) {
                let value: Value = caps["value"].parse().unwrap();
                let id: Id = caps["rid"].parse().unwrap();
                Ok(Instruction::Take { chip: Microchip(value), robot_id: id })
            } else if let Some(caps) = DONATE.captures(s) {
                let robot_id: Id = caps["rid"].parse().unwrap();
                let low_id:   Id = caps["lid"].parse().unwrap();
                let high_id:  Id = caps["hid"].parse().unwrap();
                let low_receiver = if &caps["l"] == "bot" {
                    Output::Robot(low_id)
                } else {
                    Output::Bin(low_id)
                };
                let high_receiver = if &caps["h"] == "bot" {
                    Output::Robot(high_id)
                } else {
                    Output::Bin(high_id)
                };
                Ok(Instruction::Donate {
                    robot_id: robot_id,
                    low: low_receiver,
                    high: high_receiver
                })
            } else {
                Err(format!("unrecognized instructions: {}", s))
            }
        }
    }

    /// The strange place we end up in: full of robots, bins and microchips.
    #[derive(Debug)]
    pub struct Factory {
        robots: HashMap<Id, Robot>,
        bins:   HashMap<Id, Bin>,
    }

    impl Factory {
        /// Creates a new "empty" factory.
        fn new() -> Factory {
            Factory {
                robots: HashMap::new(),
                bins:   HashMap::new(),
            }
        }

        /// Build a new factory based on a given list of instructions.
        pub fn build_from(instructions: &Vec<Instruction>) -> Factory {
            // While our `Robot` struct must be fully defined (inputs and outputs), its parameters
            // may be provided across as much as three non-consecutive instructions (two inputs,
            // one for its outputs). We work around this by looping a first time to build hashes of
            // theses parameters and then build the robots.
            //
            // On the other hand, output `Bin` may be created from a single instruction (defining
            // its only input) so we do it directly in the first loop.
            let mut factory = Factory::new();
            // robots id to its input, the vectors are expected to be of size two once we're done
            // with the first processing loop.
            let mut robots_inputs:  HashMap<Id, Vec<Gift>> = HashMap::new();
            // robots id and weight to outputs.
            let mut robots_outputs: HashMap<(Id, MicrochipWeight), Output> = HashMap::new();

            // first processing loop: create the output bins and fill in both `robots_inputs` and
            // `robots_outputs`
            for &instruction in instructions.iter() {
                match instruction {
                    Instruction::Take { robot_id: receiver_id, chip } => {
                        let mut inputs = robots_inputs.entry(receiver_id).or_insert_with(|| Vec::new());
                        inputs.push(Gift::Input { chip });
                    },
                    Instruction::Donate { robot_id: from_robot_id, low, high } => {
                        let receivers = [(MicrochipWeight::Lower, low), (MicrochipWeight::Higher, high)];
                        for &(weight, output) in receivers.iter() {
                            robots_outputs.insert((from_robot_id, weight), output);
                            match output {
                                Output::Robot(robot_id) => {
                                    let mut inputs = robots_inputs.entry(robot_id).or_insert_with(|| Vec::new());
                                    inputs.push(Gift::Donation { from_robot_id, weight });
                                },
                                Output::Bin(bin_id) => {
                                    factory.bins.insert(bin_id, Bin {
                                        id: bin_id,
                                        from: Gift::Donation { from_robot_id, weight },
                                    });
                                },
                            }
                        }
                    },
                }
            }

            // second loop, create the all the `Robot` from `robots_inputs` and `robots_outputs`.
            for (&rid, ref froms) in robots_inputs.iter() {
                assert_eq!(froms.len(), 2); // sanity check
                let &low_to  = robots_outputs.get(&(rid, MicrochipWeight::Lower)).unwrap();
                let &high_to = robots_outputs.get(&(rid, MicrochipWeight::Higher)).unwrap();
                factory.robots.insert(rid, Robot {
                    id: rid,
                    from: (froms[0], froms[1]),
                    low_to,
                    high_to,
                });
            }

            // we're done
            return factory;
        }

        /// Returns the robot responsible for comparing the microchip pair `(m0, m1)`.
        pub fn robot_comparing(&self, m0: Microchip, m1: Microchip) -> Option<Id> {
            // Each microchip follow a similar path. It start with an input bin, then goes through
            // a number of robots comparing it, and finally is given to an output bin. We can
            // represent the "path" that a microchip goes through like this:
            //
            //     input bin → first robot → another robot → another robot → ... → output bin
            //
            // starting with the robot initially holding `m0` (arbitrarily), our goal is to follow
            // its path until we find a robot comparing `m0` with `m1` (our target pair) or its
            // output bin (meaning that no robot is responsible for comparing our target pair).
            let target_pair = Microchip2::new(m0, m1);
            // Find out which robot is taking one of the target microchip from an input bin.
            let first_robot = self.robots.values().find(|&robot| robot.is_initially_holding(m0));
            if first_robot.is_none() {
                return None;
            }
            // memoized hash from robots id to its compared microchips.
            let mut memo: HashMap<Id, Microchip2> = HashMap::new();
            let mut robot = first_robot.unwrap();
            loop {
                let robot_pair = self.compared_microchips(robot, &mut memo);
                if robot_pair == target_pair { // We found it!
                    return Some(robot.id);
                }
                // Here we know that the current robot is responsible for comparing `m0` and some
                // other microchip `c != m1`. Since we know both `m0` and `c` values, we can
                // compare them to "follow" the next robots responsible for comparing `m0`.
                robot = match robot {
                    &Robot { low_to: Output::Robot(next_id), .. } if robot_pair.low == m0 => {
                        self.robots.get(&next_id).unwrap()
                    },
                    &Robot { high_to: Output::Robot(next_id), .. } if robot_pair.high == m0 => {
                        self.robots.get(&next_id).unwrap()
                    },
                    _ => return None, // could be that the next "hop" is an output bin
                }
            }
        }

        /// Returns the microchip pair compared by the given `robot`.
        fn compared_microchips(&self, robot: &Robot, memo: &mut HashMap<Id, Microchip2>) -> Microchip2 {
            if memo.contains_key(&robot.id) {
                *memo.get(&robot.id).unwrap()
            } else {
                let pair = Microchip2::new(
                    self.given_microchip(robot.from.0, memo),
                    self.given_microchip(robot.from.1, memo)
                );
                memo.insert(robot.id, pair);
                pair
            }
        }

        /// Returns the microchip that is given by the provided `gift`.
        fn given_microchip(&self, gift: Gift, memo: &mut HashMap<Id, Microchip2>) -> Microchip {
            match gift {
                Gift::Input { chip } => chip, // an input bin, easy.
                Gift::Donation { from_robot_id, weight } => {
                    let donator = self.robots.get(&from_robot_id).unwrap();
                    let donator_pair = self.compared_microchips(donator, memo);
                    match weight {
                        MicrochipWeight::Lower  => donator_pair.low,
                        MicrochipWeight::Higher => donator_pair.high,
                    }
                }
            }
        }
    }
}


use std::io::Read;
use balance_bots::*;

fn main() {
    // acquire data from stdin.
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");

    // parse the instructions, build the factory.
    let instructions: Vec<Instruction> = input.lines().map(|line| line.parse().unwrap()).collect();
    let factory = Factory::build_from(&instructions);

    // part 1
    let (m0, m1) = (Microchip(17), Microchip(61));
    if let Some(id) = factory.robot_comparing(m0, m1) {
        println!("The robot {:?} is responsible for comparing {:?} and {:?}.", id, m0, m1);
    } else {
        println!("Failed to find the robot responsible for comparing {:?} and {:?}.", m0, m1);
    }
}


#[test]
fn part1_example() {
    let input =
        "value 5 goes to bot 2
        bot 2 gives low to bot 1 and high to bot 0
        value 3 goes to bot 1
        bot 1 gives low to output 1 and high to bot 0
        bot 0 gives low to output 2 and high to output 0
        value 2 goes to bot 2".to_string();
    let instructions: Vec<Instruction> = input.lines().map(|line| line.parse().unwrap()).collect();
    let factory = Factory::build_from(&instructions);
    assert_eq!(factory.robot_comparing(Microchip(2), Microchip(5)), Some(2));
}
