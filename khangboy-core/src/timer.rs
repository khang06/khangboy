use crate::util::BitIndex;

// Keeps track of cycles
#[derive(Default)]
pub struct Timer {
    clocks: u16,
    counter: u8,
    modulo: u8,
    control: u8,

    edge_delay: bool,
}

impl Timer {
    pub fn tick(&mut self) -> bool {
        // Lots of documentation states that the timer's internal counter is incremented per T-cycle,
        // but recent research finds that it's actually incremented per M-cycle
        self.clocks = self.clocks.wrapping_add(1);

        // See https://gbdev.io/pandocs/Timer_Obscure_Behaviour.html
        let test_bit = self.control.test(2)
            & self.clocks.test(match self.control & 3 {
                0 => 7,
                1 => 1,
                2 => 3,
                3 => 5,
                _ => unreachable!(),
            });

        // Falling edge detector
        let mut trigger_interrupt = false;
        if !test_bit & self.edge_delay {
            let (val, carry) = self.counter.overflowing_add(1);
            self.counter = if carry {
                trigger_interrupt = true;
                self.modulo
            } else {
                val
            };
        }
        self.edge_delay = test_bit;

        trigger_interrupt
    }

    pub fn read_div(&self) -> u8 {
        (self.clocks >> 6) as u8
    }

    pub fn write_div(&mut self, _val: u8) {
        self.clocks = 0;
    }

    pub fn read_tima(&self) -> u8 {
        self.counter
    }

    pub fn write_tima(&mut self, val: u8) {
        self.counter = val;
    }

    pub fn read_tma(&self) -> u8 {
        self.modulo
    }

    pub fn write_tma(&mut self, val: u8) {
        self.modulo = val;
    }

    pub fn read_tac(&self) -> u8 {
        // Bits 7-4 always return 1
        self.control | 0b1111_1000
    }

    pub fn write_tac(&mut self, val: u8) {
        self.control = val;
    }
}
