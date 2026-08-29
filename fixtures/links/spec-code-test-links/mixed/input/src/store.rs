/// Implements: FR-001-AC-1
pub struct Store {
    pub count: u32,
}

impl Store {
    pub fn upsert(&mut self) {
        self.count += 1;
    }
}
