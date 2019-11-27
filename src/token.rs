use crate::error::ParseError;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Num(i32),
    Hash,
    Dot,
    Colon,
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

pub struct Prev<I: Clone, T: Iterator<Item = I>> {
    it: T,
    prev: Option<I>,
}

impl<I: Clone, T: Iterator<Item = I>> Iterator for Prev<I, T> {
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        self.prev = self.it.next();
        self.prev.clone()
    }
}

impl<I: Clone, T: Iterator<Item = I>> Prev<I, T> {
    pub fn previous(&self) -> Option<I> {
        self.prev.clone()
    }
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
            '\n' | ';' => Some(Token::Break),
            _ => None,
        }
    }
}

pub struct Tokenizer<'a> {
    it: Prev<char, std::str::Chars<'a>>,
    prev: Option<Token>,
    pub line_no: i32,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str) -> Self {
        Tokenizer {
            it: Prev {
                it: s.chars(),
                prev: None,
            },
            line_no: 0,
            prev: None,
        }
    }

    pub fn previous(&mut self) -> Option<Token> {
        self.prev.clone()
    }

    fn read_num(&mut self) -> i32 {
        let mut res = 0;
        let mut ch = self.it.previous();
        loop {
            match ch {
                Some(c) => {
                    if c >= '0' && c <= '9' {
                        res *= 10;
                        res += (c as i32) - 48;
                    } else {
                        return res;
                    }
                }
                None => return res,
            }
            ch = self.it.next()
        }
    }

    fn non_ws(&mut self) -> Option<Token> {
        while let Some(c) = self.peek {
            match c {
                ' ' | '\t' => self.peek = self.it.next(),
                _ => return self.next(),
            }
        }
        None
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
    fn take_single(&mut self) -> Option<Token> {
        let r = self.peek.take().unwrap_or(' ');
        if r == '\n' {
            self.line_no += 1
        }
        Token::special_char(r)
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
            if Token::special_char(pk).is_some() {
                return res;
            }
            match pk {
                ' ' | '\n' => return res,
                _ => {
                    self.peek.take();
                    res.push(pk)
                }
            }
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.it.next()?;
        if Token::special_char(c).is_some() {
            return self.take_single();
        }

        let res = match c {
            '"' => self.read_qoth(),
            ' ' | '\t' => self.non_ws()?,

            v if v >= '0' && v <= '9' => Token::Num(self.read_num()),
            _ => Token::Ident(self.read_ident()),
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
}
