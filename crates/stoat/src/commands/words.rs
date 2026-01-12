use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug, Clone)]
pub struct Words {
    values: Vec<String>,
    pos: Arc<AtomicUsize>,
}

impl Words {
    pub fn new(input: &str) -> Self {
        Self {
            values: input.split(' ').map(|v| v.to_string()).collect(),
            pos: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn next(&self) -> Option<String> {
        let pos = self.advance();
        let value = self.values.get(pos).cloned();

        value
    }

    pub fn current(&self) -> Option<String> {
        self.values.get(self.current_position()).cloned()
    }

    pub fn rest(&self) -> Vec<String> {
        self.values
            .iter()
            .skip(self.advance_to_end())
            .cloned()
            .collect()
    }

    pub fn current_position(&self) -> usize {
        self.pos.load(Ordering::SeqCst)
    }

    pub fn advance(&self) -> usize {
        self.pos.fetch_add(1, Ordering::SeqCst)
    }

    pub fn undo(&self) -> usize {
        self.pos.fetch_sub(1, Ordering::SeqCst)
    }

    pub fn advance_to_end(&self) -> usize {
        self.pos.swap(self.values.len(), Ordering::SeqCst)
    }
}

/*
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug, Clone)]
pub struct Words {
    value: String,
    pos: Arc<AtomicUsize>,
    previous: Arc<AtomicUsize>,
}

impl Words {
    pub fn new(input: String) -> Self {
        Self {
            value: input,
            pos: Arc::new(AtomicUsize::new(0)),
            previous: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn index(&self) -> usize {
        self.pos.load(Ordering::SeqCst)
    }

    pub fn eof(&self) -> bool {
        self.index() >= self.value.len()
    }

    pub fn current(&self) -> Option<char> {
        if self.eof() {
            None
        } else {
            self.value.chars().nth(self.index())
        }
    }

    pub fn undo(&self) {
        self.pos.store(self.previous.load(Ordering::SeqCst), Ordering::SeqCst);
    }

    pub fn skip_ws(&self) -> bool {
        let mut pos = 0;

        while !self.eof() {
            if let Some(current) = self.value.chars().nth(self.index() + pos) {
                if !current.is_whitespace() {
                    break;
                };

                pos += 1;
            } else {
                break;
            }
        };

        self.previous.store(self.index(), Ordering::SeqCst);
        self.pos.fetch_add(pos, Ordering::SeqCst);

        self.previous.load(Ordering::SeqCst) != self.index()
    }

    pub fn skip_string(&self, string: &str) -> bool {
        let len = string.len();
        let index = self.index();

        if &self.value[index..index + len] == string {
            self.previous.store(index, Ordering::SeqCst);
            self.pos.fetch_add(len, Ordering::SeqCst);

            true
        } else {
            false
        }
    }

    pub fn read_rest(&self) -> &str {
        let index = self.index();
        let rest = &self.value[index..];

        self.previous.store(index, Ordering::SeqCst);
        self.pos.store(self.value.len(), Ordering::SeqCst);

        rest
    }

    pub fn read(&self, n: usize) -> &str {
        let index = self.index();
        let rest = &self.value[index..index + n];

        self.previous.store(index, Ordering::SeqCst);
        self.pos.fetch_add(n, Ordering::SeqCst);

        rest
    }

    pub fn get(&self) -> Option<char> {
        let index = self.index();

        let ch = self.value.chars().nth(index + 1);

        self.previous.store(index, Ordering::SeqCst);
        self.pos.fetch_add(1, Ordering::SeqCst);

        ch
    }

    pub fn get_word(&self) -> &str {
        let mut pos = 0;
        let index= self.index();
        let mut chars = self.value.chars().skip(index);

        while !self.eof() {
            if let Some(ch) = chars.next() {
                if ch.is_whitespace() {
                    break;
                };

                pos += 1;
            } else {
                break
            };
        };

        self.previous.store(index, Ordering::SeqCst);
        self.pos.fetch_add(pos, Ordering::SeqCst);

        &self.value[index..index + pos]
    }

    pub fn get_quoted_word(&self) -> Option<&str> {

    }
}
*/