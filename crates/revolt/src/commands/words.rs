#[derive(Debug, Clone)]
pub struct Words {
    values: Vec<String>,
    pos: usize,
}

impl Words {
    pub fn new(input: &str) -> Self {
        Self {
            values: input.split(' ').map(|v| v.to_string()).collect(),
            pos: 0,
        }
    }

    pub fn next(&mut self) -> Option<String> {
        let value = self.values.get(self.pos).cloned();
        self.advance();

        value
    }

    pub fn current(&self) -> Option<String> {
        self.values.get(self.pos).cloned()
    }

    pub fn rest(&self) -> Vec<String> {
        self.values.iter().skip(self.pos).cloned().collect()
    }

    pub fn advance(&mut self) {
        self.pos += 1;
    }

    pub fn undo(&mut self) {
        self.pos = self.pos.saturating_sub(1);
    }
}
