use crate::error::ParseError;

#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    Ident(String),
    Num(i32),
    Hash,
    Dot,
    Colon,
    Plus,
    Minus,
    Equals,
    WS,
    NL,
    Qoth(String),
}

impl Token{
    pub fn as_str_val(&self)->Result<&str,ParseError>{
        match self{
            Token::Ident(s)=>Ok(s),
            Token::Qoth(s)=>Ok(s),
            _=>Err(ParseError::new("Not a string type",0)),
        }
    }
}

pub struct Tokenizer<'a> {
    it: std::str::Chars<'a>,
    peek: Option<char>,
    pub line_no: i32,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str) -> Self {
        Tokenizer {
            it: s.chars(),
            peek: None,
            line_no: 0,
        }
    }

    fn read_num(&mut self) -> i32 {
        let mut res = 0;
        loop {
            if self.peek == None {
                self.peek = self.it.next();
            }
            match self.peek {
                Some(c) => {
                    if c >= '0' && c <= '9' {
                        res *= 10;
                        res += (c as i32) - 48;
                        self.peek.take();
                    } else {
                        return res;
                    }
                }
                None => return res,
            }
        }
    }

    fn read_ws(&mut self) {
        while let Some(c) = self.peek {
            match c {
                ' ' | '\t' => self.peek = self.it.next(),
                _ => return,
            }
        }
    }

    fn read_qoth(&mut self) -> Token {
        self.peek = self.it.next(); //read open quote
        let mut res = String::new();
        let mut esc = false;
        loop {
            if self.peek == None {
                self.peek = self.it.next();
            }
            let pk = match self.peek {
                Some(c) => c,
                None => return Token::Qoth(res),
            };
            if esc {
                esc = false;
                res.push(pk);
                continue;
            }
            match pk {
                '\\' => esc = true,
                '"' => {
                    self.peek.take();
                    return Token::Qoth(res);
                }
                '\n' => {
                    self.line_no += 1;
                    self.peek.take();
                    res.push('\n');
                }
                c => {
                    self.peek.take();
                    res.push(c)
                }
            }
        }
    }

    ///requires the next char is the right type
    fn take_single(&mut self) -> Token {
        match self.peek.take().unwrap_or('_') {
            '#' => Token::Hash,
            ':' => Token::Colon,
            '.' => Token::Dot,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '=' => Token::Equals,
            '\n' => {
                self.line_no += 1;
                Token::NL
            }
            _ => Token::WS,
        }
    }

    fn read_ident(&mut self) -> String {
        let mut res = String::new();
        loop {
            if self.peek == None {
                self.peek = self.it.next();
            }
            let pk = match self.peek {
                Some(c) => c,
                None => return res,
            };
            match pk {
                '\n' | ' ' | ':' | ',' | '"' | '+' | '-' | '=' | '#' => return res,
                c => {
                    self.peek.take();
                    res.push(c)
                }
            }
        }
    }

    pub fn next_nws(&mut self) -> Option<Token> {
        let res = self.next();
        while res == Some(Token::WS) {
            res = self.next();
        }
        res
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        if self.peek == None {
            self.peek = self.it.next();
        }

        let res = match self.peek? {
            '\n' | ':' | '-' | '.' | ',' | '+' | '=' | '#' => self.take_single(),
            '"' => self.read_qoth(),
            ' ' | '\t' => {
                self.read_ws();
                Token::WS
            }
            v if v >= '0' && v <= '9' => Token::Num(self.read_num()),
            _ => Token::Ident(self.read_ident()),
        };

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
        assert_eq!(tk.next(), Some(Token::WS));
        assert_eq!(tk.next(), Some(Token::Plus));
        assert_eq!(tk.next(), Some(Token::WS));
        assert_eq!(tk.next(), Some(Token::Ident("d6".to_string())));
        assert!(tk.next().is_none());
    }
}
