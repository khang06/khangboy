use crate::{components::Components, util::BitIndex};

#[cfg(feature = "gb_doctor")]
use std::io::Write;

#[derive(Default, Debug)]
pub struct CPU {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8, // Flags
    h: u8,
    l: u8,
    ime: bool,        // Interrupts
    ime_queued: bool, // The effects of EI are delayed by one instruction
    halted: bool,
    halt_bug: bool,

    sp: u16,
    pc: u16,

    opcode: u8, // Fetched during execution of last instruction
    cycle: u64, // Counted in M-cycles

    #[cfg(feature = "gb_doctor")]
    log: Option<std::io::BufWriter<std::fs::File>>,
}

enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

impl Reg16 {
    #[inline]
    pub fn read(&self, cpu: &CPU) -> u16 {
        match self {
            Reg16::AF => (cpu.a as u16) << 8 | cpu.f as u16,
            Reg16::BC => (cpu.b as u16) << 8 | cpu.c as u16,
            Reg16::DE => (cpu.d as u16) << 8 | cpu.e as u16,
            Reg16::HL => (cpu.h as u16) << 8 | cpu.l as u16,
            Reg16::SP => cpu.sp,
            Reg16::PC => cpu.pc,
        }
    }

    #[inline]
    pub fn write(&self, cpu: &mut CPU, val: u16) {
        match self {
            Reg16::AF => {
                // It's important to clear any extraneous data from the flag register
                cpu.f = (val as u8) & 0xF0;
                cpu.a = (val >> 8) as u8;
            }
            Reg16::BC => {
                cpu.c = val as u8;
                cpu.b = (val >> 8) as u8;
            }
            Reg16::DE => {
                cpu.e = val as u8;
                cpu.d = (val >> 8) as u8;
            }
            Reg16::HL => {
                cpu.l = val as u8;
                cpu.h = (val >> 8) as u8;
            }
            Reg16::SP => cpu.sp = val,
            Reg16::PC => cpu.pc = val,
        }
    }
}

enum Reg8<'a> {
    A,
    B,
    C,
    D,
    E,
    //F, // Never gets used?
    H,
    L,
    HLPtr(&'a mut Components),
}

impl Reg8<'_> {
    #[inline]
    pub fn read(&mut self, cpu: &mut CPU) -> u8 {
        match self {
            Reg8::A => cpu.a,
            Reg8::B => cpu.b,
            Reg8::C => cpu.c,
            Reg8::D => cpu.d,
            Reg8::E => cpu.e,
            Reg8::H => cpu.h,
            Reg8::L => cpu.l,
            Reg8::HLPtr(com) => {
                let hl = Reg16::HL.read(cpu);
                cpu.read8(com, hl)
            }
        }
    }

    #[inline]
    pub fn write(&mut self, cpu: &mut CPU, val: u8) {
        match self {
            Reg8::A => cpu.a = val,
            Reg8::B => cpu.b = val,
            Reg8::C => cpu.c = val,
            Reg8::D => cpu.d = val,
            Reg8::E => cpu.e = val,
            Reg8::H => cpu.h = val,
            Reg8::L => cpu.l = val,
            Reg8::HLPtr(com) => {
                let hl = Reg16::HL.read(cpu);
                cpu.write8(com, hl, val);
            }
        }
    }
}

impl CPU {
    #[cfg(feature = "gb_doctor")]
    pub fn new() -> Self {
        Self {
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
            log: Some(std::io::BufWriter::new(
                std::fs::File::create("cpu.log").unwrap(),
            )),
            ..Default::default()
        }
    }

    #[cfg(not(feature = "gb_doctor"))]
    pub fn new() -> Self {
        Default::default()
    }

    // Steps by one instruction
    // Also ticks every component accordingly depending on the timing
    // M-cycle (4 T-cycles) granularity, but most other GB emulators have that too
    pub fn step(&mut self, com: &mut Components) {
        // Handle interrupts
        let interrupts = com.interrupt_enable & com.interrupt_flag;
        if self.ime && interrupts != 0 {
            for i in 0..=5 {
                if interrupts.test(i) {
                    com.interrupt_flag = com.interrupt_flag.set(i, false);
                    self.ime = false;
                    self.halted = false;
                    self.run_cycle(com);
                    self.push_val(com, self.pc.wrapping_sub(1));
                    self.pc = (0x40 + i * 8) as u16;
                    self.opcode = self.fetch8(com);
                    break;
                }
            }
        }

        // Handle halted state
        if self.halted {
            if interrupts == 0 {
                self.run_cycle(com);
                return;
            }
            self.halted = false;
        }

        // The effects of EI are delayed by one instruction
        if self.ime_queued {
            self.ime = true;
            self.ime_queued = false;
        }

        // Run opcode
        // This massive 256-case match statement is generated at compile-time
        // See build.rs
        include!("opcodes.inl");

        #[cfg(feature = "gb_doctor")]
        writeln!(
            self.log.as_mut().unwrap(),
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.a,
            self.f,
            self.b,
            self.c,
            self.d,
            self.e,
            self.h,
            self.l,
            self.sp,
            self.pc,
            com.read_passive(self.pc),
            com.read_passive(self.pc.wrapping_add(1)),
            com.read_passive(self.pc.wrapping_add(2)),
            com.read_passive(self.pc.wrapping_add(3))
        )
        .unwrap();

        //println!("{self:?}");

        // Fetch the next opcode
        // This happens in the same M-cycle as the last execution cycle
        if !self.halt_bug {
            self.opcode = self.fetch8(com);
        }
        self.halt_bug = false;

        if self.halted && !self.ime && (com.interrupt_enable & com.interrupt_flag) != 0 {
            self.halt_bug = true;
            self.halted = false;
        }
    }

    // Handles 0xCB prefix bit arithmetic opcodes
    fn handle_cb(&mut self, com: &mut Components) {
        let opcode = self.fetch8(com);
        let mut target = match opcode & 7 {
            0x0 => Reg8::B,
            0x1 => Reg8::C,
            0x2 => Reg8::D,
            0x3 => Reg8::E,
            0x4 => Reg8::H,
            0x5 => Reg8::L,
            0x6 => Reg8::HLPtr(com),
            0x7 => Reg8::A,
            _ => unreachable!(),
        };
        match opcode & 0xF8 {
            // RLC n
            0x00 => {
                let val = target.read(self);
                let res = val.rotate_left(1);
                self.set_flags(res == 0, false, false, (res & 1) != 0);
                target.write(self, res);
            }
            // RRC n
            0x08 => {
                let val = target.read(self);
                let res = val.rotate_right(1);
                self.set_flags(res == 0, false, false, (res & 0x80) != 0);
                target.write(self, res);
            }
            // RL r
            0x10 => {
                let val = target.read(self);
                let res = (val << 1) | ((self.get_c() as u8) & 1);
                self.set_flags(res == 0, false, false, (val & 0x80) != 0);
                target.write(self, res);
            }
            // RR r
            0x18 => {
                let val = target.read(self);
                let res = (val >> 1) | ((self.get_c() as u8) << 7);
                self.set_flags(res == 0, false, false, (val & 1) != 0);
                target.write(self, res);
            }
            // SLA r
            0x20 => {
                let val = target.read(self);
                let res = val << 1;
                self.set_flags(res == 0, false, false, (val & 0x80) != 0);
                target.write(self, res);
            }
            // SRA r
            0x28 => {
                let val = target.read(self);
                let res = (val as i8) >> 1;
                self.set_flags(res == 0, false, false, (val & 1) != 0);
                target.write(self, res as u8);
            }
            // SWAP r
            0x30 => {
                let val = target.read(self);
                let res = (val & 0xF0) >> 4 | (val & 0xF) << 4;
                self.set_flags(res == 0, false, false, false);
                target.write(self, res);
            }
            // SRL r
            0x38 => {
                let val = target.read(self);
                let res = val >> 1;
                self.set_flags(res == 0, false, false, (val & 1) != 0);
                target.write(self, res);
            }
            // BIT n,r
            0x40..=0x78 => {
                let n = (opcode >> 3) & 7;
                let val = target.read(self);
                self.set_z(!val.test(n));
                self.set_n(false);
                self.set_h(true);
            }
            // RES n,r
            0x80..=0xB8 => {
                let n = (opcode >> 3) & 7;
                let val = target.read(self);
                target.write(self, val & !(1 << n));
            }
            // SET n,r
            0xC0..=0xF8 => {
                let n = (opcode >> 3) & 7;
                let val = target.read(self);
                target.write(self, val | (1 << n));
            }
            _ => unimplemented!("Unhandled 0xCB opcode 0x{opcode:02x}"),
        }
    }

    // Runs one M-cycle
    #[inline]
    fn run_cycle(&mut self, com: &mut Components) {
        self.cycle += 1;
        com.tick();
    }

    // Reads an 8-bit value from an address
    #[inline]
    fn read8(&mut self, com: &mut Components, addr: u16) -> u8 {
        self.cycle += 1;
        com.read(addr)
    }

    // Writes an 8-bit value to an address
    #[inline]
    fn write8(&mut self, com: &mut Components, addr: u16, val: u8) {
        self.cycle += 1;
        com.write(addr, val)
    }

    // Reads a little-endian 16-bit integer from an address
    #[inline]
    pub fn read16(&mut self, com: &mut Components, addr: u16) -> u16 {
        self.read8(com, addr) as u16 | (self.read8(com, addr.wrapping_add(1)) as u16) << 8
    }

    // Reads the byte at PC and increments it
    #[inline]
    fn fetch8(&mut self, com: &mut Components) -> u8 {
        let ret = self.read8(com, self.pc);
        self.pc = self.pc.wrapping_add(1);
        ret
    }

    // Reads 2 bytes at PC and increments it
    // TODO: This should probably be more granular, but does it really matter?
    #[inline]
    fn fetch16(&mut self, com: &mut Components) -> u16 {
        let ret = self.read16(com, self.pc);
        self.pc = self.pc.wrapping_add(2);
        ret
    }

    // Helper function to set all 4 flags at once
    #[inline]
    fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.set_z(z);
        self.set_n(n);
        self.set_h(h);
        self.set_c(c);
    }

    // Gets the zero flag
    #[inline]
    fn get_z(&self) -> bool {
        self.f.test(7)
    }

    // Sets the zero flag
    #[inline]
    fn set_z(&mut self, val: bool) {
        self.f = self.f.set(7, val);
    }

    // Gets the subtraction flag
    #[inline]
    fn get_n(&self) -> bool {
        self.f.test(6)
    }

    // Sets the subtraction flag
    #[inline]
    fn set_n(&mut self, val: bool) {
        self.f = self.f.set(6, val);
    }

    // Gets the half carry flag
    #[inline]
    fn get_h(&self) -> bool {
        self.f.test(5)
    }

    // Sets the half carry flag
    #[inline]
    fn set_h(&mut self, val: bool) {
        self.f = self.f.set(5, val);
    }

    // Gets the carry flag
    #[inline]
    fn get_c(&self) -> bool {
        self.f.test(4)
    }

    // Sets the carry flag
    #[inline]
    fn set_c(&mut self, val: bool) {
        self.f = self.f.set(4, val);
    }

    // Pushes a 16-bit integer onto the stack
    #[inline]
    fn push_val(&mut self, com: &mut Components, val: u16) {
        self.run_cycle(com);
        self.write8(com, self.sp.wrapping_sub(1), (val >> 8) as u8);
        self.write8(com, self.sp.wrapping_sub(2), val as u8);
        self.sp = self.sp.wrapping_sub(2);
    }

    // Handles LD r16, d16
    #[inline]
    fn ld_r16_d16(&mut self, com: &mut Components, target: Reg16) {
        let imm = self.fetch16(com);
        target.write(self, imm);
    }

    // Handles LD r8, d8
    fn ld_r8_d8(&mut self, imm: u8, mut target: Reg8) {
        target.write(self, imm);
    }

    // Handles the INC r8 instruction and its flags
    #[inline]
    fn inc_r8(&mut self, mut target: Reg8) {
        let res = target.read(self).wrapping_add(1);
        target.write(self, res);
        self.set_z(res == 0);
        self.set_n(false);
        self.set_h(res & 0xF == 0);
    }

    // Handles the DEC r8 instruction and its flags
    #[inline]
    fn dec_r8(&mut self, mut target: Reg8) {
        let res = target.read(self).wrapping_sub(1);
        target.write(self, res);
        self.set_z(res == 0);
        self.set_n(true);
        self.set_h(res & 0xF == 0xF);
    }

    // Handles the INC r16 instruction and its flags
    #[inline]
    fn inc_r16(&mut self, com: &mut Components, target: Reg16) {
        self.run_cycle(com);
        target.write(self, target.read(self).wrapping_add(1));
    }

    // Handles the DEC r16 instruction and its flags
    #[inline]
    fn dec_r16(&mut self, com: &mut Components, target: Reg16) {
        self.run_cycle(com);
        target.write(self, target.read(self).wrapping_sub(1));
    }

    // Handles the XOR r8 instruction and its flags
    #[inline]
    fn xor_r8(&mut self, mut target: Reg8) {
        self.a ^= target.read(self);
        self.set_flags(self.a == 0, false, false, false);
    }

    // Handles the OR r8 instruction and its flags
    #[inline]
    fn or_r8(&mut self, mut target: Reg8) {
        self.a |= target.read(self);
        self.set_flags(self.a == 0, false, false, false);
    }

    // Handles the RST n instruction
    #[inline]
    fn rst_n(&mut self, com: &mut Components, addr: u16) {
        self.push_val(com, self.pc);
        self.pc = addr;
    }

    // Handles the PUSH r16 instruction
    #[inline]
    fn push_r16(&mut self, com: &mut Components, reg: Reg16) {
        let val = reg.read(self);
        self.push_val(com, val);
    }

    // Handles the POP r16 instruction
    #[inline]
    fn pop_r16(&mut self, com: &mut Components, reg: Reg16) {
        let val = self.read16(com, self.sp);
        reg.write(self, val);
        self.sp = self.sp.wrapping_add(2);
    }

    // Handles the RET instruction and its CC varients
    #[inline]
    fn ret(&mut self, com: &mut Components) {
        self.pop_r16(com, Reg16::PC);
        self.run_cycle(com);
    }

    // Handles subtraction from A with or without carry and its flags
    // This doesn't write to A because it's also used for the CP instructions
    // Algorithm from Mooneye-GB because carry flags are painful
    #[inline]
    fn alu_sub(&mut self, value: u8, carry: bool) -> u8 {
        let result = self.a.wrapping_sub(value).wrapping_sub(carry as u8);
        self.set_flags(
            result == 0,
            true,
            (self.a & 0xF)
                .wrapping_sub(value & 0xF)
                .wrapping_sub(carry as u8)
                & 0x10
                != 0,
            (self.a as u16) < (value as u16) + (carry as u16),
        );
        result
    }

    // Handles the SUB r8 instruction and its flags
    #[inline]
    fn sub_r8(&mut self, mut target: Reg8) {
        let val = target.read(self);
        self.a = self.alu_sub(val, false);
    }

    // Handles the SBC A, r8 instruction and its flags
    #[inline]
    fn sbc_a_r8(&mut self, mut target: Reg8) {
        let val = target.read(self);
        self.a = self.alu_sub(val, self.get_c());
    }

    // Handles the CP r8 instruction and its flags
    #[inline]
    fn cp_r8(&mut self, mut target: Reg8) {
        let val = target.read(self);
        self.alu_sub(val, false);
    }

    // Handles the AND r8 instruction and its flags
    #[inline]
    fn and_a_r8(&mut self, mut target: Reg8) {
        let val = target.read(self);
        self.a &= val;
        self.set_flags(self.a == 0, false, true, false);
    }

    // Handles the ADD A, r8 instruction and its flags
    #[inline]
    fn add_a_r8(&mut self, mut target: Reg8) {
        let val = target.read(self);
        let (res, carry) = self.a.overflowing_add(val);
        self.set_flags(
            res == 0,
            false,
            (((self.a & 0xF) + (val & 0xF)) & 0x10) != 0,
            carry,
        );
        self.a = res;
    }

    // Handles the ADC A, r8 instruction and its flags
    #[inline]
    fn adc_a_r8(&mut self, mut target: Reg8) {
        let carry = self.get_c() as u8;
        let val = target.read(self);
        let res = self.a.wrapping_add(val).wrapping_add(carry);
        self.set_flags(
            res == 0,
            false,
            (((self.a & 0xF) + (val & 0xF) + carry) & 0x10) != 0,
            (res as u16) < (self.a as u16) + (val as u16) + (carry as u16),
        );
        self.a = res;
    }

    #[inline]
    fn add_hl_r16(&mut self, com: &mut Components, reg: Reg16) {
        self.run_cycle(com);
        let val = reg.read(self);
        let (res, carry) = Reg16::HL.read(self).overflowing_add(val);
        self.set_n(false);
        self.set_h((((Reg16::HL.read(self) & 0xFFF) + (val & 0xFFF)) & 0x1000) != 0);
        self.set_c(carry);
        Reg16::HL.write(self, res);
    }
}
