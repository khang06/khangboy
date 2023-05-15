pub struct PPU;

impl Default for PPU {
    fn default() -> Self {
        Self {}
    }
}

impl PPU {
    pub fn tick(&mut self) {}

    pub fn read_vram(&self, addr: u16) -> u8 {
        // TODO
        0x00
    }

    pub fn write_vram(&mut self, addr: u16, val: u8) {
        // TODO
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        // TODO
        0x00
    }

    pub fn write_oam(&mut self, addr: u16, val: u8) {
        // TODO
    }
}
