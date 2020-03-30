use failure_derive::*;
use gobble::err::ParseError;

#[derive(Debug, Fail, PartialEq)]
#[fail(display = "Parse Error: line {} :{}", line, mess)]
pub struct LineError {
    mess: String,
    line: usize,
    pub eof: bool,
}

impl LineError {
    pub fn new(s: &str, line: usize) -> Self {
        LineError {
            mess: s.to_string(),
            line,
            eof: false,
        }
    }

    pub fn eof(line: usize) -> Self {
        LineError {
            mess: "UX - EOF".to_string(),
            line,
            eof: true,
        }
    }

    pub fn set_line(mut self, n: usize) -> Self {
        self.line = n;
        self
    }
}

#[derive(Debug, Fail, Clone, PartialEq)]
#[fail(Display)]
pub enum ActionError {
    #[fail(display = "Parse Error: {}", 0)]
    ParseErr(ParseError),
    #[fail(display = "Action Error: {}", 0)]
    DoingErr(String),
}

impl ActionError {
    pub fn new(s: &str) -> Self {
        ActionError::DoingErr(s.to_string())
    }
}

impl From<ParseError> for ActionError {
    fn from(p: ParseError) -> ActionError {
        ActionError::ParseErr(p)
    }
}
