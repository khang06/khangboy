// Handles link port stuff, used for link cable and blargg CPU tests
#[derive(Default)]
pub struct Serial;

impl Serial {
    pub fn read_sb(&self) -> u8 {
        // TODO
        0xFF
    }

    pub fn write_sb(&mut self, val: u8) {
        // TODO
        print!("{}", val as char);
    }

    pub fn read_sc(&self) -> u8 {
        // TODO
        0xFF
    }

    pub fn write_sc(&mut self, _val: u8) {
        // TODO
    }
}
