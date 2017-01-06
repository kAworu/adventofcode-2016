#[macro_use]
extern crate lazy_static;
extern crate regex;

mod two_factor_authentication {
    use regex::Regex;
    use ::std::fmt::Display;
    use ::std::str::FromStr;

    /// Represent a `Screen` operation.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Operation {
        Rect(u32, u32),
        RotateRow(u32, u32),
        RotateCol(u32, u32),
    }

    impl FromStr for Operation {
        type Err = String;

        fn from_str(s: &str) -> Result<Operation, String> {
            lazy_static! {
                static ref RECT: Regex = Regex::new(r"^rect (?P<A>\d+)x(?P<B>\d+)$").unwrap();
                static ref ROTR: Regex = Regex::new(r"^rotate row y=(?P<A>\d+) by (?P<B>\d+)$").unwrap();
                static ref ROTC: Regex = Regex::new(r"^rotate column x=(?P<A>\d+) by (?P<B>\d+)$").unwrap();
            }
            if let Some(caps) = RECT.captures(s) {
                Ok(Operation::Rect(caps["A"].parse().unwrap(), caps["B"].parse().unwrap()))
            } else if let Some(caps) = ROTR.captures(s) {
                Ok(Operation::RotateRow(caps["A"].parse().unwrap(), caps["B"].parse().unwrap()))
            } else if let Some(caps) = ROTC.captures(s) {
                Ok(Operation::RotateCol(caps["A"].parse().unwrap(), caps["B"].parse().unwrap()))
            } else {
                Err(format!("unrecognized operation: {}", s))
            }
        }
    }

    /// Represent a Pixel state: either lit or not, `On` respectively `Off`.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum PixelState {
        On,
        Off,
    }

    /// Represent a pixel on the `Sreen`. `true` if the pixel is lit, `false` otherwise.
    #[derive(Copy, Clone, Debug)]
    struct Pixel {
        state: PixelState,
    }

    impl Pixel {
        /// Create a new pixel in "off" state, i.e. not lit.
        fn off() -> Pixel {
            Pixel { state: PixelState::Off }
        }

        /// Turn a pixel "on".
        fn turn_on(&mut self) {
            self.state = PixelState::On;
        }

        /// Returns `true` if self is lit, `false` otherwise.
        fn is_on(&self) -> bool {
            self.state == PixelState::On
        }
    }

    impl Display for Pixel {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "{}", if self.is_on() { '#' } else { '.' })
        }
    }

    /// Represent A little smashable screen.
    pub struct Screen {
        width: usize,
        height: usize,
        pixels: Vec<Pixel>,
    }

    impl Screen {
        /// Create a new blank `Screen` of given dimensions, with all pixels off.
        pub fn blank(width: usize, height: usize) -> Screen {
            Screen {
                width: width,
                height: height,
                pixels: vec![Pixel::off(); width * height],
            }
        }

        /// Execute the given `Operation`. Returns `true` on success, `false` otherwise.
        pub fn execute(&mut self, op: Operation) -> bool {
            match op {
                Operation::Rect(width, height)   => self.rect(width as usize, height as usize),
                Operation::RotateRow(y, xoffset) => self.rotate_row(y as usize, xoffset as usize),
                Operation::RotateCol(x, yoffset) => self.rotate_col(x as usize, yoffset as usize),
            }
        }

        /// Returns the voltage used by `self`, i.e. the count of pixel lit.
        pub fn voltage_usage(&self) -> usize {
            self.pixels.iter().filter(|&px| px.is_on()).count()
        }

        /// > turns on all of the pixels in a rectangle at the top-left of the screen which is `A`
        /// > wide and `B` tall.
        fn rect(&mut self, /* A */ width: usize, /* B */ height: usize) -> bool {
            if width > self.width || height > self.height {
                return false;
            }
            for y in 0..height {
                for x in 0..width {
                    self.pixel_at_mut(x, y).turn_on();
                }
            }
            true
        }

        /// > shifts all of the pixels in row `A` (`0` is the top row) right by `B` pixels. Pixels
        /// > that would fall off the right end appear at the left end of the row.
        // NOTE: the typical smashed screen is significantly wider than tall. Our
        // representation allow an efficient rotate_row operation with three memcpy().
        fn rotate_row(&mut self, /* A */ y: usize, /* B */ xoffset: usize) -> bool {
            let (width, height) = (self.width, self.height);
            if y >= height || xoffset >= width {
                return false;
            }
            let (row_start, row_end) = (y * width, (y + 1) * width);
            let mut buf = vec![Pixel::off(); width];
            // 1. copy the full row into buf
            buf.copy_from_slice(&self.pixels[row_start..row_end]);
            // 2. copy the first pixels until the first "shifted" one (not included) at their new
            //    positions.
            self.pixels[(row_start + xoffset)..row_end].copy_from_slice(&buf[0..(width - xoffset)]);
            // 3. copy into our first pixels all the shifted pixels.
            self.pixels[row_start..(row_start + xoffset)].copy_from_slice(&buf[(width - xoffset)..width]);
            true
        }

        /// > shifts all of the pixels in column `A` (`0` is the left column) down by `B` pixels.
        /// > Pixels that would fall off the bottom appear at the top of the column.
        // NOTE: the typical smashed screen is significantly wider than tall. Our rotate_col
        // implementation is naive but that's ok since height is small.
        fn rotate_col(&mut self, /* A */ x: usize, /* B */ yoffset: usize) -> bool {
            let (width, height) = (self.width, self.height);
            if x >= width || yoffset >= height {
                return false;
            }
            let mut col = vec![Pixel::off(); height];
            for y in 0..height {
                col[y] = *self.pixel_at(x, y);
            }
            for y in 0..height {
                *self.pixel_at_mut(x, (y + yoffset) % height) = col[y];
            }
            true
        }

        /// Get a reference to the `Pixel` at the given (x, y) position. Panic if either `x` or `y`
        /// is out of range.
        fn pixel_at(&self, x: usize, y: usize) -> &Pixel {
            let index = self.width * y + x;
            self.pixels.get(index).unwrap()
        }

        /// Get a mutable reference to the `Pixel` at the given (x, y) position. Panic if either
        /// `x` or `y` is out of rance.
        fn pixel_at_mut(&mut self, x: usize, y: usize) -> &mut Pixel {
            let index = self.width * y + x;
            self.pixels.get_mut(index).unwrap()
        }
    }

    impl Display for Screen {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            for y in 0..self.height {
                for x in 0..self.width {
                    write!(f, "{}", self.pixel_at(x, y))?;
                }
                write!(f, "\n")?;
            }
            Ok(())
        }
    }
}


use std::io::Read;
use two_factor_authentication::*;

fn main() {
    // acquire data from stdin.
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut input).expect("no input given");

    // Parse one `Operation` per line of input.
    let operations: Vec<Operation> = input.lines().map(|line| line.parse().unwrap()).collect();

    // screen initialization and operations.
    let mut screen = Screen::blank(50, 6);
    for &operation in operations.iter() {
        screen.execute(operation);
    }

    // print the screen display and voltage usage.
    println!("{}", screen);
    println!("The screen's voltage usage is: {}", screen.voltage_usage());
}


#[test]
fn part1_example() {
    let mut screen = Screen::blank(7, 3);
    let op: Operation = "rect 3x2".parse().unwrap();
    assert_eq!(op, Operation::Rect(3, 2));
    screen.execute(op);
    assert_eq!(screen.to_string(), "\
###....
###....
.......
");
    let op: Operation = "rotate column x=1 by 1".parse().unwrap();
    assert_eq!(op, Operation::RotateCol(1, 1));
    screen.execute(op);
    assert_eq!(screen.to_string(), "\
#.#....
###....
.#.....
");
    let op: Operation = "rotate row y=0 by 4".parse().unwrap();
    assert_eq!(op, Operation::RotateRow(0, 4));
    screen.execute(op);
    assert_eq!(screen.to_string(), "\
....#.#
###....
.#.....
");
    let op: Operation = "rotate column x=1 by 1".parse().unwrap();
    assert_eq!(op, Operation::RotateCol(1, 1));
    screen.execute(op);
    assert_eq!(screen.to_string(), "\
.#..#.#
#.#....
.#.....
");
    assert_eq!(screen.voltage_usage(), 6);
}
