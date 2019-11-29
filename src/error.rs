use failure_derive::*;

#[derive(Debug, Fail)]
#[fail(display = "Parse Error: line {} :{}", line, mess)]
pub struct LineError {
    mess: String,
    line: usize,
}

impl LineError {
    pub fn new(s: &str, line: usize) -> Self {
        LineError {
            mess: s.to_string(),
            line,
        }
    }

    pub fn set_line(mut self, n: usize) -> Self {
        self.line = n;
        self
    }
}

#[derive(Debug, Fail, Clone)]
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
