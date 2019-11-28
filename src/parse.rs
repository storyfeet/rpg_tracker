use crate::error::ParseError;
use crate::prev_iter::{Prev,LineCounter};
use crate::token::{Token, Tokenizer};
use crate::value::Value;

#[derive(Debug, Clone)]
pub enum Action {
    SetItem(String),
    AddStat(String, Value),
    SubStat(String, Value),
    SetStat(String, Value),
    GainItem(String, i32),
}

pub struct ActionReader<'a> {
    it: Prev<Token, Tokenizer<'a>>,
}

impl<'a> LineCounter for ActionReader<'a>{
    fn line(&self)->usize{
        self.it.line()
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
    fn gain_item(&mut self, mul: i32) -> Result<Action, ParseError> {
        let n = match self.it.next() {
            None => return Err(self.err("UX-eof")),
            Some(Token::Qoth(s)) | Some(Token::Ident(s)) => return Ok(Action::GainItem(s, mul)),
            Some(Token::Num(n)) => n,
            x => return Err(self.err( &format!("{:?} cannot be gained", x))),
        };
        let iname = self
            .it
            .next()
            .ok_or(ParseError::new("No item to gain", 0))?
            .as_str_val()
            .map_err(|p| p.set_line(self.line()))?
            .to_string();

        Ok(Action::GainItem(iname, n * mul))
    }

    pub fn set_property(&mut self) -> Result<Action, ParseError> {
        let idstr = self
            .it
            .next()
            .ok_or(self.err("no property name"))?
            .as_str_val()?
            .to_string();

        match self.it.next() {
            None => Err(self.err("No property action")),
            Some(Token::Equals) => Ok(Action::SetStat(idstr, Value::from_tokens(&mut self.it)?)),
            Some(Token::Add) => Ok(Action::AddStat(idstr, Value::from_tokens(&mut self.it)?)),
            Some(Token::Sub) => Ok(Action::SubStat(idstr, Value::from_tokens(&mut self.it)?)),
            _ => Err(self.err( "Not sure what do do with property type")),
        }
    }
}

impl<'a> Iterator for ActionReader<'a> {
    type Item = Result<Action, ParseError>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.it.next() {
            None => None,
            Some(Token::Hash) => {
                Some(self.read_to_break());
                return self.next();
            }
            Some(Token::Ident(s)) | Some(Token::Qoth(s)) => Some(Ok(Action::SetItem(s))),
            Some(Token::Dot) => Some(self.set_property()),
            Some(Token::Break) => return self.next(),
            Some(Token::Add) => Some(self.gain_item(1)),
            Some(Token::Sub) => Some(self.gain_item(-1)),
            Some(t) => Some(Err(self.err(&format!("UX - {:?}",t)))),
        };
        res
    }
}
