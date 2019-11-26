use crate::error::ParseError;
use crate::expr::Expr;
use crate::token::{Token, Tokenizer};

#[derive(Debug)]
pub enum Action {
    SetItemType(String),
    SetItem(String),
    AddStat(String, Expr),
    SubStat(String, Expr),
    SetStat(String, Expr),
    AddListItem(String, String),
    RemListItem(String, String),
    GainItem(i32, String),
    LoseItem(i32, String),

    NoAction,
}

pub struct ActionReader<'a> {
    it: Tokenizer<'a>,
    peek: Option<Token>,
}

impl<'a> ActionReader<'a> {
    pub fn new(s: &'a str) -> Self {
        ActionReader {
            it: Tokenizer::new(s),
            peek: None,
        }
    }
}

impl<'a> ActionReader<'a> {
    pub fn read_comment_line(&mut self) -> Result<Action, ParseError> {
        while self.peek != Some(Token::Break) {
            self.peek = self.it.next();
        }
        self.peek = self.it.next();
        Ok(Action::NoAction)
    }
    //return String = svalue, bool is continue to next part
    fn set_item(&mut self) -> Result<Action, ParseError> {
        let s1 = self
            .it
            .next()
            .ok_or(ParseError::new("String short", 0))?
            .as_str_val()?
            .to_string();
        self.peek = self.it.next();
        Ok(Action::SetItem(s1))
    }

    pub fn set_item_type(&mut self) -> Result<Action, ParseError> {
        self.peek.take();
        match self.it.next() {
            None => Err(ParseError::new("Ux-EOF", self.it.line_no)),
            Some(Token::Ident(ref s)) => Ok(Action::SetItemType(s.clone())),
            Some(Token::Qoth(ref s)) => Ok(Action::SetItemType(s.clone())),
            _ => Err(ParseError::new("Expeted Ident", self.it.line_no)),
        }
    }

    pub fn set_property(&mut self) -> Result<Action, ParseError> {
        //rem dot
        self.peek = self.it.next();
        let idstr = self
            .peek
            .as_ref()
            .ok_or(ParseError::new("no property name", self.it.line_no))?
            .as_str_val()?
            .to_string();

        match self.it.next() {
            None => Err(ParseError::new("No property action", self.it.line_no)),
            Some(Token::Equals) => Ok(Action::SetStat(idstr, Expr::from_tokens(&mut self.it)?)),
            Some(Token::Add) => Ok(Action::AddStat(idstr, Expr::from_tokens(&mut self.it)?)),
            Some(Token::Sub) => Ok(Action::SubStat(idstr, Expr::from_tokens(&mut self.it)?)),
            _ => Err(ParseError::new(
                "Not sure what do do with property type",
                self.it.line_no,
            )),
        }
    }
}

impl<'a> Iterator for ActionReader<'a> {
    type Item = Result<Action, ParseError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.peek == None {
            self.peek = self.it.next();
        }
        println!("Areader self.peek == {:?}", self.peek);
        let res = match self.peek {
            None => None,
            Some(Token::Colon) => Some(self.set_item_type()),
            Some(Token::Hash) => Some(self.read_comment_line()),
            Some(Token::Ident(_)) | Some(Token::Qoth(_)) => Some(self.set_item()),
            Some(Token::Dot) => Some(self.set_property()),
            _ => {
                self.peek = self.it.next();
                Some(Err(ParseError::new("err", self.it.line_no)))
            }
        }
        self.after_break();
        res
    }
}
