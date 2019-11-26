use crate::error::ParseError;
use crate::expr::Expr;
use crate::token::{Token, Tokenizer};
use crate::dndata::Value;

#[derive(Debug)]
pub enum Action {
    SetItemType(String),
    SetItem(String),
    AddStat(String, Value),
    SetStat(String, Value),
    AddListItem(String, String),
    RemListItem(String, String),
    GainItem(String,i32),

    NoAction,
}

pub struct ActionReader<'a> {
    it: Tokenizer<'a>,
}

impl<'a> ActionReader<'a> {
    pub fn new(s: &'a str) -> Self {
        ActionReader {
            it: Tokenizer::new(s),
        }
    }
}

impl<'a> ActionReader<'a> {
    pub fn read_to_break(&mut self){
        loop{
            match self.it.next() {
                Some(Token::Break)|None => return,
                _ =>{}
            }

        }
    }
    //return String = svalue, bool is continue to next part
    fn set_item(&mut self) -> Result<Action, ParseError> {
        
        let s1 = self.it.previous()
            .ok_or(ParseError::new("String short", 0))?
            .as_str_val().map_err(|p|p.set_line(self.it.line_no))?
            .to_string();
        Ok(Action::SetItem(s1))
    }

    pub fn set_item_type(&mut self) -> Result<Action, ParseError> {
        match self.it.next() {
            None => Err(ParseError::new("Ux-EOF", self.it.line_no)),
            Some(Token::Ident(ref s)) => Ok(Action::SetItemType(s.clone())),
            Some(Token::Qoth(ref s)) => Ok(Action::SetItemType(s.clone())),
            _ => Err(ParseError::new("Expeted Ident", self.it.line_no)),
        }
    }

    fn gain_item(&mut self,mul:i32) ->Result<Action,ParseError>{
        let n = match self.it.next(){
            None=>return Err( ParseError::new("UX-eof",self.it.line_no)),
            Some(Token::Qoth(s))|Some(Token::Ident(s))=>return Ok(Action::GainItem(s,mul)),
            Some(Token::Num(n))=>n,
            x=> return Err( ParseError::new(&format!("{:?} cannot be gained",x),self.it.line_no))
        };
        let iname = self.it.next()
            .ok_or(ParseError::new("No item to gain", 0))?
            .as_str_val().map_err(|p|p.set_line(self.it.line_no))?
            .to_string();

        Ok(Action::GainItem(iname,n*mul))
    }
    

    pub fn set_property(&mut self) -> Result<Action, ParseError> {
        //rem dot
        let idstr = self.it.next()
            .ok_or(ParseError::new("no property name", self.it.line_no))?
            .as_str_val()?
            .to_string();


        match self.it.next() {
            None => Err(ParseError::new("No property action", self.it.line_no)),
            Some(Token::Equals) => Ok(Action::SetStat(idstr, Expr::from_tokens(&mut self.it)?)),
            Some(Token::Add) => Ok(Action::AddStat(idstr, Expr::from_tokens(&mut self.it)?)),
            Some(Token::Sub) => Ok(Action::AddStat(idstr, Expr::from_tokens(&mut self.it)?.neg())),
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
        let res = match self.it.next() {
            None => None,
            Some(Token::Colon) => Some(self.set_item_type()),
            Some(Token::Hash) => {
                Some(self.read_to_break());
                return self.next();
            }
            Some(Token::Ident(_)) | Some(Token::Qoth(_)) => Some(self.set_item()),
            Some(Token::Dot) => Some(self.set_property()),
            Some(Token::Break) => return self.next(),
            Some(Token::Add) => Some(self.gain_item(1)),
            Some(Token::Sub) => Some(self.gain_item(-1)),
            _ => {
                Some(Err(ParseError::new("err", self.it.line_no)))
            }
        };
        res
    }
}
