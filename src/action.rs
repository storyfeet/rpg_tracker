use crate::error::LineError;
use crate::expr::Expr;
use crate::proto_ex::ProtoX;
use crate::token::{TokPrev, Token};


#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Select(Option<ProtoX>),
    Add(Expr, Expr),
    Sub(Expr, Expr),
    Set(Expr, Expr),
    Proto(Expr),
    Expr(Expr),
    Declare(Expr, Expr),
}

impl Action {
    pub fn from_add_sub(sign: Token, t: &mut TokPrev) -> Result<Action, LineError> {
        unimplemented!();
    }

    pub fn from_proto(p: ProtoX, t: &mut TokPrev) -> Result<Action, LineError> {
        unimplemented!();
    }

    pub fn from_tokens(t: &mut TokPrev) -> Result<Action, LineError> {
        unimplemented!();
    }

    //if the function should be added to the
    pub fn is_fileworthy(&self) -> bool {
        match self {
            Action::Expr(_) => false,
            _ => true,
        }
    }
}
