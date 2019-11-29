use crate::error::LineError;
use crate::prev_iter::{LineCounter, Prev};
use crate::proto::Proto;
use crate::token::{Token, Tokenizer};
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct LineAction {
    pub action: Action,
    pub line: usize,
}

impl LineAction {
    pub fn err(&self,s:&str)->LineError{
        LineError::new(&format!("{:?}:{}",self.action,s),self.line)
    }
}

#[derive(Debug, Clone,PartialEq)]
pub enum Action {
    Select(Proto),
    Add(Proto, Value),
    Sub(Proto, Value),
    Set(Proto, Value),
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
    pub fn read_to_break(&mut self) {
        loop {
            match self.it.next() {
                Some(Token::Break) | None => return,
                _ => {}
            }
        }
    }

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

    pub fn on_ident(&mut self) -> Result<Action, LineError> {
        let p = Proto::from_tokens(&mut self.it);
        match self.it.next() {
            None | Some(Token::Break) => Ok(Action::Select(p)),
            Some(Token::Equals) => Ok(Action::Set(p, Value::from_tokens(&mut self.it)?)),
            Some(Token::Add) => Ok(Action::Add(p, Value::from_tokens(&mut self.it)?)),
            Some(Token::Sub) => Ok(Action::Sub(p, Value::from_tokens(&mut self.it)?)),
            e => Err(self.err(&format!("Ux - {:?} - after ident", e))),
        }
    }
}

impl<'a> Iterator for ActionReader<'a> {
    type Item = Result<LineAction, LineError>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.it.next() {
            None => return None,
            Some(Token::Hash) => {
                self.read_to_break();
                return self.next();
            }
            Some(Token::Dot) | Some(Token::Ident(_)) | Some(Token::Qoth(_)) => {
                self.it.back();
                self.on_ident()
            }
            Some(Token::Break) => return self.next(),
            Some(Token::Add) | Some(Token::Sub) => {
                self.it.back();
                self.on_add_sub()
            }
            Some(t) => Err(self.err(&format!("UX - {:?}", t))),
        };
        Some(res.map(|action| LineAction {
            action,
            line: self.line(),
        }))
    }
}
