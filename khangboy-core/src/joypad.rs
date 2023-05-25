use crate::util::BitIndex;

pub struct Joypad {
    pub cur_input: u8,
    p1: u8,
    last_p1: u8,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            cur_input: 0x00,
            p1: 0xCF,
            last_p1: 0xCF,
        }
    }
}

impl Joypad {
    pub fn tick(&mut self) -> bool {
        self.p1 &= 0x30;

        // A, B, Start, Select
        if !self.p1.test(5) {
            self.p1 |= !self.cur_input & 0xF;
        }

        // Right, Left, Up, Down
        if !self.p1.test(4) {
            self.p1 |= !self.cur_input >> 4;
        }

        let interrupt = (self.last_p1 & !self.p1 & 3) != 0;
        self.last_p1 = self.p1;
        interrupt
    }

    pub fn read_p1(&self) -> u8 {
        self.p1 | 0xC0
    }

    pub fn write_p1(&mut self, val: u8) {
        self.p1 = val & 0x30;
    }
}
