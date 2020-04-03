use crate::expr::{Expr, Op};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    NoOp,
    Select(Expr),
    SetSelect(Expr, Expr),
    OpSet(Op, Expr, Expr),
    Set(Expr, Expr),
    AddItem(isize, String),
    RemItem(isize, String),
    Return(Expr),
    Display(Expr),
}

impl Action {
    //if the function should be added to the history file?
    pub fn is_fileworthy(&self) -> bool {
        match self {
            Action::Display(_) => false,
            _ => true,
        }
    }
}
