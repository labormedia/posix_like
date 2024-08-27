use std::io::{
    self,
    Error,
    ErrorKind,
};

#[derive(Clone, Debug)]
pub struct Selector {
    pub active: bool,
    pub chars_selected: Vec<char>,
    pub char_stack: Vec<char>,
    pub catch_error: bool,
}

impl Selector {
    pub fn select(&mut self, c: &char) -> io::Result<bool> {
        match self.active {
            true => {
               if let Some(c_stacked) = self.char_stack.last() { 
                    if c == c_stacked {
                        self.char_stack.pop();
                        if self.char_stack.last() == None {
                            self.active = false;
                        }
                        Ok(true)
                    } else {
                        Ok(self.active)
                    }
                } else {
                    self.catch_error = true;
                    Err(Error::new(ErrorKind::Other.into(), "unmatched quotes."))
                } 
            },
            false => {
                if self.chars_selected.contains(&c) {
                    self.char_stack.push(*c);
                    self.active = true
                }
                Ok(c != &' ' || self.active)
            }
        }
    }
}