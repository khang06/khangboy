use crate::util::BitIndex;

// The pixel processing unit, which handles display stuff
#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
    vram: [u8; 0x2000],
    oam: [OAMObject; 40],
    viewport_y: u8,
    viewport_x: u8,
    lcd_control: u8,
    lcd_status: u8, // NOTE: Only the writable bits are stored here!
    lcd_y: u8,
    lcd_y_compare: u8,
    bg_palette: u8,

    draw_mode: DrawMode,
    scanline_dot: u16,

    scanline_objs: [OAMObject; 10],
    scanline_objs_idx: usize,

    bg_fifo: PixelFIFO,
    obj_fifo: PixelFIFO,

    framebuffer: [u8; 160 * 144],
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DrawMode {
    HBlank = 0,
    VBlank = 1,
    OAMScan = 2,
    Drawing = 3,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct OAMObject {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    pub flags: u8,
}

impl Default for PPU {
    fn default() -> Self {
        // Initial values from BGB
        Self {
            // TODO: Fill this with psuedo-random data
            vram: [0u8; 0x2000],
            oam: [Default::default(); 40],
            viewport_y: 0,
            viewport_x: 0,
            lcd_control: 0,
            lcd_status: 0x84,
            lcd_y: 0,
            lcd_y_compare: 0,
            bg_palette: 0xFC,
            draw_mode: DrawMode::OAMScan,
            scanline_dot: 0,
            scanline_objs: Default::default(),
            scanline_objs_idx: 0,
            bg_fifo: Default::default(),
            obj_fifo: Default::default(),
            framebuffer: [0u8; 160 * 144],
        }
    }
}

impl PPU {
    // Ticks one M-cycle
    pub fn tick(&mut self) -> bool {
        let mut vblank_interrupt = false;

        // Check if the PPU and LCD are enabled
        // TODO: This should render a blank screen
        if !self.lcd_control.test(7) {
            return vblank_interrupt;
        }
        for _ in 0..4 {
            match self.draw_mode {
                // Scan for sprites to be included on the current scanline
                // 80 dots, 1 sprite checked per 2 dots
                // TODO: This is just a guess and probably isn't T-cycle accurate
                DrawMode::OAMScan => {
                    if self.scanline_dot % 2 == 0 {
                        if self.scanline_dot == 0 {
                            self.scanline_objs_idx = 0;
                        }
                        let obj = self.oam[self.scanline_dot as usize / 2];
                        let height = if self.lcd_control.test(2) { 16 } else { 8 };
                        if obj.x != 0
                            && self.lcd_y.wrapping_add(16) >= obj.y
                            && self.lcd_y.wrapping_add(16) < obj.y.wrapping_add(height)
                            && self.scanline_objs_idx != self.scanline_objs.len()
                        {
                            self.scanline_objs[self.scanline_objs_idx] = obj;
                            self.scanline_objs_idx += 1;
                        }
                    }
                    if self.scanline_dot == 79 {
                        self.draw_mode = DrawMode::Drawing;
                    }
                }
                DrawMode::Drawing => {
                    if self.scanline_dot == 251 {
                        self.draw_mode = DrawMode::HBlank;
                    }
                }
                DrawMode::HBlank => {
                    if self.scanline_dot == 455 {
                        self.draw_mode = if self.lcd_y == 143 {
                            vblank_interrupt = true;
                            DrawMode::VBlank
                        } else {
                            DrawMode::OAMScan
                        }
                    }
                }
                DrawMode::VBlank => {}
            }

            self.scanline_dot += 1;
            if self.scanline_dot > 455 {
                self.lcd_y += 1;
                self.scanline_dot = 0;
            }
            if self.lcd_y > 153 {
                self.lcd_y = 0;
                self.draw_mode = DrawMode::OAMScan;
            }
        }

        vblank_interrupt
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        if self.draw_mode < DrawMode::Drawing || !self.lcd_control.test(7) {
            self.vram[addr as usize & 0x1FFF]
        } else {
            0xFF
        }
    }

    pub fn write_vram(&mut self, addr: u16, val: u8) {
        if self.draw_mode < DrawMode::Drawing || !self.lcd_control.test(7) {
            self.vram[addr as usize & 0x1FFF] = val;
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        // TODO: Reads from 0xFEA0-0xFEFF can trigger OAM corruption on DMG
        let index = addr as usize & 0xFF;
        if index < 0xA0 || self.draw_mode < DrawMode::OAMScan || !self.lcd_control.test(7) {
            // SAFETY: index is bounds checked
            unsafe {
                let oam_bytes = self.oam.as_ptr() as *const u8;
                *oam_bytes.add(index)
            }
        } else {
            0xFF
        }
    }

    pub fn write_oam(&mut self, addr: u16, val: u8) {
        // TODO: What happens with writes to 0xFEA0-0xFEFF?
        let index = addr as usize & 0xFF;
        if index < 0xA0 || self.draw_mode < DrawMode::OAMScan || !self.lcd_control.test(7) {
            // SAFETY: index is bounds checked
            unsafe {
                let oam_bytes = self.oam.as_mut_ptr() as *mut u8;
                *oam_bytes.add(index) = val;
            }
        }
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
        // Bit 7:   Always 1
        // Bit 2:   LYC=LY flag
        // Bit 1-0: Mode
        self.lcd_status | ((self.lcd_y == self.lcd_y_compare) as u8) << 2 | (self.draw_mode as u8)
    }

    pub fn write_stat(&mut self, val: u8) {
        // Only bits 3-6 are writable
        self.lcd_status = val & 0b0111_1000;
    }

    pub fn read_ly(&self) -> u8 {
        self.lcd_y
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

#[derive(Default)]
struct PixelFIFO {
    // Bit 0-1: Color index
    // Bit 2-4: Palette
    // Bit 5: Priority
    inner: [u8; 16],

    count: u8,
    read_head: u8,
    write_head: u8,
}

impl PixelFIFO {
    pub fn push(&mut self, pixel: u8) {
        assert!(self.count < self.inner.len() as u8);
        self.inner[self.write_head as usize] = pixel;
        self.write_head = (self.write_head + 1) % self.inner.len() as u8;
        self.count += 1;
    }

    pub fn pop(&mut self) -> u8 {
        assert!(self.count != 0);
        let ret = self.inner[self.read_head as usize];
        self.read_head = (self.read_head + 1) % self.inner.len() as u8;
        self.count -= 1;
        ret
    }
}
