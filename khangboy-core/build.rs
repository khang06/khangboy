use std::{format, fs::File, io::Write};

// This generates src/opcodes.inl
fn main() {
    let table = gen_table();
    write_opcodes(&mut File::create("src/opcodes.inl").unwrap(), &table);
}

// Generate the opcode table
// https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
// TODO: Using string manipulation for this is a bit messy
fn gen_table() -> [Option<String>; 256] {
    // Weird workaround https://stackoverflow.com/questions/28656387/initialize-a-large-fixed-size-array-with-non-copy-types
    const INIT: Option<String> = None;
    let mut out = [INIT; 256];

    let reg8_enum = ["B", "C", "D", "E", "H", "L", "HLPtr(com)", "A"];
    let reg16_enum = ["BC", "DE", "HL", "AF"];

    // NOP
    // 0x00
    set_opcode(&mut out, 0x00, "");

    // LD r16, d16
    // 0x01, 0x11, 0x21, 0x31
    for (i, x) in ["BC", "DE", "HL", "SP"].iter().enumerate() {
        set_opcode(
            &mut out,
            (i << 4) | 0x01,
            &format!("self.ld_r16_d16(com, Reg16::{x});"),
        );
    }

    // XOR r8
    // 0xA8 to 0xAF
    for (i, x) in reg8_enum.iter().enumerate() {
        set_opcode(&mut out, 0xA8 | i, &format!("self.xor_r8(Reg8::{x})"));
    }

    // LD (r16), A
    // 0x02, 0x12, 0x22, 0x32
    set_opcode(
        &mut out,
        0x02,
        "let addr = Reg16::BC.read(self); self.write8(com, addr, self.a);",
    );
    set_opcode(
        &mut out,
        0x12,
        "let addr = Reg16::DE.read(self); self.write8(com, addr, self.a);",
    );
    set_opcode(&mut out, 0x22, "let addr = Reg16::HL.read(self); self.write8(com, addr, self.a); Reg16::HL.write(self, addr.wrapping_add(1));");
    set_opcode(&mut out, 0x32, "let addr = Reg16::HL.read(self); self.write8(com, addr, self.a); Reg16::HL.write(self, addr.wrapping_sub(1));");

    // LD A, (r16)
    // 0x0A, 0x1A, 0x2A, 0x3A
    set_opcode(
        &mut out,
        0x0A,
        "let addr = Reg16::BC.read(self); self.a = self.read8(com, addr);",
    );
    set_opcode(
        &mut out,
        0x1A,
        "let addr = Reg16::DE.read(self); self.a = self.read8(com, addr);",
    );
    set_opcode(&mut out, 0x2A, "let addr = Reg16::HL.read(self); self.a = self.read8(com, addr); Reg16::HL.write(self, addr.wrapping_add(1));");
    set_opcode(&mut out, 0x3A, "let addr = Reg16::HL.read(self); self.a = self.read8(com, addr); Reg16::HL.write(self, addr.wrapping_sub(1));");

    // 0xCB prefix
    set_opcode(&mut out, 0xCB, "self.handle_cb(com);");

    // JR cc, r8
    // 0x20, 0x30, 0x28, 0x38
    set_opcode(&mut out, 0x20, &jr_cc("!self.get_z()"));
    set_opcode(&mut out, 0x30, &jr_cc("!self.get_c()"));
    set_opcode(&mut out, 0x28, &jr_cc("self.get_z()"));
    set_opcode(&mut out, 0x38, &jr_cc("self.get_c()"));

    // LD r8, d8
    // 0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E
    for (i, x) in reg8_enum.iter().enumerate() {
        let hi = (i / 2) << 4;
        let lo = if i % 2 == 0 { 0x06 } else { 0x0E };
        set_opcode(
            &mut out,
            hi | lo,
            &format!("let imm = self.fetch8(com); self.ld_r8_d8(imm, Reg8::{x});"),
        );
    }

    // LD (C), A
    // 0xE2
    set_opcode(
        &mut out,
        0xE2,
        "self.write8(com, 0xFF00 | self.c as u16, self.a)",
    );

    // INC r8
    // 0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x34, 0x3C
    for (i, x) in reg8_enum.iter().enumerate() {
        let hi = (i / 2) << 4;
        let lo = if i % 2 == 0 { 0x04 } else { 0x0C };
        set_opcode(&mut out, hi | lo, &format!("self.inc_r8(Reg8::{x});"));
    }

    // DEC r8
    // 0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x35, 0x3D
    for (i, x) in reg8_enum.iter().enumerate() {
        let hi = (i / 2) << 4;
        let lo = if i % 2 == 0 { 0x05 } else { 0x0D };
        set_opcode(&mut out, hi | lo, &format!("self.dec_r8(Reg8::{x});"));
    }

    // LD r8, r8
    // 0x40 to 0x7F except 0x76
    for (i, x) in reg8_enum.iter().enumerate() {
        let hi = 0x40 + ((i / 2) << 4);
        for (j, y) in reg8_enum.iter().enumerate() {
            let lo = if i % 2 == 0 { 0x00 } else { 0x08 } + j;
            if (hi | lo) != 0x76 {
                set_opcode(
                    &mut out,
                    hi | lo,
                    &format!("let val = Reg8::{y}.read(self); Reg8::{x}.write(self, val);"),
                );
            }
        }
    }

    // LDH (a8), A
    // 0xE0
    set_opcode(
        &mut out,
        0xE0,
        "let n = self.fetch8(com); self.write8(com, 0xFF00 | n as u16, self.a);",
    );

    // LDH A, (a8)
    // 0xF0
    set_opcode(
        &mut out,
        0xF0,
        "let n = self.fetch8(com); self.a = self.read8(com, 0xFF00 | n as u16);",
    );

    // CALL a16
    // 0xCD
    set_opcode(
        &mut out,
        0xCD,
        "let nn = self.fetch16(com); self.push_r16(com, Reg16::PC); self.pc = nn;",
    );

    // PUSH r16
    // 0xC5, 0xD5, 0xE5, 0xF5
    for (i, x) in reg16_enum.iter().enumerate() {
        set_opcode(
            &mut out,
            0xC5 | (i << 4),
            &format!("self.push_r16(com, Reg16::{x});"),
        );
    }

    // POP r16
    // 0xC1, 0xD1, 0xE1, 0xF1
    for (i, x) in reg16_enum.iter().enumerate() {
        set_opcode(
            &mut out,
            0xC1 | (i << 4),
            &format!("self.pop_r16(com, Reg16::{x});"),
        );
    }

    // RLA
    // 0x17
    set_opcode(&mut out, 0x17, "let carry = (self.a & 0x80) != 0; self.a = (self.a << 1) | ((self.get_c() as u8) & 1); self.set_flags(false, false, false, carry);");

    // RLCA
    // 0x07
    set_opcode(
        &mut out,
        0x07,
        "self.a = self.a.rotate_left(1); self.set_flags(false, false, false, (self.a & 1) != 0);",
    );

    // RRA
    // 0x1F
    set_opcode(&mut out, 0x1F, "let carry = (self.a & 1) != 0; self.a = (self.a >> 1) | ((self.get_c() as u8) << 7); self.set_flags(false, false, false, carry);");

    // RRCA
    // 0x0F
    set_opcode(
        &mut out,
        0x0F,
        "self.a = self.a.rotate_right(1); self.set_flags(false, false, false, (self.a & 0x80) != 0);",
    );

    // INC r16
    // 0x03, 0x13, 0x23, 0x33
    for (i, x) in ["BC", "DE", "HL", "SP"].iter().enumerate() {
        set_opcode(
            &mut out,
            (i << 4) | 0x03,
            &format!("self.inc_r16(com, Reg16::{x});"),
        );
    }

    // DEC r16
    // 0x0B, 0x1B, 0x2B, 0x3B
    for (i, x) in ["BC", "DE", "HL", "SP"].iter().enumerate() {
        set_opcode(
            &mut out,
            (i << 4) | 0x0B,
            &format!("self.dec_r16(com, Reg16::{x});"),
        );
    }

    // RET
    // 0xC9
    set_opcode(&mut out, 0xC9, "self.ret(com);");

    // RETI
    // 0xD9
    set_opcode(
        &mut out,
        0xD9,
        "self.ime = true; self.pop_r16(com, Reg16::PC); self.run_cycle(com);",
    );

    // RET cc
    // 0xC0, 0xC8, 0xD0, 0xD8
    set_opcode(&mut out, 0xC0, &ret_cc("!self.get_z()"));
    set_opcode(&mut out, 0xD0, &ret_cc("!self.get_c()"));
    set_opcode(&mut out, 0xC8, &ret_cc("self.get_z()"));
    set_opcode(&mut out, 0xD8, &ret_cc("self.get_c()"));

    // CP d8
    // 0xFE
    set_opcode(
        &mut out,
        0xFE,
        "let n = self.fetch8(com); self.alu_sub(n, false);",
    );

    // LD (a16), A
    // 0xEA
    set_opcode(
        &mut out,
        0xEA,
        "let addr = self.fetch16(com); self.write8(com, addr, self.a);",
    );

    // LD A, (a16)
    // 0xFA
    set_opcode(
        &mut out,
        0xFA,
        "let addr = self.fetch16(com); self.a = self.read8(com, addr);",
    );

    // JR d8
    // 0x18
    set_opcode(
        &mut out,
        0x18,
        "let offset = self.fetch8(com) as i8; self.run_cycle(com); self.pc = self.pc.wrapping_add_signed(offset as i16);",
    );

    // SUB r8
    // 0x90 to 0x97
    for (i, x) in reg8_enum.iter().enumerate() {
        set_opcode(&mut out, 0x90 | i, &format!("self.sub_r8(Reg8::{x});"));
    }

    // CP r8
    // 0xB8 to 0xBF
    for (i, x) in reg8_enum.iter().enumerate() {
        set_opcode(&mut out, 0xB8 | i, &format!("self.cp_r8(Reg8::{x});"));
    }

    // ADD A, r8
    // 0x80 to 0x87
    for (i, x) in reg8_enum.iter().enumerate() {
        set_opcode(&mut out, 0x80 | i, &format!("self.add_a_r8(Reg8::{x});"));
    }

    // OR r8
    // 0xB0 to 0xB7
    for (i, x) in reg8_enum.iter().enumerate() {
        set_opcode(&mut out, 0xB0 | i, &format!("self.or_r8(Reg8::{x});"));
    }

    // JP a16
    // 0xC3
    set_opcode(
        &mut out,
        0xC3,
        "let nn = self.fetch16(com); self.run_cycle(com); self.pc = nn;",
    );

    // DI
    // 0xF3
    set_opcode(&mut out, 0xF3, "self.ime_queued = false; self.ime = false;");

    // EI
    // 0xFB
    set_opcode(&mut out, 0xFB, "self.ime_queued = true;");

    // AND d8
    // 0xE6
    set_opcode(&mut out, 0xE6, "let imm = self.fetch8(com); self.a &= imm; self.set_flags(self.a == 0, false, true, false);");

    // OR d8
    // 0xF6
    set_opcode(&mut out, 0xF6, "let imm = self.fetch8(com); self.a |= imm; self.set_flags(self.a == 0, false, false, false);");

    // XOR d8
    // 0xEE
    set_opcode(
        &mut out,
        0xEE,
        "self.a ^= self.fetch8(com); self.set_flags(self.a == 0, false, false, false);",
    );

    // CALL cc, r16
    // 0xC4, 0xD4, 0xCC, 0xDC
    set_opcode(&mut out, 0xC4, &call_cc("!self.get_z()"));
    set_opcode(&mut out, 0xD4, &call_cc("!self.get_c()"));
    set_opcode(&mut out, 0xCC, &call_cc("self.get_z()"));
    set_opcode(&mut out, 0xDC, &call_cc("self.get_c()"));

    // ADD A, d8
    // 0xC6
    set_opcode(
        &mut out,
        0xC6,
        r"let val = self.fetch8(com);
        let (res, carry) = self.a.overflowing_add(val);
        self.set_flags(
            res == 0,
            false,
            (((self.a & 0xF) + (val & 0xF)) & 0x10) != 0,
            carry,
        );
        self.a = res;",
    );

    // SUB d8
    // 0xD6
    set_opcode(
        &mut out,
        0xD6,
        "let imm = self.fetch8(com); self.a = self.alu_sub(imm, false);",
    );

    // ADC A, d8
    // 0xCE
    set_opcode(
        &mut out,
        0xCE,
        r"let carry = self.get_c() as u8;
        let val = self.fetch8(com);
        let res = self.a.wrapping_add(val).wrapping_add(carry);
        self.set_flags(
            res == 0,
            false,
            (((self.a & 0xF) + (val & 0xF) + carry) & 0x10) != 0,
            (res as u16) < (self.a as u16) + (val as u16) + (carry as u16),
        );
        self.a = res;",
    );

    // SBC A, d8
    // 0xDE
    set_opcode(
        &mut out,
        0xDE,
        "let imm = self.fetch8(com); self.a = self.alu_sub(imm, self.get_c());",
    );

    // ADD HL, r16
    // 0x09, 0x19, 0x29, 0x39
    for (i, x) in ["BC", "DE", "HL", "SP"].iter().enumerate() {
        set_opcode(
            &mut out,
            (i << 4) | 0x09,
            &format!("self.add_hl_r16(com, Reg16::{x});"),
        );
    }

    // JP (HL)
    // 0xE9
    set_opcode(
        &mut out,
        0xE9,
        "let nn = Reg16::HL.read(self); self.pc = nn;",
    );

    // JP cc, a16
    // 0xC2, 0xCA, 0xD2, 0xDA
    set_opcode(&mut out, 0xC2, &jp_cc("!self.get_z()"));
    set_opcode(&mut out, 0xD2, &jp_cc("!self.get_c()"));
    set_opcode(&mut out, 0xCA, &jp_cc("self.get_z()"));
    set_opcode(&mut out, 0xDA, &jp_cc("self.get_c()"));

    // DAA
    // 0x27
    set_opcode(
        &mut out,
        0x27,
        r"let mut correction = 0;

        if self.get_h() || (!self.get_n() && (self.a & 0xF) > 9) {
            correction |= 0x6;
        }

        let carry = self.get_c() || (!self.get_n() && self.a > 0x99);
        if carry {
            correction |= 0x60;
        }

        if self.get_n() {
            self.a = self.a.wrapping_sub(correction);
        } else {
            self.a = self.a.wrapping_add(correction);
        }

        self.set_z(self.a == 0);
        self.set_h(false);
        self.set_c(carry);",
    );

    // LD SP, HL
    // 0xF9
    set_opcode(
        &mut out,
        0xF9,
        "self.run_cycle(com); Reg16::SP.write(self, Reg16::HL.read(self));",
    );

    // LD HL, SP+r8
    // 0xF8
    set_opcode(
        &mut out,
        0xF8,
        r"let imm = self.fetch8(com) as i8;
        let val = self.sp.wrapping_add_signed(imm as i16);
        let (_, carry) = (self.sp as u8).overflowing_add(imm as u8);
        self.set_flags(
            false,
            false,
            (((Reg16::SP.read(self) & 0xF) + (imm as u16 & 0xF)) & 0x10) != 0,
            carry,
        );
        Reg16::HL.write(self, val);
        self.run_cycle(com);",
    );

    // HALT
    // 0x76
    set_opcode(&mut out, 0x76, "self.halted = true;");

    // LD (a16), SP
    // 0x08
    set_opcode(&mut out, 0x08, "let nn = self.fetch16(com); self.write8(com, nn, self.sp as u8); self.write8(com, nn.wrapping_add(1), (self.sp >> 8) as u8);");

    // ADD SP, r8
    // 0xE8
    set_opcode(
        &mut out,
        0xE8,
        r"let imm = self.fetch8(com) as i8;
        let val = self.sp.wrapping_add_signed(imm as i16);
        let (_, carry) = (self.sp as u8).overflowing_add(imm as u8);
        self.set_flags(
            false,
            false,
            (((Reg16::SP.read(self) & 0xF) + (imm as u16 & 0xF)) & 0x10) != 0,
            carry,
        );
        self.sp = val;
        self.run_cycle(com);
        self.run_cycle(com);",
    );

    // RST n
    // 0xC7, 0xCF, 0xD7, 0xDF, 0xE7, 0xEF, 0xF7, 0xFF
    for i in 0..8 {
        let hi = 0xC0 | ((i / 2) << 4);
        let lo = if i % 2 == 0 { 0x07 } else { 0x0F };
        set_opcode(
            &mut out,
            hi | lo,
            &format!("self.rst_n(com, {});", (hi | lo) & !0xC7),
        );
    }

    // LD A, (C)
    // 0xF2
    set_opcode(
        &mut out,
        0xF2,
        "self.a = self.read8(com, 0xFF00 | self.c as u16)",
    );

    // CPL
    // 0x2F
    set_opcode(
        &mut out,
        0x2F,
        "self.a = !self.a; self.set_n(true); self.set_h(true);",
    );

    // SCF
    // 0x37
    set_opcode(
        &mut out,
        0x37,
        "self.set_n(false); self.set_h(false); self.set_c(true);",
    );

    // CCF
    // 0x3F
    set_opcode(
        &mut out,
        0x3F,
        "self.set_n(false); self.set_h(false); self.set_c(!self.get_c());",
    );

    // ADC A, r8
    // 0x88 to 0x8F
    for (i, x) in reg8_enum.iter().enumerate() {
        set_opcode(&mut out, 0x88 | i, &format!("self.adc_a_r8(Reg8::{x});"));
    }

    // SBC A, r8
    // 0x98 to 0x9F
    for (i, x) in reg8_enum.iter().enumerate() {
        set_opcode(&mut out, 0x98 | i, &format!("self.sbc_a_r8(Reg8::{x});"));
    }

    // AND r8
    // 0xA0 to 0xA7
    for (i, x) in reg8_enum.iter().enumerate() {
        set_opcode(&mut out, 0xA0 | i, &format!("self.and_a_r8(Reg8::{x});"));
    }

    out
}

fn jr_cc(condition: &str) -> String {
    format!("let offset = self.fetch8(com) as i8; if {condition} {{ self.run_cycle(com); self.pc = self.pc.wrapping_add_signed(offset as i16); }}")
}

fn jp_cc(condition: &str) -> String {
    format!("let nn = self.fetch16(com); if {condition} {{ self.run_cycle(com); self.pc = nn; }}")
}

fn call_cc(condition: &str) -> String {
    format!("let nn = self.fetch16(com); if {condition} {{ self.push_r16(com, Reg16::PC); self.pc = nn; }}")
}

fn ret_cc(condition: &str) -> String {
    format!("self.run_cycle(com); if {condition} {{ self.pop_r16(com, Reg16::PC); self.run_cycle(com); }}")
}

// Ensure that we aren't accidentally overwriting an opcode
fn set_opcode(out: &mut [Option<String>], idx: usize, contents: &str) {
    assert!(out[idx].is_none(), "Opcode overlap at 0x{idx:X}");
    out[idx] = Some(contents.to_string());
}

// Writes the opcode handler table to a big switch statement
fn write_opcodes(writer: &mut impl Write, opcodes: &[Option<String>]) {
    writer
        .write_all("match self.opcode {\n".as_bytes())
        .unwrap();

    let mut count = 0;
    for (i, x) in opcodes.iter().enumerate() {
        if let Some(x) = x {
            writer
                .write_fmt(format_args!("0x{i:X} => {{ {x} }}\n"))
                .unwrap();
            count += 1;
        }
    }

    if count != 256 {
        writer
            .write_all("x => panic!(\"Unhandled opcode 0x{x:X}\")\n".as_bytes())
            .unwrap();
    }

    writer.write_all("}".as_bytes()).unwrap();
}
