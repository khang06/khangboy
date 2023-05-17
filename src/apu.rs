use crate::util::BitIndex;

// The audio processing unit, which handles audio stuff
pub struct APU {
    enabled: bool,
}

impl Default for APU {
    fn default() -> Self {
        Self { enabled: false }
    }
}

impl APU {
    pub fn tick(&mut self) {}

    pub fn write_nr11(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr12(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr13(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr14(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr50(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr51(&mut self, _val: u8) {
        // TODO
    }

    pub fn read_nr52(&mut self) -> u8 {
        // TODO
        (self.enabled as u8) << 7
    }

    pub fn write_nr52(&mut self, val: u8) {
        self.enabled = val.test(7);
    }
}
