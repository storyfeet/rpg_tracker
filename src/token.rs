use crate::error::ParseError;
use crate::prev_iter::{Prev,LineCounter};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Num(i32),
    Hash,
    Dot,
    Colon,
    Comma,
    Add,
    Sub,
    Mul,
    Div,
    Equals,
    Break,
    BOpen,
    BClose,
    SBOpen,
    SBClose,
    Qoth(String),
}

impl Token {
    pub fn as_str_val(&self) -> Result<&str, ParseError> {
        match self {
            Token::Ident(s) => Ok(s),
            Token::Qoth(s) => Ok(s),
            _ => Err(ParseError::new(&format!("{:?} not a string type", self), 0)),
        }
    }

    pub fn special_char(c: char) -> Option<Token> {
        match c {
            '#' => Some(Token::Hash),
            ':' => Some(Token::Colon),
            '.' => Some(Token::Dot),
            '+' => Some(Token::Add),
            '-' => Some(Token::Sub),
            '=' => Some(Token::Equals),
            '*' => Some(Token::Mul),
            '/' => Some(Token::Div),
            '(' => Some(Token::BOpen),
            ')' => Some(Token::BClose),
            '[' => Some(Token::SBOpen),
            ']' => Some(Token::SBClose),
            ',' => Some(Token::Comma),
            '\n' | ';' => Some(Token::Break),
            _ => None,
        }
    }
}

pub struct Tokenizer<'a> {
    it: Prev<char, std::str::Chars<'a>>,
    prev: Option<Token>,
    pub line_no: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str) -> Self {
        Tokenizer {
            it: Prev::new(s.chars()),
            line_no: 0,
            prev: None,
        }
    }

    pub fn previous(&mut self) -> Option<Token> {
        self.prev.clone()
    }

    fn read_num(&mut self) -> i32 {
        let mut res = 0;
        loop {
            match self.it.next() {
                Some(c) => {
                    if c >= '0' && c <= '9' {
                        res *= 10;
                        res += (c as i32) - 48;
                    } else {
                        self.it.back();
                        return res;
                    }
                }
                None => return res,
            }
        }
    }

    /// starts after the '"'
    fn read_qoth(&mut self) -> Token {
        let mut res = String::new();
        while let Some(c) = self.it.next() {
            match c {
                '\\' => res.push(self.it.next().unwrap_or(' ')),
                '"' => return Token::Qoth(res),
                '\n' => {
                    self.line_no += 1;
                    res.push('\n');
                }
                c => res.push(c),
            }
        }
        Token::Qoth(res)
    }

    fn read_ident(&mut self) -> String {
        let mut res = String::new();
        loop {
            let c = match self.it.next() {
                Some(c) => c,
                None => return res,
            };
            if let Some(_) = Token::special_char(c) {
                self.it.back();
                return res;
            }
            match c {
                ' ' | '\t' => return res,
                _ => res.push(c),
            }
        }
    }
}

impl<'a> LineCounter for Tokenizer<'a>{
    fn line(&self)->usize{
        self.line_no
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        let mut c = self.it.next()?;
        while c == ' ' || c == '\t' {
            c = self.it.next()?;
        }
        if let Some(r) = Token::special_char(c) {
            return Some(r);
        }

        let res = match c {
            '"' => self.read_qoth(),

            v if v >= '0' && v <= '9' =>{
                self.it.back();
                Token::Num(self.read_num())
            }
            _ => {
                self.it.back();
                Token::Ident(self.read_ident())
            }
        };

        self.prev = Some(res.clone());
        Some(res)
    }
}

#[cfg(test)]
mod test_tokens {
    use super::*;
    #[test]
    pub fn test_token_reads() {
        let mut tk = Tokenizer::new("hello:52 + d6");
        assert_eq!(tk.next(), Some(Token::Ident("hello".to_string())));
        assert_eq!(tk.next(), Some(Token::Colon), "c1-2");
        assert_eq!(tk.next(), Some(Token::Num(52)));
        assert_eq!(tk.next(), Some(Token::Add));
        assert_eq!(tk.next(), Some(Token::Ident("d6".to_string())));
        assert!(tk.next().is_none());
    }

    #[test]
    pub fn test_qoth(){
        let mut tk = Tokenizer::new(r#"hello:"Goodbye","Nice","to \"meet\" you""#);
        assert_eq!(tk.next().unwrap(),Token::Ident("hello".to_string()));
        assert_eq!(tk.next().unwrap(),Token::Colon);
        assert_eq!(tk.next().unwrap(),Token::Qoth("Goodbye".to_string()));
        assert_eq!(tk.next().unwrap(),Token::Comma);
        assert_eq!(tk.next().unwrap(),Token::Qoth("Nice".to_string()));
        assert_eq!(tk.next().unwrap(),Token::Comma);
        assert_eq!(tk.next().unwrap(),Token::Qoth("to \"meet\" you".to_string()));
        assert!(tk.next().is_none());
    }

}
