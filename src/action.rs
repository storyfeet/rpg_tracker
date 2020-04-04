use crate::error::ActionError;
use crate::expr::{Expr, Op};
use crate::value::Value;

pub type AcResult = Result<(AcReturn, Value), ActionError>;

pub enum AcReturn {
    No,
    Func,
    //Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    NoOp,
    Select(Expr),
    SetSelect(Expr, Expr),
    OpSet(Op, Expr, Expr),
    Set(Expr, Expr),
    AddItem(isize, String),
    RemItem(isize, String),
    Resolve(Expr),
    Return(Expr),
}
