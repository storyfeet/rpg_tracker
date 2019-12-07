use crate::action::Action;
use crate::error::LineError;
use crate::prev_iter::LineCounter;
use crate::token::TokPrev;

#[derive(Debug, Clone)]
pub struct LineAction {
    pub action: Action,
    pub line: usize,
}

pub struct ActionReader<'a> {
    it: TokPrev<'a>,
}

impl<'a> LineCounter for ActionReader<'a> {
    fn line(&self) -> usize {
        let res = self.it.line();
        //println!("AR - line {}",res);
        res
    }
}

impl<'a> ActionReader<'a> {
    pub fn new(s: &'a str) -> Self {
        ActionReader {
            it: TokPrev::new(s),
        }
    }

    pub fn line_acc(&self, action: Action) -> LineAction {
        LineAction {
            line: self.line(),
            action,
        }
    }
}
impl<'a> Iterator for ActionReader<'a> {
    type Item = Result<LineAction, LineError>;
    fn next(&mut self) -> Option<Self::Item> {
        match Action::from_tokens(&mut self.it) {
            Ok(v) => Some(Ok(self.line_acc(v))),
            Err(e) => {
                if e.eof {
                    None
                } else {
                    Some(Err(e))
                }
            }
        }
    }
}
