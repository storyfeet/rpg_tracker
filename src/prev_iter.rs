use crate::error::LineError;
use std::fmt::Debug;

pub trait LineCounter {
    fn line(&self) -> usize;
    fn err(&self, s: &str) -> LineError {
        LineError::new(s, self.line())
    }
    fn ux<T: Debug>(&self, t: T, s: &str) -> LineError {
        let fs = format!("UX - {:?} - {}", t, s);
        LineError::new(&fs, self.line())
    }
    fn eof(&self)->LineError{
        LineError::eof(self.line())
    }

}

pub trait Backer {
    fn back(&mut self);
}

impl<I, T> Backer for Prev<I, T>
where
    I: Clone,
    T: LineCounter + Iterator<Item = I>,
{
    fn back(&mut self) {
        self.is_back = true;
    }
}

impl<I, T> LineCounter for Prev<I, T>
where
    I: Clone,
    T: LineCounter + Iterator<Item = I>,
{
    fn line(&self) -> usize {
        let res = self.it.line();
        //        println!("Pr - line {}",res);
        res
        //self.it.line()
    }
}

pub struct Prev<I: Clone, T: Iterator<Item = I>> {
    it: T,
    prev: Option<I>,
    is_back: bool,
}

impl<I: Clone, T: Iterator<Item = I>> Iterator for Prev<I, T> {
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_back {
            self.prev = self.it.next();
        }
        self.is_back = false;
        self.prev.clone()
    }
}

impl<I: Clone, T: Iterator<Item = I>> Prev<I, T> {
    pub fn new(it: T) -> Self {
        Prev {
            it,
            prev: None,
            is_back: false,
        }
    }
    /*pub fn previous(&self) -> Option<I> {
        self.prev.clone()
    }*/
    pub fn back(&mut self) {
        self.is_back = true
    }
}
