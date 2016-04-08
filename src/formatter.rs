use std::error;
use std::fmt;
use std::result;

#[derive(PartialEq, Debug)]
pub struct Formatter {
    width: usize,
    counter: usize
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            width: 1,
            counter: 1
        }
    }

    pub fn width(&mut self, width: usize) {
        self.width = width;
    }

    pub fn counter(&mut self, counter: usize) {
        self.counter = counter;
    }

    fn inc(&mut self) -> usize {
        let counter = self.counter;
        self.counter += 1;
        counter
    }

    pub fn escape(s: &str) -> String {
        let mut buf = String::new();

        for c in s.chars() {
            buf.push(c);
            if c == '%' {
                buf.push(c);
            }
        }

        buf
    }

    fn pad_left(&self, counter: usize) -> String {
        format!("{0:>01$}", counter, self.width)
    }

    pub fn format(&mut self, s: &str) -> Result {
        let mut buf = String::new();
        let mut iter = s.chars();

        while let Some(c) = iter.next() {
            if c == '%' {
                let spec = try!(iter.next().ok_or(Error::ExpectedSpec));

                match spec {
                    '%' => buf.push(c),
                    'n' => {
                        let counter = self.inc();
                        let counter = self.pad_left(counter);
                        buf.push_str(&counter);
                    },
                    _ => return Err(Error::UnknownSpec(spec))
                }
            } else {
                buf.push(c);
            }
        }

        Ok(buf)
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Error {
    UnknownSpec(char),
    ExpectedSpec
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnknownSpec(c) =>
                try!(write!(fmt, "Unknown conversion specifier: {}", c)),
            Error::ExpectedSpec =>
                try!(write!(fmt, "Expected conversion specifier or `%'"))
        }

        Ok(())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnknownSpec(_) => "Unknown conversion specifier",
            Error::ExpectedSpec => "Expected conversion specifier"
        }
    }
}

pub type Result = result::Result<String, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape() {
        assert_eq!("foo%%bar", Formatter::escape("foo%bar"));
        assert_eq!("foobar", Formatter::escape("foobar"));
    }

    #[test]
    fn round_trip() {
        let mut formatter = Formatter::new();

        assert_eq!(Ok(String::from("foo%bar")), formatter.format(&Formatter::escape("foo%bar")));
    }

    #[test]
    fn format() {
        let mut formatter = Formatter::new();
        let pattern = "foo%n.txt";
        formatter.width(2);

        assert_eq!(Ok(String::from("foo01.txt")), formatter.format(pattern));
        assert_eq!(Ok(String::from("foo02.txt")), formatter.format(pattern));
        assert_eq!(Ok(String::from("foo03.txt")), formatter.format(pattern));

        formatter.counter(1);

        assert_eq!(Ok(String::from("foo01.txt")), formatter.format(pattern));
        assert_eq!(Ok(String::from("foo02.txt")), formatter.format(pattern));
        assert_eq!(Ok(String::from("foo03.txt")), formatter.format(pattern));

        assert_eq!(Err(Error::ExpectedSpec), formatter.format("%"));
        assert_eq!(Err(Error::UnknownSpec('x')), formatter.format("%x"));
    }

    #[test]
    fn padding() {
        let mut formatter = Formatter::new();

        assert_eq!("1", formatter.pad_left(1));
        assert_eq!("2", formatter.pad_left(2));
        assert_eq!("20", formatter.pad_left(20));

        formatter.width(2);

        assert_eq!("01", formatter.pad_left(1));
        assert_eq!("02", formatter.pad_left(2));
        assert_eq!("20", formatter.pad_left(20));
    }
}
