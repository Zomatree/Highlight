use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};

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
        self.values.iter().skip(self.advance_to_end()).cloned().collect()
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
