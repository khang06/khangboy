use crate::components::Components;
use crate::cpu::CPU;
use crate::rom::ROM;

pub struct Gameboy {
    cpu: CPU,
    components: Components,
}

impl Gameboy {
    pub fn new(rom: Box<dyn ROM>) -> Self {
        Self {
            cpu: CPU::new(),
            components: Components::new(rom),
        }
    }

    // TODO: Maybe this should run for X cycles?
    // Need to figure out the best way to update GUI and audio
    pub fn run(&mut self) {
        loop {
            self.cpu.step(&mut self.components)
        }
    }
}
