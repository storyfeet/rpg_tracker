use crate::expr::{Expr, Op};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Select(Expr),
    OpSet(Op, Expr, Expr),
    Set(Expr, Expr),
    AddItem(i32, String),
    RemItem(i32, String),
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
