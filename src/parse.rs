use crate::error::ParseError;
use crate::token::{Token, Tokenizer};

pub enum Action {
    SetItemType(String),
    SetItem(String),
    AddStat(String, i32),
    SetStat(String, i32),

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
        self.peek = self.it.next();
        match self.peek {
            None => Err(ParseError::new("Ux-EOF", self.it.line_no)),
            Some(Token::Ident(ref s)) => Ok(Action::SetItemType(s.clone())),
            Some(Token::Qoth(ref s)) => Ok(Action::SetItemType(s.clone())),
            _ => Err(ParseError::new("Expeted Ident", self.it.line_no)),
        }
    }

    pub fn set_property(&mut self) -> Result<Action, ParseError> {
        //TODO 
        return Ok(Action::SetItem("H".to_string()))
    }
}

impl<'a> Iterator for ActionReader<'a> {
    type Item = Result<Action, ParseError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.peek == None {
            self.peek = self.it.next();
        }
        match self.peek {
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
    }
}
