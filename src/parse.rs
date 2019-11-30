use crate::error::LineError;
use crate::prev_iter::{LineCounter, Prev};
use crate::proto::Proto;
use crate::token::{Token, Tokenizer};
use crate::value::Value;
use crate::action::Action;

#[derive(Debug, Clone)]
pub struct LineAction {
    pub action: Action,
    pub line: usize,
}

impl LineAction {
    pub fn err(&self, s: &str) -> LineError {
        LineError::new(&format!("{:?}:{}", self.action, s), self.line)
    }
}

pub struct ActionReader<'a> {
    it: Prev<Token, Tokenizer<'a>>,
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
            it: Prev::new(Tokenizer::new(s)),
        }
    }
}

impl<'a> ActionReader<'a> {

    pub fn on_add_sub(&mut self) -> Result<Action, LineError> {
        let sign = match self.it.next() {
            Some(Token::Add) => Token::Add,
            Some(Token::Sub) => Token::Sub,
            _ => return Err(self.it.err("Not add or sub")),
        };
        let n = match self.it.next() {
            Some(Token::Num(n)) => n,
            _ => {
                self.it.back();
                1
            }
        };
        let id = self.it.next();
        let id = id
            .ok_or(self.it.err("Not stringable"))?
            .as_str_val()
            .map_err(|p| p.set_line(self.it.line()))?
            .to_string();
        match sign {
            Token::Add => Ok(Action::Add(Proto::one(&id, true), Value::num(n))),
            Token::Sub => Ok(Action::Sub(Proto::one(&id, true), Value::num(n))),
            _ => Err(self.it.err("Not Addable")),
        }
    }

}

impl<'a> Iterator for ActionReader<'a> {
    type Item = Result<LineAction, LineError>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.it.next(){
            None=>return None,
            Some(Token::Hash)=>


        }

        Some(res.map(|action| LineAction {
            action,
            line: self.line(),
        }))
    }
}
