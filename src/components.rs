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
        }
    }

    // Processes one M-cycle/four T-cycles
    pub fn tick(&mut self) {
        // TODO: What order is this supposed to be in? Does it even matter?
        self.timer.tick();
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
            0xFFFF => unimplemented!("No interrupts yet"),
        }
    }

    // Handles I/O region (0xFFxx) reads
    fn read_io(&mut self, addr: u16) -> u8 {
        match addr as u8 {
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
            0xFFFF => unimplemented!("No interrupts yet"),
        }
    }

    // Handles I/O region (0xFFxx) writes
    fn write_io(&mut self, addr: u16, val: u8) {
        match addr as u8 {
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
