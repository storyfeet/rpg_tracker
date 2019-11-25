use crate::token::{Tokenizer,Token};
use crate::error::ParseError;

pub enum Action{
    Create(String,String),
    Select(String),
    AddStat(String,i32),
    SetStat(String,i32),
    NoAction,
}



pub struct ActionReader<'a>{ 
    it:Tokenizer<'a>,
    peek:Option<Token>
}

impl<'a> ActionReader<'a>{
    pub fn new(s:&'a str)->Self{
        ActionReader{
            it : Tokenizer::new(s),
            peek:None,
        }
    }
}

impl<'a> ActionReader<'a>{
    pub fn read_comment_line(&mut self)->Result<Action,ParseError>{
        while self.peek != Some(Token::NL){
            self.peek = self.it.next();
        }
        self.peek = self.it.next();
        Ok(Action::NoAction)
    }
    pub fn read_selector(&mut self)->Result<Action,ParseError>{
        let s1 = self.it.next()
            .ok_or(ParseError::new("String short",0))?
            .as_str_val()?.to_string();
        loop{
            match self.it.next_nws() {
                None:: return OK(Action::Select(s1))
                Some(Token::Colon) =>break,
                Some(Token::NL) => {
                    self.peek = self.it.next();
                    return OK(Action::Select(s1))
                }
                Token::Ident(s) => s1.push_str(
            }
        }
         
    }
}

impl<'a> Iterator for ActionReader<'a>{
    type Item = Result<Action,ParseError>;
    fn next(&mut self)->Option<Self::Item>{
        if self.peek == None {
            self.peek = self.it.next();
        }
        if self.peek == None{
            return None;
        }
        match self.peek{
            Token::Hash=> self.read_comment_line(),
            Token::Ident=> self.read_property(),
            Token::WS => self.read_selector(),
            _=>{
                self.peek = self.it.next();
                return Some(Err(ParseError{mess:"err",line:self.it.line_no}))
            }
        }
    }
}




