use crate::components::Components;

#[derive(Default)]
pub struct CPU {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8, // Flags
    h: u8,
    l: u8,

    sp: u16,
    pc: u16,

    opcode: u8, // Fetched during execution of last instruction
    cycle: u64, // Counted in M-cycles
}

impl CPU {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn step(&mut self, com: &mut Components) {
        // TODO
    }
}
