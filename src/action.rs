use crate::error::LineError;
use crate::expr::Expr;
use crate::prev_iter::{Backer, LineCounter};
use crate::proto::Proto;
use crate::token::{TokPrev, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Select(Option<Proto>),
    Add(Proto, Expr),
    Sub(Proto, Expr),
    Set(Proto, Expr),
    CallFunc(Proto, Vec<Expr>),
    Expr(Expr),
}

impl Action {
    pub fn from_add_sub(sign: Token, t: &mut TokPrev) -> Result<Action, LineError> {
        let n = match t.next() {
            Some(Token::Num(n)) => n,
            _ => {
                t.back();
                1
            }
        };
        let id = t.next();
        let id = id
            .ok_or(t.err("Not stringable"))?
            .as_str_val()
            .map_err(|p| p.set_line(t.line()))?
            .to_string();
        match sign {
            Token::Add => Ok(Action::Add(Proto::one(&id, 1), Expr::num(n))),
            Token::Sub => Ok(Action::Sub(Proto::one(&id, 1), Expr::num(n))),
            _ => Err(t.err("Not Addable")),
        }
    }

    pub fn from_proto(p: Proto, t: &mut TokPrev) -> Result<Action, LineError> {
        match t.next() {
            None | Some(Token::Break) => Ok(Action::Expr(Expr::proto(p.with_deref(1)))),
            Some(Token::Colon) => Ok(Action::Select(Some(p))),
            Some(Token::Equals) => Ok(Action::Set(p, Expr::from_tokens(t)?)),
            Some(Token::Add) => Ok(Action::Add(p, Expr::from_tokens(t)?)),
            Some(Token::Sub) => Ok(Action::Sub(p, Expr::from_tokens(t)?)),
            Some(Token::BracketO) => {
                let mut params = Vec::new();
                while let Some(tk) = t.next() {
                    match tk {
                        Token::Comma => {}
                        Token::BracketC => return Ok(Action::CallFunc(p, params)),
                        _ => {
                            t.back();
                            params.push(Expr::from_tokens(t)?);
                        }
                    }
                }
                return Err(t.eof());
            }
            e => Err(t.ux(e, "after ident")),
        }
    }

    pub fn from_tokens(t: &mut TokPrev) -> Result<Action, LineError> {
        match t.next().ok_or(t.eof())? {
            Token::Hash => {
                t.read_to_break();
                return Self::from_tokens(t);
            }
            Token::Colon => Ok(Action::Select(None)),
            Token::Dollar | Token::Dot | Token::Ident(_) | Token::Qoth(_) => {
                t.back();
                let p = Proto::from_tokens(t);
                //println!("PROTO from_tokens= {:?}",p);
                Self::from_proto(p, t)
            }
            Token::Break => Self::from_tokens(t),
            Token::Add => Self::from_add_sub(Token::Add, t),
            Token::Sub => Self::from_add_sub(Token::Sub, t),
            _ => {
                t.back();
                Ok(Action::Expr(Expr::from_tokens(t)?))
            }
        }
    }

    //if the function should be added to the
    pub fn is_fileworthy(&self) -> bool {
        match self {
            Action::Expr(_) => false,
            _ => true,
        }
    }
}
