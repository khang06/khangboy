use crate::components::Components;
use crate::cpu::CPU;
use crate::rom::ROM;

pub struct Gameboy {
    pub cpu: CPU,
    pub components: Components,
}

impl Gameboy {
    pub fn new(rom: Box<dyn ROM>) -> Self {
        Self {
            cpu: CPU::new(),
            components: Components::new(rom),
        }
    }

    // Runs for AT LEAST n cycles
    // Actual cycle count is returned
    pub fn run(&mut self, cycles: u64) -> u64 {
        let mut executed = 0;
        while executed < cycles {
            executed += self.cpu.step(&mut self.components);
        }
        executed
    }
}
