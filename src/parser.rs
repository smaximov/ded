use std::cmp::{Ordering};
use std::error;
use std::fmt;
use std::result;

#[derive(PartialEq, Eq, Debug)]
pub struct Error {
    desc: String,
    pos: Position,
    inner: Option<Box<Error>>
}

impl Error {
    pub fn new(desc: &str, pos: Position) -> Self {
        Error {
            desc: String::from(desc),
            pos: pos,
            inner: None
        }
    }

    pub fn new_wrap(desc: &str, pos: Position, inner: Self) -> Self {
        Error {
            desc: String::from(desc),
            pos: pos,
            inner: Some(Box::new(inner))
        }
    }
}

fn fmt_error(err: &Error, fmt: &mut fmt::Formatter) -> fmt::Result {
    try!(write!(fmt, "at {}:{}: {}", err.pos.line, err.pos.col, err.desc));

    if let Some(ref inner) = err.inner {
        try!(write!(fmt, ", "));
        try!(fmt_error(inner, fmt));
    }

    Ok(())
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "Parse errors: ["));
        try!(fmt_error(self, fmt));
        try!(write!(fmt, "]"));

        Ok(())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Parser error"
    }
}

pub type Result<R> = result::Result<R, Error>;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Transform {
    Rename {
        hash_fragment: String,
        pattern: String
    },
    Remove {
        hash_fragment: String
    }
}

impl Transform {
    pub fn rename(hash_fragment: String, pattern: String) -> Self {
        Transform::Rename {
            hash_fragment: hash_fragment,
            pattern: pattern
        }
    }

    pub fn remove(hash_fragment: String) -> Self {
        Transform::Remove {
            hash_fragment: hash_fragment
        }
    }

    pub fn hash_fragment(&self) -> &str {
        match *self {
            Transform::Rename { ref hash_fragment, .. } => hash_fragment,
            Transform::Remove { ref hash_fragment, .. } => hash_fragment
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Position {
    offset: usize,
    line: usize,
    col: usize
}

impl Position {
    fn new() -> Self {
        Position {
            offset: 0,
            line: 1,
            col: 1
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.offset.partial_cmp(&other.offset)
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

#[derive(Debug)]
pub struct Parser {
    input: Vec<char>,
    pos: Position
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Parser {
            input: input.chars().collect(),
            pos: Position::new()
        }
    }

    pub fn reset(&mut self, input: &str) {
        self.input = input.chars().collect();
        self.pos = Position::new();
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos.offset).map(|c| *c)
    }

    pub fn position(&self) -> Position {
        self.pos.clone()
    }

    pub fn rest_input(&self) -> String {
        self.input[self.pos.offset..].iter().map(|c| *c).collect()
    }

    fn next(&mut self) -> Option<char> {
        self.peek().map(|c| {
            self.pos.offset += 1;

            if  c == '\n' {
                self.pos.line += 1;
                self.pos.col = 1;
            } else {
                self.pos.col += 1;
            }

            c
        })
    }

    fn satisfy<F>(&mut self, predicate: F) -> Result<char>
        where F: FnOnce(char) -> bool{
        let pos = self.position();
        if let Some(c) = self.next() {
            if predicate(c) {
                Ok(c)
            } else {
                Err(Error::new("doesn't satisfy a predicate", pos))
            }
        } else {
            Err(Error::new("unexpected end of input", pos))
        }
    }

    fn one_of(&mut self, set: &str) -> Result<char> {
        let pos = self.position();
        self.satisfy(|c| set.contains(c)).map_err(|e| {
            Error::new_wrap(&format!("expected on of {:?}", set), pos, e)
        })
    }

    fn hex_digit(&mut self) -> Result<char> {
        let pos = self.position();
        self.one_of("0123456789abcdef").map_err(|e| {
            Error::new_wrap("expected hex digit", pos, e)
        })
    }

    fn try_parser<F, U>(&mut self, parser: &F) -> Option<U>
        where F: Fn(&mut Parser) -> Result<U> {
        let pos = self.position();
        if let Ok(result) = parser(self) {
            Some(result)
        } else {
            self.pos = pos;
            None
        }
    }

    fn many0<F, U>(&mut self, parser: &F) -> Vec<U>
        where F: Fn(&mut Parser) -> Result<U> {
        let mut buf = Vec::new();
        while let Some(result) = self.try_parser(parser) {
            buf.push(result);
        }
        buf
    }

    fn many1<F, U>(&mut self, parser: &F) -> Result<Vec<U>>
        where F: Fn(&mut Parser) -> Result<U>, U: Clone {
        parser(self).map(|first| {
            let mut buf = Vec::new();
            buf.push(first);
            let rest = self.many0(parser);
            buf.extend_from_slice(&rest);
            buf
        })
    }

    fn hex_string(&mut self) -> Result<String> {
        self.many1(&Parser::hex_digit).map(|vec| vec.into_iter().collect())
    }

    fn char(&mut self, c: char) -> Result<char> {
        let pos = self.position();
        self.satisfy(|o| o == c).map_err(|e| {
            Error::new_wrap(&format!("expected character {:?}", c), pos, e)
        })
    }

    fn eof(&mut self) -> Result<()> {
        if self.pos.offset >= self.input.len() {
            Ok(())
        } else {
            Err(Error::new("expected end of input", self.position()))
        }
    }

    fn newline(&mut self) -> Result<()> {
        self.char('\n').map(|_| ())
    }

    fn either<F, G, U>(&mut self, this: &F, that: &G) -> Result<U>
        where F: Fn(&mut Parser) -> Result<U>,
              G: Fn(&mut Parser) -> Result<U> {
        if let Some(result) = self.try_parser(this) {
            Ok(result)
        } else {
            that(self)
        }
    }

    fn line_ending(&mut self) -> Result<()> {
        self.either(&Parser::eof, &Parser::newline)
    }

    fn take_until_consume<F, U>(&mut self, parser: &F) -> String
        where F: Fn(&mut Parser) -> Result<U> {
        let mut buf = String::new();

        while let Some(c) = self.peek() {
            if self.try_parser(parser).is_some() {
                break;
            }

            buf.push(c);
            self.next();
        }

        buf
    }

    fn ignore_many0<F, U>(&mut self, parser: &F)
        where F: Fn(&mut Parser) -> Result<U> {
        while let Some(_) = self.try_parser(parser) {
            // empty body
        }
    }

    fn ignore_many1<F, U>(&mut self, parser: &F) -> Result<()>
        where F: Fn(&mut Parser) -> Result<U> {
        try!(parser(self));
        self.ignore_many0(parser);
        Ok(())
    }

    fn comment(&mut self) -> Result<()> {
        try!(self.char('#'));
        self.take_until_consume(&Parser::line_ending);
        Ok(())
    }

    fn space(&mut self) -> Result<()> {
        self.ignore_many1(&|x: &mut Parser| x.one_of(" \t\n"))
    }

    fn whitespace(&mut self) -> Result<()> {
        self.either(&Parser::comment, &Parser::space)
    }

    fn ignore_whitespace(&mut self) {
        self.ignore_many0(&Parser::whitespace);
    }

    fn transform(&mut self) -> Result<Transform> {
        self.ignore_whitespace();
        let hash_fragment = try!(self.hex_string());

        let space = self.try_parser(&|x: &mut Parser| {
            x.ignore_many1(&|x: &mut Parser| x.one_of(" \t"))
        });

        if space.is_none() {
            let pos = self.position();
            return self.line_ending()
                .map(|_| Transform::remove(hash_fragment))
                .map_err(|e| Error::new_wrap("expected pattern", pos, e));
        }

        let pattern = self.take_until_consume(&Parser::line_ending);

        Ok(match pattern.trim() {
            "" => Transform::remove(hash_fragment),
            pattern => Transform::rename(hash_fragment, String::from(pattern))
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Transform>> {
        let transforms = self.many0(&Parser::transform);
        self.ignore_whitespace();
        let pos = self.position();
        try!(self.eof().map_err(|e| {
            Error::new_wrap("expected hash string", pos, e)
        }));
        Ok(transforms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next() {
        let mut parser = Parser::new("");

        assert_eq!(None, parser.next());

        parser.reset("fo\nba");

        assert_eq!(Position { offset: 0, line: 1, col: 1 }, parser.pos);
        assert_eq!(Some('f'), parser.next());
        assert_eq!(Position { offset: 1, line: 1, col: 2 }, parser.pos);
        assert_eq!(Some('o'), parser.next());
        assert_eq!(Position { offset: 2, line: 1, col: 3 }, parser.pos);
        assert_eq!(Some('\n'), parser.next());
        assert_eq!(Position { offset: 3, line: 2, col: 1 }, parser.pos);
        assert_eq!(Some('b'), parser.next());
        assert_eq!(Position { offset: 4, line: 2, col: 2 }, parser.pos);
        assert_eq!(Some('a'), parser.next());
        assert_eq!(Position { offset: 5, line: 2, col: 3 }, parser.pos);
        assert_eq!(None, parser.next());
    }

    #[test]
    fn peek() {
        let mut parser = Parser::new("");
        assert_eq!(None, parser.peek());
        parser.reset("f");
        assert_eq!(Some('f'), parser.peek());
    }

    #[test]
    fn satisfy() {
        let mut parser = Parser::new("f");
        assert_eq!(Ok('f'), parser.satisfy(|c| c == 'f'));
        assert!(parser.satisfy(|c| c == 'x').is_err());
        parser.next();
        assert!(parser.satisfy(|c| c == 'f').is_err());
    }

    #[test]
    fn one_of() {
        let mut parser = Parser::new("x");
        let pattern = "xyz";

        assert!(parser.one_of(pattern).is_ok());
        parser.reset("y");
        assert!(parser.one_of(pattern).is_ok());
        parser.reset("u");
        assert!(parser.one_of(pattern).is_err());
    }

    #[test]
    fn hex_digit() {
        let mut parser = Parser::new("ab9h");
        assert_eq!(Ok('a'), parser.hex_digit());
        assert_eq!(Ok('b'), parser.hex_digit());
        assert_eq!(Ok('9'), parser.hex_digit());
        assert!(parser.hex_digit().is_err());
    }

    #[test]
    fn many0() {
        let mut parser = Parser::new("abc012");
        let pattern = "abc";
        assert_eq!(vec!['a', 'b', 'c'], parser.many0(&|x: &mut Parser| x.one_of(pattern)));
        assert_eq!(0, parser.many0(&|x: &mut Parser| x.one_of(pattern)).len());
        assert_eq!("012", parser.rest_input());
    }

    #[test]
    fn many1() {
        let mut parser = Parser::new("abc012");
        let pattern = "abc";
        assert_eq!(Ok(vec!['a', 'b', 'c']), parser.many1(&|x: &mut Parser| x.one_of(pattern)));
        assert_eq!("012", parser.rest_input());
        parser.reset("012");
        assert!(parser.many1(&|x: &mut Parser| x.one_of(pattern)).is_err())
    }

    #[test]
    fn hex_string() {
        let hex = "b488a4ca";
        let mut parser = Parser::new(hex);
        assert_eq!(Ok(String::from(hex)), parser.hex_string());
        assert_eq!("", parser.rest_input());
        parser.reset(" deadbeef");
        assert!(parser.hex_digit().is_err());
    }

    #[test]
    fn char() {
        let mut parser = Parser::new("foo");
        assert_eq!(Ok('f'), parser.char('f'));
        assert_eq!(Ok('o'), parser.char('o'));
        assert!(parser.char('f').is_err());
    }

    #[test]
    fn eof() {
        let mut parser = Parser::new("");
        assert!(parser.eof().is_ok());
        parser.reset("\n");
        assert!(parser.eof().is_err());
    }

    #[test]
    fn newline() {
        let mut parser = Parser::new("");
        assert!(parser.newline().is_err());
        parser.reset("\n");
        assert!(parser.newline().is_ok());
        assert!(parser.eof().is_ok());
    }

    #[test]
    fn either() {
        let mut parser = Parser::new(" ");
        assert_eq!(Ok(' '), parser.either(&|x: &mut Parser| x.char(' '),
                                          &|x: &mut Parser| x.hex_digit()));
        parser.reset("d");
        assert_eq!(Ok('d'), parser.either(&|x: &mut Parser| x.char(' '),
                                          &|x: &mut Parser| x.hex_digit()));
        parser.reset("A");
        assert!(parser.either(&|x: &mut Parser| x.char(' '),
                              &|x: &mut Parser| x.hex_digit()).is_err());
    }

    #[test]
    fn line_ending() {
        let mut parser = Parser::new("");
        assert!(parser.line_ending().is_ok());
        parser.reset("\n ");
        assert!(parser.line_ending().is_ok());
        assert!(parser.line_ending().is_err());
    }

    #[test]
    fn take_until_consume() {
        let mut parser = Parser::new("  deadbeef");
        assert_eq!("  ", parser.take_until_consume(&Parser::hex_digit));
        assert_eq!(Ok(String::from("eadbeef")), parser.hex_string());
        parser.reset("  ");
        assert_eq!("  ", parser.take_until_consume(&Parser::hex_digit));
    }

    #[test]
    fn ignore_many() {
        let mut parser = Parser::new("deadbeef");
        parser.ignore_many0(&Parser::line_ending);
        assert_eq!("deadbeef", parser.rest_input());
        parser.ignore_many0(&Parser::hex_digit);
        assert!(parser.eof().is_ok());
    }

    #[test]
    fn comment() {
        let mut parser = Parser::new("# foobar\n\
                                      # foobar");
        assert!(parser.comment().is_ok());
        assert!(parser.comment().is_ok());
        assert!(parser.eof().is_ok());
        parser.reset(" ");
        assert!(parser.comment().is_err());
    }

    #[test]
    fn space() {
        let mut parser = Parser::new("");
        assert!(parser.space().is_err());
        parser.reset(" \t");
        assert!(parser.space().is_ok());
        assert!(parser.eof().is_ok());
    }

    #[test]
    fn whitespace() {
        let mut parser = Parser::new("   #  this is a comment\n\
                                      \t   \n\
                                      # ^ this is an empty line\n\
                                      deadbeef");
        assert!(parser.whitespace().is_ok()); // consume whitespace before the first comment
        assert!(parser.whitespace().is_ok()); // consume the first comment
        assert!(parser.whitespace().is_ok()); // consume the empty line
        assert!(parser.whitespace().is_ok()); // consume the second comment
        assert!(parser.try_parser(&Parser::whitespace).is_none());
        assert_eq!("deadbeef", parser.rest_input());
    }

    #[test]
    fn ignore_whitespace() {
        let mut parser = Parser::new("   #  this is a comment\n\
                                      \t   \n\
                                      # ^ this is an empty line\n\
                                         deadbeef");
        parser.ignore_whitespace();
        assert_eq!("deadbeef", parser.rest_input());
        parser.ignore_whitespace();
        assert_eq!("deadbeef", parser.rest_input());
    }

    #[test]
    fn transform() {
        let mut parser = Parser::new("   #  this is a comment\n\
                                      \t   \n\
                                      # ^ this is an empty line\n\
                                      deadbeef /etc/secret");
        assert_eq!(Ok(Transform::Rename { hash_fragment: String::from("deadbeef"),
                                          pattern: String::from("/etc/secret") }),
                   parser.transform());
        assert!(parser.eof().is_ok());

        parser.reset("#  this is a comment\n\
                      \t   \n\
                      # ^ this is an empty line\n\
                      deadbeef ");
        assert_eq!(Ok(Transform::Remove { hash_fragment: String::from("deadbeef")}), parser.transform());
        assert!(parser.eof().is_ok());

        parser.reset("#  this is a comment\n\
                      \t   \n\
                      # ^ this is an empty line\n\
                      deadbeef");
        assert_eq!(Ok(Transform::Remove { hash_fragment: String::from("deadbeef")}), parser.transform());
        assert!(parser.eof().is_ok());
    }
}
