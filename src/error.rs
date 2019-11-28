use failure_derive::*;

#[derive(Debug, Fail)]
#[fail(display = "Parse Error: line {} :{}", line, mess)]
pub struct ParseError {
    mess: String,
    line: usize,
}

impl ParseError {
    pub fn new(s: &str, line: usize) -> Self {
        ParseError {
            mess: s.to_string(),
            line,
        }
    }

    pub fn set_line(mut self, n: usize) -> Self {
        self.line = n;
        self
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Action Error: {}", mess)]
pub struct ActionError {
    mess: String,
}

impl ActionError {
    pub fn new(s: &str) -> Self {
        ActionError {
            mess: s.to_string(),
        }
    }
}
