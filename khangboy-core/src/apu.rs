use crate::util::BitIndex;

// The audio processing unit, which handles audio stuff
#[derive(Default)]
pub struct APU {
    enabled: bool,
    pan: u8,
    wave_ram: [u8; 0x10],
}

impl APU {
    pub fn tick(&mut self) {}

    pub fn write_nr10(&mut self, _val: u8) {
        // TODO
    }

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

    pub fn write_nr21(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr22(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr23(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr24(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr30(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr31(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr32(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr33(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr34(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr41(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr42(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr43(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr44(&mut self, _val: u8) {
        // TODO
    }

    pub fn write_nr50(&mut self, _val: u8) {
        // TODO
    }

    pub fn read_nr51(&self) -> u8 {
        self.pan
    }

    pub fn write_nr51(&mut self, val: u8) {
        self.pan = val
    }

    pub fn read_nr52(&self) -> u8 {
        // TODO
        (self.enabled as u8) << 7
    }

    pub fn write_nr52(&mut self, val: u8) {
        self.enabled = val.test(7);
    }

    pub fn write_wave(&mut self, addr: u16, val: u8) {
        // TODO: Needs to implement access conflict behavior
        self.wave_ram[addr as usize & 0xF] = val;
    }
}
