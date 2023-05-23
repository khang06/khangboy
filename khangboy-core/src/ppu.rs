use std::default;

use crate::util::BitIndex;

// The pixel processing unit, which handles display stuff
#[allow(clippy::upper_case_acronyms)]
pub struct PPU {
    pub vram: [u8; 0x2000],
    oam: [OAMObject; 40],
    viewport_y: u8,
    viewport_x: u8,
    window_y: u8,
    window_x: u8,
    lcd_control: u8,
    lcd_status: u8, // NOTE: Only the writable bits are stored here!
    lcd_y: u8,
    lcd_y_compare: u8,
    lcd_x: u8,
    bg_palette: u8,
    obp0: u8,
    obp1: u8,

    draw_mode: DrawMode,
    scanline_dot: u16,

    fetcher: PixelFetcher,

    scanline_objs: [OAMObject; 10],
    scanline_objs_count: usize,

    window_triggered: bool,
    window_lcd_y: u8,

    pub oam_dma_running: bool,
    pub oam_dma_src: u8,
    pub oam_dma_idx: u8,

    temp_framebuffer: [u8; 160 * 144],
    pub framebuffer: [u8; 160 * 144],
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
            window_y: 0,
            window_x: 0,
            lcd_control: 0,
            lcd_status: 0x84,
            lcd_y: 0,
            lcd_y_compare: 0,
            lcd_x: 0,
            bg_palette: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            draw_mode: DrawMode::OAMScan,
            scanline_dot: 0,
            fetcher: Default::default(),
            scanline_objs: Default::default(),
            scanline_objs_count: 0,
            window_triggered: false,
            window_lcd_y: 0,
            oam_dma_running: false,
            oam_dma_src: 0x00,
            oam_dma_idx: 0,
            temp_framebuffer: [0u8; 160 * 144],
            framebuffer: [0u8; 160 * 144],
        }
    }
}

impl PPU {
    // Ticks one M-cycle
    pub fn tick(&mut self) -> (bool, bool) {
        let mut vblank_interrupt = false;
        let mut stat_interrupt = false;

        // Check if the PPU and LCD are enabled
        // TODO: This should render a blank screen
        if !self.lcd_control.test(7) {
            return (vblank_interrupt, stat_interrupt);
        }
        for _ in 0..4 {
            // This is hell
            // http://pixelbits.16-b.it/GBEDG/ppu/#the-pixel-fifo
            match self.draw_mode {
                // Scan for sprites to be included on the current scanline
                // 80 dots, 1 sprite checked per 2 dots
                // TODO: This is just a guess and probably isn't T-cycle accurate
                DrawMode::OAMScan => {
                    if self.scanline_dot % 2 == 0 {
                        if self.scanline_dot == 0 {
                            self.scanline_objs_count = 0;
                            if self.lcd_y == self.window_y {
                                self.window_triggered = true;
                            }
                        }
                        let obj = self.oam[self.scanline_dot as usize / 2];
                        let height = if self.lcd_control.test(2) { 16 } else { 8 };
                        if obj.x != 0
                            && self.lcd_y.wrapping_add(16) >= obj.y
                            && self.lcd_y.wrapping_add(16) < obj.y.wrapping_add(height)
                            && self.scanline_objs_count != self.scanline_objs.len()
                        {
                            self.scanline_objs[self.scanline_objs_count] = obj;
                            self.scanline_objs_count += 1;
                        }
                    }
                    if self.scanline_dot == 79 {
                        self.scanline_objs[..self.scanline_objs_count].sort_by_key(|x| x.x);
                        self.fetcher = Default::default();
                        self.fetcher.bg_excess = self.viewport_x & 7;
                        self.lcd_x = 0;
                        self.draw_mode = DrawMode::Drawing;
                    }
                    self.scanline_dot += 1;
                }
                DrawMode::Drawing => {
                    self.scanline_dot += 1;
                    self.tick_fetcher();
                    if self.fetcher.bg_fifo.count != 0 {
                        if self.fetcher.bg_excess == 0 {
                            let bg = self.fetcher.bg_fifo.pop();
                            let bg_col = if (bg & 4) != 0 {
                                (self.bg_palette >> ((bg & 3) * 2)) & 3
                            } else {
                                0x00
                            };
                            let col = if self.fetcher.sprite_fifo.count != 0 {
                                let sprite = self.fetcher.sprite_fifo.pop();
                                if sprite & 3 != 0 {
                                    if sprite.test(4) && (bg & 3 != 0) {
                                        bg_col
                                    } else {
                                        if sprite.test(3) {
                                            (self.obp1 >> ((sprite & 3) * 2)) & 3
                                        } else {
                                            (self.obp0 >> ((sprite & 3) * 2)) & 3
                                        }
                                    }
                                } else {
                                    bg_col
                                }
                            } else {
                                bg_col
                            };
                            self.temp_framebuffer
                                [self.lcd_y as usize * 160 + self.lcd_x as usize] = col;
                            self.lcd_x += 1;
                            if self.lcd_x == 160 {
                                if self.lcd_control.test(3) {
                                    stat_interrupt = true;
                                }
                                self.draw_mode = DrawMode::HBlank;
                            }
                        } else {
                            self.fetcher.bg_fifo.pop();
                            self.fetcher.bg_excess -= 1;
                        }
                    }
                    assert!(self.scanline_dot <= 389);
                }
                DrawMode::HBlank => {
                    self.scanline_dot += 1;
                    if self.scanline_dot == 456 {
                        self.lcd_y += 1;
                        if self.fetcher.bg_window {
                            self.window_lcd_y += 1;
                        }
                        self.scanline_dot = 0;
                        if self.lcd_y == self.lcd_y_compare && self.lcd_control.test(6) {
                            stat_interrupt = true;
                        }
                        self.draw_mode = if self.lcd_y == 144 {
                            vblank_interrupt = true;
                            if self.lcd_control.test(4) {
                                stat_interrupt = true;
                            }
                            self.window_triggered = false;
                            self.framebuffer.clone_from(&self.temp_framebuffer);
                            DrawMode::VBlank
                        } else {
                            if self.lcd_control.test(5) {
                                stat_interrupt = true;
                            }
                            DrawMode::OAMScan
                        }
                    }
                }
                DrawMode::VBlank => {
                    self.scanline_dot += 1;
                    if self.scanline_dot == 456 {
                        self.lcd_y += 1;
                        if self.lcd_y == self.lcd_y_compare && self.lcd_control.test(6) {
                            stat_interrupt = true;
                        }
                        self.scanline_dot = 0;
                    }
                    if self.lcd_y > 153 {
                        self.lcd_y = 0;
                        self.window_lcd_y = 0;
                        if self.lcd_control.test(5) {
                            stat_interrupt = true;
                        }
                        self.draw_mode = DrawMode::OAMScan;
                    }
                }
            }
        }

        (vblank_interrupt, stat_interrupt)
    }

    pub fn tick_fetcher(&mut self) {
        if self.lcd_control.test(1)
            && !self.fetcher.fetching_sprite
            && self.fetcher.sprite_next_idx != self.scanline_objs_count
            && self.scanline_objs[self.fetcher.sprite_next_idx].x - 8 <= self.lcd_x
        {
            self.fetcher.fetching_sprite = true;
            self.fetcher.sprite_ticks = 0;
            self.fetcher.sprite_state = FetcherState::GetTile;
            self.fetcher.sprite_obj = self.scanline_objs[self.fetcher.sprite_next_idx];
            self.fetcher.sprite_next_idx += 1;
        }

        if self.fetcher.fetching_sprite {
            self.tick_fetcher_sprite();
            return;
        }

        if self.window_triggered
            && self.lcd_control.test(5)
            && !self.fetcher.bg_window
            && self.lcd_x >= self.window_x - 7
        {
            self.fetcher.bg_window = true;
            self.fetcher.bg_fifo = Default::default();
            self.fetcher.x = 0;
            self.fetcher.bg_state = FetcherState::GetTile;
        }
        self.tick_fetcher_bg();
    }

    fn tick_fetcher_sprite(&mut self) {
        // Each stage takes 2 M-cycles
        self.fetcher.sprite_ticks += 1;
        if self.fetcher.sprite_ticks % 2 == 1 {
            return;
        }

        self.fetcher.sprite_state = match self.fetcher.sprite_state {
            FetcherState::GetTile => {
                self.fetcher.sprite_tile = self.fetcher.sprite_obj.tile;
                FetcherState::GetTileDataLow
            }
            FetcherState::GetTileDataLow | FetcherState::GetTileDataHigh => {
                let tile_addr = self.fetcher.sprite_tile as usize * 16;

                // TODO: Investigate mid-scanline OBJ size change behavior
                let offset = if self.fetcher.sprite_obj.flags.test(6) {
                    if self.lcd_status.test(2) {
                        30 - (self.lcd_y - (self.fetcher.sprite_obj.y - 16)) as usize * 2
                    } else {
                        14 - (self.lcd_y - (self.fetcher.sprite_obj.y - 16)) as usize * 2
                    }
                } else {
                    (self.lcd_y - (self.fetcher.sprite_obj.y - 16)) as usize * 2
                };
                if self.fetcher.sprite_state == FetcherState::GetTileDataLow {
                    self.fetcher.sprite_low = self.vram[(tile_addr + offset) & 0x1FFF];
                    FetcherState::GetTileDataHigh
                } else {
                    self.fetcher.sprite_high = self.vram[(tile_addr + offset + 1) & 0x1FFF];
                    FetcherState::Push
                }
            }
            FetcherState::Push => {
                for i in (0..=7).rev() {
                    self.fetcher.sprite_fifo.push(
                        0u8.set(0, self.fetcher.sprite_low.test(i))
                            .set(1, self.fetcher.sprite_high.test(i))
                            .set(3, self.fetcher.sprite_obj.flags.test(4))
                            .set(4, self.fetcher.sprite_obj.flags.test(1)),
                    );
                }
                self.fetcher.fetching_sprite = false;
                FetcherState::GetTile
            }
        }
    }

    fn tick_fetcher_bg(&mut self) {
        // Each stage takes 2 M-cycles
        self.fetcher.bg_ticks += 1;
        if self.fetcher.bg_ticks % 2 == 1 {
            return;
        }

        self.fetcher.bg_state = match self.fetcher.bg_state {
            FetcherState::GetTile => {
                let map_bit = if self.fetcher.bg_window { 6 } else { 3 };
                let map_addr = if self.lcd_control.test(map_bit) {
                    0x1C00
                } else {
                    0x1800
                };
                let (x, y) = if self.fetcher.bg_window {
                    (self.fetcher.x, self.window_lcd_y / 8)
                } else {
                    (
                        (self.fetcher.x.wrapping_add(self.viewport_x / 8)) & 0x1F,
                        (self.lcd_y.wrapping_add(self.viewport_y)) / 8,
                    )
                };
                self.fetcher.bg_tile =
                    self.vram[map_addr + ((y as usize * 32 + x as usize) & 0x3FF)];
                FetcherState::GetTileDataLow
            }
            FetcherState::GetTileDataLow | FetcherState::GetTileDataHigh => {
                let tile_addr = if self.lcd_control.test(4) {
                    self.fetcher.bg_tile as usize * 16
                } else {
                    (0x1000 + self.fetcher.bg_tile as i8 as isize * 16) as usize
                };

                let offset = if self.fetcher.bg_window {
                    (self.window_lcd_y & 7) as usize * 2
                } else {
                    ((self.lcd_y + self.viewport_y) & 7) as usize * 2
                };
                if self.fetcher.bg_state == FetcherState::GetTileDataLow {
                    self.fetcher.bg_low = self.vram[(tile_addr + offset) & 0x1FFF];
                    FetcherState::GetTileDataHigh
                } else {
                    self.fetcher.bg_high = self.vram[(tile_addr + offset + 1) & 0x1FFF];
                    FetcherState::Push
                }
            }
            FetcherState::Push => {
                if self.fetcher.bg_fifo.count == 0 {
                    for i in (0..=7).rev() {
                        self.fetcher.bg_fifo.push(
                            0u8.set(0, self.fetcher.bg_low.test(i))
                                .set(1, self.fetcher.bg_high.test(i))
                                .set(2, self.lcd_control.test(0)),
                        );
                    }
                    self.fetcher.x += 1;
                    FetcherState::GetTile
                } else {
                    FetcherState::Push
                }
            }
        }
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
        if index < 0xA0 && (self.draw_mode < DrawMode::OAMScan || !self.lcd_control.test(7)) {
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
        if index < 0xA0 && (self.draw_mode < DrawMode::OAMScan || !self.lcd_control.test(7)) {
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

    pub fn read_obp0(&self) -> u8 {
        self.obp0
    }

    pub fn write_obp0(&mut self, val: u8) {
        self.obp0 = val;
    }

    pub fn read_obp1(&self) -> u8 {
        self.obp1
    }

    pub fn write_obp1(&mut self, val: u8) {
        self.obp1 = val;
    }

    pub fn read_dma(&self) -> u8 {
        self.oam_dma_src
    }

    pub fn write_dma(&mut self, val: u8) {
        self.oam_dma_running = true;
        self.oam_dma_idx = 0;
        self.oam_dma_src = val;
    }

    pub fn read_wy(&self) -> u8 {
        self.window_y
    }

    pub fn write_wy(&mut self, val: u8) {
        self.window_y = val;
    }

    pub fn read_wx(&self) -> u8 {
        self.window_x
    }

    pub fn write_wx(&mut self, val: u8) {
        self.window_x = val;
    }
}

#[derive(Default)]
struct PixelFIFO {
    // Bit 0-1: Color index
    // Bit 2: Enabled (BG only)
    // Bit 3: Palette (Sprites only)
    // Bit 4: Priority (Sprites only)
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

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum FetcherState {
    #[default]
    GetTile,
    GetTileDataLow,
    GetTileDataHigh,
    Push,
}

#[derive(Default)]
struct PixelFetcher {
    sprite_state: FetcherState,
    sprite_ticks: usize,
    sprite_fifo: PixelFIFO,
    sprite_next_idx: usize,
    sprite_obj: OAMObject,
    sprite_tile: u8,
    sprite_low: u8,
    sprite_high: u8,

    bg_state: FetcherState,
    bg_ticks: usize,
    bg_fifo: PixelFIFO,
    bg_tile: u8,
    bg_low: u8,
    bg_high: u8,
    bg_excess: u8,
    bg_window: bool,

    x: u8,
    fetching_sprite: bool,
}
