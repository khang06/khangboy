use std::unimplemented;

use crate::{apu::APU, ppu::PPU, rom::ROM, serial::Serial, timer::Timer};

// Holds everything that the CPU has to interact with
// Also gets ticked by the CPU struct
pub struct Components {
    rom: Box<dyn ROM>,
    bootrom: Bootrom,
    ppu: PPU,
    apu: APU,
    timer: Timer,
    serial: Serial,
    wram: [u8; 0x2000],
    hram: [u8; 0x80],

    bootrom_disabled: bool,
    pub interrupt_flag: u8,
    pub interrupt_enable: u8,
}

impl Components {
    pub fn new(rom: Box<dyn ROM>) -> Self {
        Self {
            rom,
            bootrom: Default::default(),
            ppu: Default::default(),
            apu: Default::default(),
            timer: Default::default(),
            serial: Default::default(),

            // TODO: Fill this with a psuedo-random pattern
            wram: [0u8; 0x2000],
            hram: [0u8; 0x80],

            bootrom_disabled: false,
            interrupt_flag: 0,
            interrupt_enable: 0,
        }
    }

    // Processes one M-cycle/four T-cycles
    pub fn tick(&mut self) {
        // TODO: What order is this supposed to be in? Does it even matter?
        self.interrupt_flag |= (self.timer.tick() as u8) << 2;
        self.ppu.tick();
        self.apu.tick();
    }

    // Ticks components by one M-cycle, then reads a byte from an address
    pub fn read(&mut self, addr: u16) -> u8 {
        self.tick();
        self.read_passive(addr)
    }

    // Reads a byte from an address without ticking
    pub fn read_passive(&mut self, addr: u16) -> u8 {
        match addr {
            // Cart ROM/Bootrom
            0x0000..=0x7FFF => {
                if addr < 0x100 && !self.bootrom_disabled {
                    self.bootrom.read(addr as u8)
                } else {
                    self.rom.read_rom(addr)
                }
            }
            // VRAM
            0x8000..=0x9FFF => self.ppu.read_vram(addr),
            // Cart RAM
            0xA000..=0xBFFF => self.rom.read_ram(addr),
            // WRAM
            0xC000..=0xDFFF => self.wram[addr as usize & 0x1FFF],
            // Echo RAM
            0xE000..=0xFDFF => self.wram[addr as usize & 0x1FFF],
            // OAM
            0xFE00..=0xFEFF => self.ppu.read_oam(addr),
            // I/O region
            0xFF00..=0xFF7F => self.read_io(addr),
            // HRAM
            0xFF80..=0xFFFE => self.hram[addr as usize & 0x7F],
            // Interrupt enable
            // TODO: Do invalid interrupt bits get set/cleared?
            0xFFFF => self.interrupt_enable,
        }
    }

    // Handles I/O region (0xFFxx) reads
    fn read_io(&mut self, addr: u16) -> u8 {
        match addr as u8 {
            // P1/JOYP: Joypad
            0x00 => 0x00,
            // Serial transfer data
            0x01 => self.serial.read_sb(),
            // Serial transfer control
            0x02 => self.serial.read_sc(),
            // Divider register
            0x04 => self.timer.read_div(),
            // Timer counter
            0x05 => self.timer.read_tima(),
            // Timer modulo
            0x06 => self.timer.read_tma(),
            // Timer control
            0x07 => self.timer.read_tac(),
            // Interrupt flag
            0x0F => self.interrupt_flag,
            // NR52: Sound on/off
            0x26 => self.apu.read_nr52(),
            // LCD control
            0x40 => self.ppu.read_lcdc(),
            // LCD status
            0x41 => self.ppu.read_stat(),
            // Viewport Y position
            0x42 => self.ppu.read_scy(),
            // Viewport X position
            0x43 => self.ppu.read_scx(),
            // LCD Y coordinate
            0x44 => self.ppu.read_ly(),
            // LCD Y compare
            0x45 => self.ppu.read_lyc(),
            // BG palette data
            0x47 => self.ppu.read_bgp(),
            // KEY1
            0x4D => 0xFF,
            // Bootrom disable
            0x50 => self.bootrom_disabled as u8,
            x => unimplemented!("Unmapped I/O read at 0xff{x:02x}"),
        }
    }

    // Ticks components by one M-cycle, then writes a byte to an address
    pub fn write(&mut self, addr: u16, val: u8) {
        self.tick();
        self.write_passive(addr, val)
    }

    // Writes a byte to an address without ticking
    pub fn write_passive(&mut self, addr: u16, val: u8) {
        match addr {
            // ROM banks/Bootrom
            0x0000..=0x7FFF => self.rom.write_rom(addr, val),
            // VRAM
            0x8000..=0x9FFF => self.ppu.write_vram(addr, val),
            // Cart RAM
            0xA000..=0xBFFF => self.rom.write_ram(addr, val),
            // WRAM
            0xC000..=0xDFFF => self.wram[addr as usize & 0x1FFF] = val,
            // Echo RAM
            0xE000..=0xFDFF => self.wram[addr as usize & 0x1FFF] = val,
            // OAM
            0xFE00..=0xFEFF => self.ppu.write_oam(addr, val),
            // I/O region
            0xFF00..=0xFF7F => self.write_io(addr, val),
            // HRAM
            0xFF80..=0xFFFE => self.hram[addr as usize & 0x7F] = val,
            // Interrupt enable
            0xFFFF => self.interrupt_enable = val,
        }
    }

    // Handles I/O region (0xFFxx) writes
    fn write_io(&mut self, addr: u16, val: u8) {
        match addr as u8 {
            // P1/JOYP: Joypad
            0x00 => (),
            // Serial transfer data
            0x01 => self.serial.write_sb(val),
            // Serial transfer control
            0x02 => self.serial.write_sc(val),
            // Divider register
            0x04 => self.timer.write_div(val),
            // Timer counter
            0x05 => self.timer.write_tima(val),
            // Timer modulo
            0x06 => self.timer.write_tma(val),
            // Timer control
            0x07 => self.timer.write_tac(val),
            // Interrupt flag
            0x0F => self.interrupt_flag = val,
            // NR11: Channel 1 length timer & duty cycle
            0x11 => self.apu.write_nr11(val),
            // NR12: Channel 1 volume & envelope
            0x12 => self.apu.write_nr12(val),
            // NR13: Channel 1 wavelength low
            0x13 => self.apu.write_nr13(val),
            // NR14: Channel 1 wavelength high & control
            0x14 => self.apu.write_nr14(val),
            // NR50: Master volume & VIN panning
            0x24 => self.apu.write_nr50(val),
            // NR51: Sound panning
            0x25 => self.apu.write_nr51(val),
            // NR52: Sound on/off
            0x26 => self.apu.write_nr52(val),
            // LCD control
            0x40 => self.ppu.write_lcdc(val),
            // LCD status
            0x41 => self.ppu.write_stat(val),
            // Viewport Y position
            0x42 => self.ppu.write_scy(val),
            // Viewport X postion
            0x43 => self.ppu.write_scx(val),
            // LCD Y compare
            0x45 => self.ppu.write_lyc(val),
            // BG palette data
            0x47 => self.ppu.write_bgp(val),
            // Bootrom disable (can only be set once!)
            0x50 => self.bootrom_disabled |= val != 0,
            x => unimplemented!("Unmapped I/O write at 0xff{x:02x}"),
        }
    }
}

// Holds the DMG bootrom
// TODO: Load this from a file
struct Bootrom {
    rom: [u8; 0x100],
}

impl Default for Bootrom {
    fn default() -> Self {
        Self {
            rom: *include_bytes!("../dmg_rom.bin"),
        }
    }
}

impl Bootrom {
    pub fn read(&self, addr: u8) -> u8 {
        self.rom[addr as usize]
    }
}
