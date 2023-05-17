match self.opcode {
0x0 => {  }
0x1 => { self.ld_r16_d16(com, Reg16::BC); }
0x2 => { let addr = Reg16::BC.read(self); self.write8(com, addr, self.a); }
0x3 => { self.inc_r16(com, Reg16::BC); }
0x4 => { self.inc_r8(Reg8::B); }
0x5 => { self.dec_r8(Reg8::B); }
0x6 => { let imm = self.fetch8(com); self.ld_r8_d8(imm, Reg8::B); }
0x7 => { self.a = self.a.rotate_left(1); self.set_flags(false, false, false, (self.a & 1) != 0); }
0x8 => { let nn = self.fetch16(com); self.write8(com, nn, self.sp as u8); self.write8(com, nn.wrapping_add(1), (self.sp >> 8) as u8); }
0x9 => { self.add_hl_r16(com, Reg16::BC); }
0xA => { let addr = Reg16::BC.read(self); self.a = self.read8(com, addr); }
0xB => { self.dec_r16(com, Reg16::BC); }
0xC => { self.inc_r8(Reg8::C); }
0xD => { self.dec_r8(Reg8::C); }
0xE => { let imm = self.fetch8(com); self.ld_r8_d8(imm, Reg8::C); }
0xF => { self.a = self.a.rotate_right(1); self.set_flags(false, false, false, (self.a & 0x80) != 0); }
0x11 => { self.ld_r16_d16(com, Reg16::DE); }
0x12 => { let addr = Reg16::DE.read(self); self.write8(com, addr, self.a); }
0x13 => { self.inc_r16(com, Reg16::DE); }
0x14 => { self.inc_r8(Reg8::D); }
0x15 => { self.dec_r8(Reg8::D); }
0x16 => { let imm = self.fetch8(com); self.ld_r8_d8(imm, Reg8::D); }
0x17 => { let carry = (self.a & 0x80) != 0; self.a = (self.a << 1) | ((self.get_c() as u8) & 1); self.set_flags(false, false, false, carry); }
0x18 => { let offset = self.fetch8(com) as i8; self.run_cycle(com); self.pc = self.pc.wrapping_add_signed(offset as i16); }
0x19 => { self.add_hl_r16(com, Reg16::DE); }
0x1A => { let addr = Reg16::DE.read(self); self.a = self.read8(com, addr); }
0x1B => { self.dec_r16(com, Reg16::DE); }
0x1C => { self.inc_r8(Reg8::E); }
0x1D => { self.dec_r8(Reg8::E); }
0x1E => { let imm = self.fetch8(com); self.ld_r8_d8(imm, Reg8::E); }
0x1F => { let carry = (self.a & 1) != 0; self.a = (self.a >> 1) | ((self.get_c() as u8) << 7); self.set_flags(false, false, false, carry); }
0x20 => { let offset = self.fetch8(com) as i8; if !self.get_z() { self.run_cycle(com); self.pc = self.pc.wrapping_add_signed(offset as i16); } }
0x21 => { self.ld_r16_d16(com, Reg16::HL); }
0x22 => { let addr = Reg16::HL.read(self); self.write8(com, addr, self.a); Reg16::HL.write(self, addr.wrapping_add(1)); }
0x23 => { self.inc_r16(com, Reg16::HL); }
0x24 => { self.inc_r8(Reg8::H); }
0x25 => { self.dec_r8(Reg8::H); }
0x26 => { let imm = self.fetch8(com); self.ld_r8_d8(imm, Reg8::H); }
0x27 => { let mut correction = 0;

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
        self.set_c(carry); }
0x28 => { let offset = self.fetch8(com) as i8; if self.get_z() { self.run_cycle(com); self.pc = self.pc.wrapping_add_signed(offset as i16); } }
0x29 => { self.add_hl_r16(com, Reg16::HL); }
0x2A => { let addr = Reg16::HL.read(self); self.a = self.read8(com, addr); Reg16::HL.write(self, addr.wrapping_add(1)); }
0x2B => { self.dec_r16(com, Reg16::HL); }
0x2C => { self.inc_r8(Reg8::L); }
0x2D => { self.dec_r8(Reg8::L); }
0x2E => { let imm = self.fetch8(com); self.ld_r8_d8(imm, Reg8::L); }
0x2F => { self.a = !self.a; self.set_n(true); self.set_h(true); }
0x30 => { let offset = self.fetch8(com) as i8; if !self.get_c() { self.run_cycle(com); self.pc = self.pc.wrapping_add_signed(offset as i16); } }
0x31 => { self.ld_r16_d16(com, Reg16::SP); }
0x32 => { let addr = Reg16::HL.read(self); self.write8(com, addr, self.a); Reg16::HL.write(self, addr.wrapping_sub(1)); }
0x33 => { self.inc_r16(com, Reg16::SP); }
0x34 => { self.inc_r8(Reg8::HLPtr(com)); }
0x35 => { self.dec_r8(Reg8::HLPtr(com)); }
0x36 => { let imm = self.fetch8(com); self.ld_r8_d8(imm, Reg8::HLPtr(com)); }
0x37 => { self.set_n(false); self.set_h(false); self.set_c(true); }
0x38 => { let offset = self.fetch8(com) as i8; if self.get_c() { self.run_cycle(com); self.pc = self.pc.wrapping_add_signed(offset as i16); } }
0x39 => { self.add_hl_r16(com, Reg16::SP); }
0x3A => { let addr = Reg16::HL.read(self); self.a = self.read8(com, addr); Reg16::HL.write(self, addr.wrapping_sub(1)); }
0x3B => { self.dec_r16(com, Reg16::SP); }
0x3C => { self.inc_r8(Reg8::A); }
0x3D => { self.dec_r8(Reg8::A); }
0x3E => { let imm = self.fetch8(com); self.ld_r8_d8(imm, Reg8::A); }
0x3F => { self.set_n(false); self.set_h(false); self.set_c(!self.get_c()); }
0x40 => { let val = Reg8::B.read(self); Reg8::B.write(self, val); }
0x41 => { let val = Reg8::C.read(self); Reg8::B.write(self, val); }
0x42 => { let val = Reg8::D.read(self); Reg8::B.write(self, val); }
0x43 => { let val = Reg8::E.read(self); Reg8::B.write(self, val); }
0x44 => { let val = Reg8::H.read(self); Reg8::B.write(self, val); }
0x45 => { let val = Reg8::L.read(self); Reg8::B.write(self, val); }
0x46 => { let val = Reg8::HLPtr(com).read(self); Reg8::B.write(self, val); }
0x47 => { let val = Reg8::A.read(self); Reg8::B.write(self, val); }
0x48 => { let val = Reg8::B.read(self); Reg8::C.write(self, val); }
0x49 => { let val = Reg8::C.read(self); Reg8::C.write(self, val); }
0x4A => { let val = Reg8::D.read(self); Reg8::C.write(self, val); }
0x4B => { let val = Reg8::E.read(self); Reg8::C.write(self, val); }
0x4C => { let val = Reg8::H.read(self); Reg8::C.write(self, val); }
0x4D => { let val = Reg8::L.read(self); Reg8::C.write(self, val); }
0x4E => { let val = Reg8::HLPtr(com).read(self); Reg8::C.write(self, val); }
0x4F => { let val = Reg8::A.read(self); Reg8::C.write(self, val); }
0x50 => { let val = Reg8::B.read(self); Reg8::D.write(self, val); }
0x51 => { let val = Reg8::C.read(self); Reg8::D.write(self, val); }
0x52 => { let val = Reg8::D.read(self); Reg8::D.write(self, val); }
0x53 => { let val = Reg8::E.read(self); Reg8::D.write(self, val); }
0x54 => { let val = Reg8::H.read(self); Reg8::D.write(self, val); }
0x55 => { let val = Reg8::L.read(self); Reg8::D.write(self, val); }
0x56 => { let val = Reg8::HLPtr(com).read(self); Reg8::D.write(self, val); }
0x57 => { let val = Reg8::A.read(self); Reg8::D.write(self, val); }
0x58 => { let val = Reg8::B.read(self); Reg8::E.write(self, val); }
0x59 => { let val = Reg8::C.read(self); Reg8::E.write(self, val); }
0x5A => { let val = Reg8::D.read(self); Reg8::E.write(self, val); }
0x5B => { let val = Reg8::E.read(self); Reg8::E.write(self, val); }
0x5C => { let val = Reg8::H.read(self); Reg8::E.write(self, val); }
0x5D => { let val = Reg8::L.read(self); Reg8::E.write(self, val); }
0x5E => { let val = Reg8::HLPtr(com).read(self); Reg8::E.write(self, val); }
0x5F => { let val = Reg8::A.read(self); Reg8::E.write(self, val); }
0x60 => { let val = Reg8::B.read(self); Reg8::H.write(self, val); }
0x61 => { let val = Reg8::C.read(self); Reg8::H.write(self, val); }
0x62 => { let val = Reg8::D.read(self); Reg8::H.write(self, val); }
0x63 => { let val = Reg8::E.read(self); Reg8::H.write(self, val); }
0x64 => { let val = Reg8::H.read(self); Reg8::H.write(self, val); }
0x65 => { let val = Reg8::L.read(self); Reg8::H.write(self, val); }
0x66 => { let val = Reg8::HLPtr(com).read(self); Reg8::H.write(self, val); }
0x67 => { let val = Reg8::A.read(self); Reg8::H.write(self, val); }
0x68 => { let val = Reg8::B.read(self); Reg8::L.write(self, val); }
0x69 => { let val = Reg8::C.read(self); Reg8::L.write(self, val); }
0x6A => { let val = Reg8::D.read(self); Reg8::L.write(self, val); }
0x6B => { let val = Reg8::E.read(self); Reg8::L.write(self, val); }
0x6C => { let val = Reg8::H.read(self); Reg8::L.write(self, val); }
0x6D => { let val = Reg8::L.read(self); Reg8::L.write(self, val); }
0x6E => { let val = Reg8::HLPtr(com).read(self); Reg8::L.write(self, val); }
0x6F => { let val = Reg8::A.read(self); Reg8::L.write(self, val); }
0x70 => { let val = Reg8::B.read(self); Reg8::HLPtr(com).write(self, val); }
0x71 => { let val = Reg8::C.read(self); Reg8::HLPtr(com).write(self, val); }
0x72 => { let val = Reg8::D.read(self); Reg8::HLPtr(com).write(self, val); }
0x73 => { let val = Reg8::E.read(self); Reg8::HLPtr(com).write(self, val); }
0x74 => { let val = Reg8::H.read(self); Reg8::HLPtr(com).write(self, val); }
0x75 => { let val = Reg8::L.read(self); Reg8::HLPtr(com).write(self, val); }
0x76 => { self.halted = true; }
0x77 => { let val = Reg8::A.read(self); Reg8::HLPtr(com).write(self, val); }
0x78 => { let val = Reg8::B.read(self); Reg8::A.write(self, val); }
0x79 => { let val = Reg8::C.read(self); Reg8::A.write(self, val); }
0x7A => { let val = Reg8::D.read(self); Reg8::A.write(self, val); }
0x7B => { let val = Reg8::E.read(self); Reg8::A.write(self, val); }
0x7C => { let val = Reg8::H.read(self); Reg8::A.write(self, val); }
0x7D => { let val = Reg8::L.read(self); Reg8::A.write(self, val); }
0x7E => { let val = Reg8::HLPtr(com).read(self); Reg8::A.write(self, val); }
0x7F => { let val = Reg8::A.read(self); Reg8::A.write(self, val); }
0x80 => { self.add_a_r8(Reg8::B); }
0x81 => { self.add_a_r8(Reg8::C); }
0x82 => { self.add_a_r8(Reg8::D); }
0x83 => { self.add_a_r8(Reg8::E); }
0x84 => { self.add_a_r8(Reg8::H); }
0x85 => { self.add_a_r8(Reg8::L); }
0x86 => { self.add_a_r8(Reg8::HLPtr(com)); }
0x87 => { self.add_a_r8(Reg8::A); }
0x88 => { self.adc_a_r8(Reg8::B); }
0x89 => { self.adc_a_r8(Reg8::C); }
0x8A => { self.adc_a_r8(Reg8::D); }
0x8B => { self.adc_a_r8(Reg8::E); }
0x8C => { self.adc_a_r8(Reg8::H); }
0x8D => { self.adc_a_r8(Reg8::L); }
0x8E => { self.adc_a_r8(Reg8::HLPtr(com)); }
0x8F => { self.adc_a_r8(Reg8::A); }
0x90 => { self.sub_r8(Reg8::B); }
0x91 => { self.sub_r8(Reg8::C); }
0x92 => { self.sub_r8(Reg8::D); }
0x93 => { self.sub_r8(Reg8::E); }
0x94 => { self.sub_r8(Reg8::H); }
0x95 => { self.sub_r8(Reg8::L); }
0x96 => { self.sub_r8(Reg8::HLPtr(com)); }
0x97 => { self.sub_r8(Reg8::A); }
0x98 => { self.sbc_a_r8(Reg8::B); }
0x99 => { self.sbc_a_r8(Reg8::C); }
0x9A => { self.sbc_a_r8(Reg8::D); }
0x9B => { self.sbc_a_r8(Reg8::E); }
0x9C => { self.sbc_a_r8(Reg8::H); }
0x9D => { self.sbc_a_r8(Reg8::L); }
0x9E => { self.sbc_a_r8(Reg8::HLPtr(com)); }
0x9F => { self.sbc_a_r8(Reg8::A); }
0xA0 => { self.and_a_r8(Reg8::B); }
0xA1 => { self.and_a_r8(Reg8::C); }
0xA2 => { self.and_a_r8(Reg8::D); }
0xA3 => { self.and_a_r8(Reg8::E); }
0xA4 => { self.and_a_r8(Reg8::H); }
0xA5 => { self.and_a_r8(Reg8::L); }
0xA6 => { self.and_a_r8(Reg8::HLPtr(com)); }
0xA7 => { self.and_a_r8(Reg8::A); }
0xA8 => { self.xor_r8(Reg8::B) }
0xA9 => { self.xor_r8(Reg8::C) }
0xAA => { self.xor_r8(Reg8::D) }
0xAB => { self.xor_r8(Reg8::E) }
0xAC => { self.xor_r8(Reg8::H) }
0xAD => { self.xor_r8(Reg8::L) }
0xAE => { self.xor_r8(Reg8::HLPtr(com)) }
0xAF => { self.xor_r8(Reg8::A) }
0xB0 => { self.or_r8(Reg8::B); }
0xB1 => { self.or_r8(Reg8::C); }
0xB2 => { self.or_r8(Reg8::D); }
0xB3 => { self.or_r8(Reg8::E); }
0xB4 => { self.or_r8(Reg8::H); }
0xB5 => { self.or_r8(Reg8::L); }
0xB6 => { self.or_r8(Reg8::HLPtr(com)); }
0xB7 => { self.or_r8(Reg8::A); }
0xB8 => { self.cp_r8(Reg8::B); }
0xB9 => { self.cp_r8(Reg8::C); }
0xBA => { self.cp_r8(Reg8::D); }
0xBB => { self.cp_r8(Reg8::E); }
0xBC => { self.cp_r8(Reg8::H); }
0xBD => { self.cp_r8(Reg8::L); }
0xBE => { self.cp_r8(Reg8::HLPtr(com)); }
0xBF => { self.cp_r8(Reg8::A); }
0xC0 => { self.run_cycle(com); if !self.get_z() { self.pop_r16(com, Reg16::PC); self.run_cycle(com); } }
0xC1 => { self.pop_r16(com, Reg16::BC); }
0xC2 => { let nn = self.fetch16(com); if !self.get_z() { self.run_cycle(com); self.pc = nn; } }
0xC3 => { let nn = self.fetch16(com); self.run_cycle(com); self.pc = nn; }
0xC4 => { let nn = self.fetch16(com); if !self.get_z() { self.push_r16(com, Reg16::PC); self.pc = nn; } }
0xC5 => { self.push_r16(com, Reg16::BC); }
0xC6 => { let val = self.fetch8(com);
        let (res, carry) = self.a.overflowing_add(val);
        self.set_flags(
            res == 0,
            false,
            (((self.a & 0xF) + (val & 0xF)) & 0x10) != 0,
            carry,
        );
        self.a = res; }
0xC7 => { self.rst_n(com, 0); }
0xC8 => { self.run_cycle(com); if self.get_z() { self.pop_r16(com, Reg16::PC); self.run_cycle(com); } }
0xC9 => { self.ret(com); }
0xCA => { let nn = self.fetch16(com); if self.get_z() { self.run_cycle(com); self.pc = nn; } }
0xCB => { self.handle_cb(com); }
0xCC => { let nn = self.fetch16(com); if self.get_z() { self.push_r16(com, Reg16::PC); self.pc = nn; } }
0xCD => { let nn = self.fetch16(com); self.push_r16(com, Reg16::PC); self.pc = nn; }
0xCE => { let carry = self.get_c() as u8;
        let val = self.fetch8(com);
        let res = self.a.wrapping_add(val).wrapping_add(carry);
        self.set_flags(
            res == 0,
            false,
            (((self.a & 0xF) + (val & 0xF) + carry) & 0x10) != 0,
            (res as u16) < (self.a as u16) + (val as u16) + (carry as u16),
        );
        self.a = res; }
0xCF => { self.rst_n(com, 8); }
0xD0 => { self.run_cycle(com); if !self.get_c() { self.pop_r16(com, Reg16::PC); self.run_cycle(com); } }
0xD1 => { self.pop_r16(com, Reg16::DE); }
0xD2 => { let nn = self.fetch16(com); if !self.get_c() { self.run_cycle(com); self.pc = nn; } }
0xD4 => { let nn = self.fetch16(com); if !self.get_c() { self.push_r16(com, Reg16::PC); self.pc = nn; } }
0xD5 => { self.push_r16(com, Reg16::DE); }
0xD6 => { let imm = self.fetch8(com); self.a = self.alu_sub(imm, false); }
0xD7 => { self.rst_n(com, 16); }
0xD8 => { self.run_cycle(com); if self.get_c() { self.pop_r16(com, Reg16::PC); self.run_cycle(com); } }
0xD9 => { self.ime = true; self.pop_r16(com, Reg16::PC); self.run_cycle(com); }
0xDA => { let nn = self.fetch16(com); if self.get_c() { self.run_cycle(com); self.pc = nn; } }
0xDC => { let nn = self.fetch16(com); if self.get_c() { self.push_r16(com, Reg16::PC); self.pc = nn; } }
0xDE => { let imm = self.fetch8(com); self.a = self.alu_sub(imm, self.get_c()); }
0xDF => { self.rst_n(com, 24); }
0xE0 => { let n = self.fetch8(com); self.write8(com, 0xFF00 | n as u16, self.a); }
0xE1 => { self.pop_r16(com, Reg16::HL); }
0xE2 => { self.write8(com, 0xFF00 | self.c as u16, self.a) }
0xE5 => { self.push_r16(com, Reg16::HL); }
0xE6 => { let imm = self.fetch8(com); self.a &= imm; self.set_flags(self.a == 0, false, true, false); }
0xE7 => { self.rst_n(com, 32); }
0xE8 => { let imm = self.fetch8(com) as i8;
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
        self.run_cycle(com); }
0xE9 => { let nn = Reg16::HL.read(self); self.pc = nn; }
0xEA => { let addr = self.fetch16(com); self.write8(com, addr, self.a); }
0xEE => { self.a ^= self.fetch8(com); self.set_flags(self.a == 0, false, false, false); }
0xEF => { self.rst_n(com, 40); }
0xF0 => { let n = self.fetch8(com); self.a = self.read8(com, 0xFF00 | n as u16); }
0xF1 => { self.pop_r16(com, Reg16::AF); }
0xF2 => { self.a = self.read8(com, 0xFF00 | self.c as u16) }
0xF3 => { self.ime_queued = false; self.ime = false; }
0xF5 => { self.push_r16(com, Reg16::AF); }
0xF6 => { let imm = self.fetch8(com); self.a |= imm; self.set_flags(self.a == 0, false, false, false); }
0xF7 => { self.rst_n(com, 48); }
0xF8 => { let imm = self.fetch8(com) as i8;
        let val = self.sp.wrapping_add_signed(imm as i16);
        let (_, carry) = (self.sp as u8).overflowing_add(imm as u8);
        self.set_flags(
            false,
            false,
            (((Reg16::SP.read(self) & 0xF) + (imm as u16 & 0xF)) & 0x10) != 0,
            carry,
        );
        Reg16::HL.write(self, val);
        self.run_cycle(com); }
0xF9 => { self.run_cycle(com); Reg16::SP.write(self, Reg16::HL.read(self)); }
0xFA => { let addr = self.fetch16(com); self.a = self.read8(com, addr); }
0xFB => { self.ime_queued = true; }
0xFE => { let n = self.fetch8(com); self.alu_sub(n, false); }
0xFF => { self.rst_n(com, 56); }
x => panic!("Unhandled opcode 0x{x:X}")
}