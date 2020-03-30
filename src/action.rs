use crate::expr::{Expr, Op};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Select(Expr),
    Display(Expr),
    OpSet(Op, Expr, Expr),
    Set(Expr, Expr),
    Declare(Expr, Expr),
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
