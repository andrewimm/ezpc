//! Tier 1 operand decoding
//!
//! Decodes operands on-the-fly during first execution (cold path)
//! This module provides methods for decoding instruction operands directly
//! from memory without caching.

use crate::cpu::Cpu;
use crate::cpu::decode::instruction::DecodedInstruction;
use crate::cpu::decode::operands::Operand;
use crate::memory::MemoryBus;

impl Cpu {
    /// Decode operands for a given opcode and create a DecodedInstruction
    ///
    /// This is called during tier 1 execution to decode operands on-the-fly.
    /// The handler is already selected from the dispatch table.
    pub fn decode_instruction_t1(
        &mut self,
        mem: &MemoryBus,
        opcode: u8,
        handler: fn(&mut Cpu, &mut MemoryBus, &DecodedInstruction),
    ) -> DecodedInstruction {
        let mut instr = DecodedInstruction::new(opcode, handler);

        // Decode operands based on opcode
        match opcode {
            // NOP - no operands
            0x90 => {
                // No operands needed
            }

            // MOV r/m, r (0x88, 0x89)
            0x88 | 0x89 => {
                let is_byte = opcode == 0x88;
                let (dst, src, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // MOV r, r/m (0x8A, 0x8B)
            0x8A | 0x8B => {
                let is_byte = opcode == 0x8A;
                let (src, dst, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // XCHG r/m, r (0x86, 0x87)
            0x86 | 0x87 => {
                let is_byte = opcode == 0x86;
                let (dst, src, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // MOV r, imm (0xB0-0xBF)
            0xB0..=0xBF => {
                let is_byte = opcode < 0xB8;
                let reg = opcode & 0x07;

                if is_byte {
                    let imm = self.fetch_u8(mem);
                    instr = instr
                        .with_dst(Operand::reg8(reg))
                        .with_src(Operand::imm8(imm))
                        .with_length(2);
                } else {
                    let imm = self.fetch_u16(mem);
                    instr = instr
                        .with_dst(Operand::reg16(reg))
                        .with_src(Operand::imm16(imm))
                        .with_length(3);
                }
            }

            // MOV r/m, imm (0xC6, 0xC7)
            0xC6 | 0xC7 => {
                let is_byte = opcode == 0xC6;
                let (dst, _, modrm_len) = self.decode_modrm_operands(mem, is_byte);

                if is_byte {
                    let imm = self.fetch_u8(mem);
                    instr = instr
                        .with_dst(dst)
                        .with_src(Operand::imm8(imm))
                        .with_length(1 + modrm_len + 1);
                } else {
                    let imm = self.fetch_u16(mem);
                    instr = instr
                        .with_dst(dst)
                        .with_src(Operand::imm16(imm))
                        .with_length(1 + modrm_len + 2);
                }
            }

            // XCHG AX, r16 (0x91-0x97)
            0x91..=0x97 => {
                let reg = opcode & 0x07;
                instr = instr.with_dst(Operand::reg16(reg)).with_length(1);
            }

            // PUSH r16 (0x50-0x57)
            0x50..=0x57 => {
                let reg = opcode & 0x07;
                instr = instr.with_src(Operand::reg16(reg)).with_length(1);
            }

            // POP r16 (0x58-0x5F)
            0x58..=0x5F => {
                let reg = opcode & 0x07;
                instr = instr.with_dst(Operand::reg16(reg)).with_length(1);
            }

            // PUSH ES (0x06), PUSH CS (0x0E), PUSH SS (0x16), PUSH DS (0x1E)
            0x06 | 0x0E | 0x16 | 0x1E => {
                let seg = (opcode >> 3) & 0x03; // Extract segment index
                instr = instr.with_src(Operand::seg(seg)).with_length(1);
            }

            // POP ES (0x07), POP SS (0x17), POP DS (0x1F)
            0x07 | 0x17 | 0x1F => {
                let seg = (opcode >> 3) & 0x03; // Extract segment index
                instr = instr.with_dst(Operand::seg(seg)).with_length(1);
            }

            // INC r16 (0x40-0x47)
            0x40..=0x47 => {
                let reg = opcode & 0x07;
                instr = instr.with_dst(Operand::reg16(reg)).with_length(1);
            }

            // DEC r16 (0x48-0x4F)
            0x48..=0x4F => {
                let reg = opcode & 0x07;
                instr = instr.with_dst(Operand::reg16(reg)).with_length(1);
            }

            // ADD r/m, r (0x00, 0x01)
            0x00 | 0x01 => {
                let is_byte = opcode == 0x00;
                let (dst, src, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // ADD r, r/m (0x02, 0x03)
            0x02 | 0x03 => {
                let is_byte = opcode == 0x02;
                let (src, dst, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // OR r/m, r (0x08, 0x09)
            0x08 | 0x09 => {
                let is_byte = opcode == 0x08;
                let (dst, src, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // OR r, r/m (0x0A, 0x0B)
            0x0A | 0x0B => {
                let is_byte = opcode == 0x0A;
                let (src, dst, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // OR AL/AX, imm (0x0C, 0x0D)
            0x0C => {
                let imm = self.fetch_u8(mem);
                instr = instr
                    .with_dst(Operand::reg8(0)) // AL
                    .with_src(Operand::imm8(imm))
                    .with_length(2);
            }
            0x0D => {
                let imm = self.fetch_u16(mem);
                instr = instr
                    .with_dst(Operand::reg16(0)) // AX
                    .with_src(Operand::imm16(imm))
                    .with_length(3);
            }

            // ADC r/m, r (0x10, 0x11)
            0x10 | 0x11 => {
                let is_byte = opcode == 0x10;
                let (dst, src, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // ADC r, r/m (0x12, 0x13)
            0x12 | 0x13 => {
                let is_byte = opcode == 0x12;
                let (src, dst, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // ADC AL/AX, imm (0x14, 0x15)
            0x14 => {
                let imm = self.fetch_u8(mem);
                instr = instr
                    .with_dst(Operand::reg8(0)) // AL
                    .with_src(Operand::imm8(imm))
                    .with_length(2);
            }
            0x15 => {
                let imm = self.fetch_u16(mem);
                instr = instr
                    .with_dst(Operand::reg16(0)) // AX
                    .with_src(Operand::imm16(imm))
                    .with_length(3);
            }

            // ADD AL/AX, imm (0x04, 0x05)
            0x04 => {
                let imm = self.fetch_u8(mem);
                instr = instr
                    .with_dst(Operand::reg8(0)) // AL
                    .with_src(Operand::imm8(imm))
                    .with_length(2);
            }
            0x05 => {
                let imm = self.fetch_u16(mem);
                instr = instr
                    .with_dst(Operand::reg16(0)) // AX
                    .with_src(Operand::imm16(imm))
                    .with_length(3);
            }

            // AND r/m, r (0x20, 0x21)
            0x20 | 0x21 => {
                let is_byte = opcode == 0x20;
                let (dst, src, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // AND r, r/m (0x22, 0x23)
            0x22 | 0x23 => {
                let is_byte = opcode == 0x22;
                let (src, dst, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // AND AL/AX, imm (0x24, 0x25)
            0x24 => {
                let imm = self.fetch_u8(mem);
                instr = instr
                    .with_dst(Operand::reg8(0)) // AL
                    .with_src(Operand::imm8(imm))
                    .with_length(2);
            }
            0x25 => {
                let imm = self.fetch_u16(mem);
                instr = instr
                    .with_dst(Operand::reg16(0)) // AX
                    .with_src(Operand::imm16(imm))
                    .with_length(3);
            }

            // SUB r/m, r (0x28, 0x29)
            0x28 | 0x29 => {
                let is_byte = opcode == 0x28;
                let (dst, src, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // SUB r, r/m (0x2A, 0x2B)
            0x2A | 0x2B => {
                let is_byte = opcode == 0x2A;
                let (src, dst, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // SUB AL/AX, imm (0x2C, 0x2D)
            0x2C => {
                let imm = self.fetch_u8(mem);
                instr = instr
                    .with_dst(Operand::reg8(0)) // AL
                    .with_src(Operand::imm8(imm))
                    .with_length(2);
            }
            0x2D => {
                let imm = self.fetch_u16(mem);
                instr = instr
                    .with_dst(Operand::reg16(0)) // AX
                    .with_src(Operand::imm16(imm))
                    .with_length(3);
            }

            // XOR r/m, r (0x30, 0x31)
            0x30 | 0x31 => {
                let is_byte = opcode == 0x30;
                let (dst, src, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // XOR r, r/m (0x32, 0x33)
            0x32 | 0x33 => {
                let is_byte = opcode == 0x32;
                let (src, dst, len) = self.decode_modrm_operands(mem, is_byte);
                instr = instr.with_dst(dst).with_src(src).with_length(1 + len);
            }

            // XOR AL/AX, imm (0x34, 0x35)
            0x34 => {
                let imm = self.fetch_u8(mem);
                instr = instr
                    .with_dst(Operand::reg8(0)) // AL
                    .with_src(Operand::imm8(imm))
                    .with_length(2);
            }
            0x35 => {
                let imm = self.fetch_u16(mem);
                instr = instr
                    .with_dst(Operand::reg16(0)) // AX
                    .with_src(Operand::imm16(imm))
                    .with_length(3);
            }

            // JMP short (0xEB)
            0xEB => {
                let rel8 = self.fetch_u8(mem) as i8 as i16 as u16;
                instr = instr.with_src(Operand::imm16(rel8)).with_length(2);
            }

            // JMP near (0xE9)
            0xE9 => {
                let rel16 = self.fetch_u16(mem);
                instr = instr.with_src(Operand::imm16(rel16)).with_length(3);
            }

            // Conditional jumps (0x70-0x7F)
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x76 | 0x77 | 0x78 | 0x79 | 0x7A | 0x7B
            | 0x7C | 0x7D | 0x7E | 0x7F => {
                let rel8 = self.fetch_u8(mem) as i8 as i16 as u16;
                instr = instr.with_src(Operand::imm16(rel8)).with_length(2);
            }

            // CALL near (0xE8)
            0xE8 => {
                let rel16 = self.fetch_u16(mem);
                instr = instr.with_src(Operand::imm16(rel16)).with_length(3);
            }

            // CALL far (0x9A)
            0x9A => {
                let offset = self.fetch_u16(mem);
                let segment = self.fetch_u16(mem);
                instr = instr
                    .with_src(Operand::imm16(offset))
                    .with_dst(Operand::imm16(segment))
                    .with_length(5);
            }

            // RET near (0xC3) - no operands
            0xC3 => {
                // No operands needed
            }

            // RET near imm16 (0xC2)
            0xC2 => {
                let imm = self.fetch_u16(mem);
                instr = instr.with_src(Operand::imm16(imm)).with_length(3);
            }

            // RETF (0xCB) - no operands
            0xCB => {
                // No operands needed
            }

            // RETF imm16 (0xCA)
            0xCA => {
                let imm = self.fetch_u16(mem);
                instr = instr.with_src(Operand::imm16(imm)).with_length(3);
            }

            // Group 0x80/0x82: Arithmetic r/m8, imm8
            0x80 | 0x82 => {
                let modrm = self.fetch_u8(mem);
                let reg = (modrm >> 3) & 0x07;
                let (rm_operand, extra_len) = self.decode_rm_from_modrm_byte(mem, modrm, true);
                let imm = self.fetch_u8(mem);

                // Store reg field in high byte of dst.value for group handler to use
                let mut dst_with_reg = rm_operand;
                dst_with_reg.value = (dst_with_reg.value & 0xFF) | ((reg as u16) << 8);

                instr = instr
                    .with_dst(dst_with_reg)
                    .with_src(Operand::imm8(imm))
                    .with_length(1 + 1 + extra_len + 1);
            }

            // Group 0x81: Arithmetic r/m16, imm16
            0x81 => {
                let modrm = self.fetch_u8(mem);
                let reg = (modrm >> 3) & 0x07;
                let (rm_operand, extra_len) = self.decode_rm_from_modrm_byte(mem, modrm, false);
                let imm = self.fetch_u16(mem);

                // Store reg field in high byte of dst.value for group handler to use
                let mut dst_with_reg = rm_operand;
                dst_with_reg.value = (dst_with_reg.value & 0xFF) | ((reg as u16) << 8);

                instr = instr
                    .with_dst(dst_with_reg)
                    .with_src(Operand::imm16(imm))
                    .with_length(1 + 1 + extra_len + 2);
            }

            // Group 0x83: Arithmetic r/m16, imm8 (sign-extended)
            0x83 => {
                let modrm = self.fetch_u8(mem);
                let reg = (modrm >> 3) & 0x07;
                let (rm_operand, extra_len) = self.decode_rm_from_modrm_byte(mem, modrm, false);
                let imm8 = self.fetch_u8(mem);
                // Sign-extend imm8 to 16 bits
                let imm16 = (imm8 as i8) as i16 as u16;

                // Store reg field in high byte of dst.value for group handler to use
                let mut dst_with_reg = rm_operand;
                dst_with_reg.value = (dst_with_reg.value & 0xFF) | ((reg as u16) << 8);

                instr = instr
                    .with_dst(dst_with_reg)
                    .with_src(Operand::imm16(imm16))
                    .with_length(1 + 1 + extra_len + 1);
            }

            // Group 0xFF: INC/DEC/CALL/JMP/PUSH r/m16
            0xFF => {
                let modrm = self.fetch_u8(mem);
                let reg = (modrm >> 3) & 0x07;
                let (rm_operand, extra_len) = self.decode_rm_from_modrm_byte(mem, modrm, false);

                // Store reg field in high byte of dst.value for group_ff to use
                let mut dst_with_reg = rm_operand;
                dst_with_reg.value = (dst_with_reg.value & 0xFF) | ((reg as u16) << 8);

                instr = instr.with_dst(dst_with_reg).with_length(1 + 1 + extra_len);
            }

            // Default case for unimplemented/invalid opcodes
            _ => {
                // No operands, length is just 1
            }
        }

        instr
    }

    /// Helper: Decode ModR/M byte and return (dst, src, length)
    ///
    /// Returns the two operands decoded from ModR/M and the total length
    /// of the ModR/M byte + displacement
    fn decode_modrm_operands(&mut self, mem: &MemoryBus, is_byte: bool) -> (Operand, Operand, u8) {
        let modrm = self.fetch_u8(mem);
        let reg = (modrm >> 3) & 0x07;

        let reg_operand = if is_byte {
            Operand::reg8(reg)
        } else {
            Operand::reg16(reg)
        };

        let (rm_operand, extra_len) = self.decode_rm_from_modrm_byte(mem, modrm, is_byte);

        // Total length is 1 (ModR/M byte) + displacement length
        (rm_operand, reg_operand, 1 + extra_len)
    }

    /// Helper: Decode the r/m operand from ModR/M byte
    ///
    /// Returns (operand, displacement_length)
    fn decode_rm_from_modrm_byte(
        &mut self,
        mem: &MemoryBus,
        modrm: u8,
        is_byte: bool,
    ) -> (Operand, u8) {
        let mod_bits = (modrm >> 6) & 0x03;
        let rm = modrm & 0x07;

        match mod_bits {
            0b11 => {
                // Register mode
                let op = if is_byte {
                    Operand::reg8(rm)
                } else {
                    Operand::reg16(rm)
                };
                (op, 0)
            }
            0b00 => {
                // Memory mode, no displacement (except rm=110)
                if rm == 0b110 {
                    // Direct addressing [disp16]
                    let disp = self.fetch_u16(mem);
                    // Direct addressing: value field holds the direct address
                    let op = if is_byte {
                        Operand::new(crate::cpu::decode::operands::OperandType::Mem8, disp)
                    } else {
                        Operand::new(crate::cpu::decode::operands::OperandType::Mem16, disp)
                    };
                    (op, 2)
                } else {
                    // Indirect addressing [reg]
                    let op = if is_byte {
                        Operand::mem8(rm)
                    } else {
                        Operand::mem16(rm)
                    };
                    (op, 0)
                }
            }
            0b01 => {
                // Memory mode, 8-bit displacement
                let disp = self.fetch_u8(mem) as i8 as i16;
                let op = if is_byte {
                    Operand::mem8_disp(rm, disp)
                } else {
                    Operand::mem16_disp(rm, disp)
                };
                (op, 1)
            }
            0b10 => {
                // Memory mode, 16-bit displacement
                let disp = self.fetch_u16(mem) as i16;
                let op = if is_byte {
                    Operand::mem8_disp(rm, disp)
                } else {
                    Operand::mem16_disp(rm, disp)
                };
                (op, 2)
            }
            _ => unreachable!(),
        }
    }
}
