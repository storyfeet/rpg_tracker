use crate::prev_iter::{Backer, LineCounter, Prev};
use crate::error::LineError;
use crate::proto::Proto;
use crate::value::Value;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Select(Proto),
    Add(Proto, Value),
    Sub(Proto, Value),
    Set(Proto, Value),
    Func(Proto, Vec<Value>),
}

impl Action {
    pub fn from_tokens<T: Iterator<Item = Token> + LineCounter + Backer>(
        t: &mut T,
    ) -> Result<Self, LineError> {
        let res = match t.next().ok_or(t.err("UxEOF"))? {
            Token::Hash => loop{
                match t.next() {
                    Some(Token::Break) | None => return Action::from_tokens(t),
                    _ => {}
                }
            }
            Token::Dot | Token::Ident(_) | Token::Qoth(_) => {
                t.back();
                Action::on_ident()
            }
            Token::Break => return self.next(),
            Token::Add | Token::Sub => {
                t.back();
                self.on_add_sub()
            }
            Some(t) => Err(self.err(&format!("UX - {:?}", t))),
        };
        return res
    }
    pub fn on_ident<T:Iterator<Item=Token> + LineCounter+Backer>(p:Proto,t:&mut T) -> Result<Self,LineError> {
        match t.next() {
            None | Some(Token::Break) => Ok(Action::Select(p)),
            Some(Token::Equals) => Ok(Action::Set(p, Value::from_tokens(t)?)),
            Some(Token::Add) => Ok(Action::Add(p, Value::from_tokens(t)?)),
            Some(Token::Sub) => Ok(Action::Sub(p, Value::from_tokens(t)?)),
            Some(Token::BOPEN) => {
                let params = Vec::new();
                while let Some(tk) = t.next(){
                    match tk {
                        Token::Comma|Token::Break => {},
                        params.push(Value::from_tokens(t)),
                    }
                }
                
            }
            e => {
                t.back();
                Ok(Action::Select(p)),
            }
                Err(self.err(&format!("Ux - {:?} - after ident", e))),
        }
    }
}
