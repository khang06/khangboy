pub struct PPU {
    viewport_y: u8,
    viewport_x: u8,
    lcd_control: u8,
    lcd_status: u8, // NOTE: Only the writable bits are stored here!
    lcd_y: u8,
    lcd_y_compare: u8,
    bg_palette: u8,

    scanline_dot: u16,
}

impl Default for PPU {
    fn default() -> Self {
        // Initial values from BGB
        Self {
            viewport_y: 0,
            viewport_x: 0,
            lcd_control: 0,
            lcd_status: 0x84,
            lcd_y: 0,
            lcd_y_compare: 0,
            bg_palette: 0xFC,
            scanline_dot: 0,
        }
    }
}

impl PPU {
    pub fn tick(&mut self) {
        self.scanline_dot += 1;
        if self.scanline_dot > 455 {
            self.lcd_y += 1;
            self.scanline_dot = 0;
        }
        if self.lcd_y > 153 {
            self.lcd_y = 0;
        }
    }

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

    pub fn read_scy(&self) -> u8 {
        self.viewport_y
    }

    pub fn write_scy(&mut self, val: u8) {
        self.viewport_y = val;
    }

    pub fn read_scx(&self) -> u8 {
        self.viewport_x
    }

    pub fn write_scx(&mut self, val: u8) {
        self.viewport_x = val;
    }

    pub fn read_lcdc(&self) -> u8 {
        self.lcd_control
    }

    pub fn write_lcdc(&mut self, val: u8) {
        self.lcd_control = val;
    }

    pub fn read_stat(&self) -> u8 {
        // TODO
        0x00
    }

    pub fn write_stat(&mut self, val: u8) {
        // Only bits 3-6 are writable
        self.lcd_status = val & 0b0111_1000;
    }

    pub fn read_ly(&self) -> u8 {
        #[cfg(not(feature = "gb_doctor"))]
        return self.lcd_y;

        #[cfg(feature = "gb_doctor")]
        0x90
    }

    pub fn read_lyc(&self) -> u8 {
        self.lcd_y_compare
    }

    pub fn write_lyc(&mut self, val: u8) {
        self.lcd_y_compare = val;
    }

    pub fn read_bgp(&self) -> u8 {
        self.bg_palette
    }

    pub fn write_bgp(&mut self, val: u8) {
        self.bg_palette = val;
    }
}
