use failure_derive::*;

#[derive(Debug, Fail)]
#[fail(display = "Parse Error: line {} :{}", line, mess)]
pub struct ParseError {
    mess: String,
    line: i32,
}

impl ParseError {
    pub fn new(s: &str, line: i32) -> Self {
        ParseError {
            mess: s.to_string(),
            line,
        }
    }
}
