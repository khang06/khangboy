pub struct Serial;

impl Default for Serial {
    fn default() -> Self {
        Self {}
    }
}

impl Serial {
    pub fn tick(&mut self) {}
}
