use crate::api::{console, io};
use crate::{print, println};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use vte::{Params, Parser, Perform};

pub struct Prompt {
    offset: usize, // Offset line by the length of the prompt string
    cursor: usize,
    line: Vec<char>, // UTF-32
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            offset: 0,
            cursor: 0,
            line: Vec::with_capacity(80),
        }
    }

    pub fn input(&mut self, prompt: &str) -> Option<String> {
        print!("{}", prompt);
        self.offset = offset_from_prompt(prompt);
        self.cursor = self.offset;
        self.line = Vec::with_capacity(80);
        let mut parser = Parser::new();
        while let Some(c) = io::stdin().read_char() {
            match c {
                console::ETX_KEY => {
                    // End of Text (^C)
                    println!();
                    return Some(String::new());
                }
                console::EOT_KEY => {
                    // End of Transmission (^D)
                    println!();
                    return None;
                }
                '\n' => {
                    // New Line
                    println!();
                    return Some(self.line.iter().collect());
                }
                c => {
                    for b in c.to_string().as_bytes() {
                        parser.advance(self, *b);
                    }
                }
            }
        }

        None
    }

    fn handle_delete_key(&mut self) {
        if self.cursor < self.offset + self.line.len() {
            let i = self.cursor - self.offset;
            self.line.remove(i);
            let s = &self.line[i..]; // UTF-32
            let n = s.len() + 1;
            let s: String = s.iter().collect(); // UTF-8
            print!("{} \x1b[{}D", s, n);
        }
    }

    fn handle_backspace_key(&mut self) {
        if self.cursor > self.offset {
            let i = self.cursor - self.offset - 1;
            self.line.remove(i);
            let s = &self.line[i..]; // UTF-32
            let n = s.len() + 1;
            let s: String = s.iter().collect(); // UTF-8
            print!("\x08{} \x1b[{}D", s, n);
            self.cursor -= 1;
        }
    }

    fn handle_printable_key(&mut self, c: char) {
        if console::is_printable(c) {
            let i = self.cursor - self.offset;
            self.line.insert(i, c);
            let s = &self.line[i..]; // UTF-32
            let n = s.len();
            let s: String = s.iter().collect(); // UTF-8
            print!("{}\x1b[{}D", s, n);
            self.cursor += 1;
        }
    }
}

impl Perform for Prompt {
    fn execute(&mut self, b: u8) {
        let c = b as char;
        match c {
            '\x08' => self.handle_backspace_key(),
            _ => {}
        }
    }

    fn print(&mut self, c: char) {
        match c {
            '\x7f' => self.handle_delete_key(),
            c => self.handle_printable_key(c),
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            '~' => {
                for param in params.iter() {
                    if param[0] == 3 {
                        // Delete
                        self.handle_delete_key();
                    }
                }
            }
            _ => {}
        }
    }
}
struct Offset(usize);

impl Perform for Offset {
    fn print(&mut self, c: char) {
        self.0 += c.len_utf8();
    }
}

fn offset_from_prompt(s: &str) -> usize {
    let mut parser = Parser::new();
    let mut offset = Offset(0);

    for b in s.bytes() {
        parser.advance(&mut offset, b);
    }
    offset.0
}
