pub trait ROM {
    // 0x0000 to 0x7FFF
    fn read_rom(&mut self, addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, val: u8);

    // 0xA000 to 0xBFFF
    fn read_ram(&mut self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, val: u8);
}

pub fn rom_from_bytes(rom: &[u8]) -> Result<Box<dyn ROM>, String> {
    // Every cartridge is at least 0x8000 bytes long
    if rom.len() < 0x8000 {
        return Err("ROM is too small to be valid".into());
    }

    // Match based on mapper
    match rom[0x147] {
        // No mapper
        0x00 => Ok(Box::new(NoMapper {
            rom: rom[..0x8000].try_into().unwrap(),
        })),
        // MBC1
        // TODO: Actually check the header for bank count
        0x01..=0x03 => {
            let mut mbc1 = MBC1::default();
            for x in rom.chunks_exact(0x4000) {
                mbc1.rom_banks.push(x.try_into().unwrap());
            }
            if rom[0x147] != 0x01 {
                let banks = match rom[0x149] {
                    0 => 0,
                    2 => 1,
                    3 => 4,
                    4 => 16,
                    5 => 8,
                    x => unimplemented!("Unknown bank count {x}"),
                };
                mbc1.ram_banks.resize(banks, [0u8; 0x2000]);
            }

            Ok(Box::new(mbc1))
        }
        x => Err(format!("Unhandled mapper 0x{:02X}", x)),
    }
}

pub struct NoMapper {
    rom: [u8; 0x8000],
}

impl ROM for NoMapper {
    fn read_rom(&mut self, addr: u16) -> u8 {
        self.rom[addr as usize & 0x7FFF]
    }
    fn write_rom(&mut self, _addr: u16, _val: u8) {}

    fn read_ram(&mut self, _addr: u16) -> u8 {
        // TODO: Verify that this is actually the value that gets returned
        0xFF
    }
    fn write_ram(&mut self, _addr: u16, _val: u8) {}
}

#[derive(Default)]
pub struct MBC1 {
    rom_banks: Vec<[u8; 0x4000]>,
    ram_banks: Vec<[u8; 0x2000]>,

    ram_enabled: bool,
    rom_bank_idx: u8,
    ram_bank_idx: u8,
}

impl ROM for MBC1 {
    fn read_rom(&mut self, addr: u16) -> u8 {
        let bank = if addr < 0x4000 {
            &self.rom_banks[0]
        } else {
            &self.rom_banks[self.rom_bank_idx.max(1) as usize % self.rom_banks.len()]
        };
        bank[addr as usize & 0x3FFF]
    }

    fn write_rom(&mut self, addr: u16, val: u8) {
        match addr & 0x7FFF {
            // RAM Enable
            0x0000..=0x1FFF => self.ram_enabled = val & 0xF == 0xA,
            // ROM Bank Number
            0x2000..=0x3FFF => self.rom_bank_idx = (self.rom_bank_idx & !0x1F) | (val & 0x1F),
            // RAM Bank Number or Upper ROM Bank Number
            0x4000..=0x5FFF => {
                if self.rom_banks.len() < 32 {
                    self.ram_bank_idx = val & 3;
                } else {
                    self.rom_bank_idx = (self.rom_bank_idx & 0x1F) | ((val & 3) << 5)
                }
            }
            // Banking Mode Select
            0x6000..=0x7FFF => {
                if (val & 1) != 0 {
                    unimplemented!("Advanced banking mode not implemented");
                }
            }
            _ => unreachable!(),
        }
    }

    fn read_ram(&mut self, addr: u16) -> u8 {
        if self.ram_enabled && !self.ram_banks.is_empty() {
            let bank = &self.ram_banks[self.ram_bank_idx.max(1) as usize % self.ram_banks.len()];
            bank[addr as usize & 0x1FFF]
        } else {
            // TODO: Verify that this is actually the value that gets returned
            0xFF
        }
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        if self.ram_enabled && !self.ram_banks.is_empty() {
            let bank_count = self.ram_banks.len();
            let bank = &mut self.ram_banks[self.ram_bank_idx.max(1) as usize % bank_count];
            bank[addr as usize & 0x1FFF] = val;
        }
    }
}
