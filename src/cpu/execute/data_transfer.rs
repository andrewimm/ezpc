//! Data transfer instruction handlers (MOV, XCHG, etc.)

use crate::cpu::decode::DecodedInstruction;
use crate::cpu::Cpu;
use crate::memory::MemoryBus;

/// MOV r/m, r - Move register to register/memory
/// Handles both byte (0x88) and word (0x89) variants
///
/// The destination (r/m) is in the dst operand, source (r) is in the src operand
pub fn mov_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let src_value = cpu.read_operand(mem, &instr.src);
    cpu.write_operand(mem, &instr.dst, src_value);
}

/// MOV r, r/m - Move register/memory to register
/// Handles both byte (0x8A) and word (0x8B) variants
///
/// The destination (r) is in the dst operand, source (r/m) is in the src operand
pub fn mov_r_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let src_value = cpu.read_operand(mem, &instr.src);
    cpu.write_operand(mem, &instr.dst, src_value);
}

/// MOV r, imm - Move immediate to register
/// Handles both byte (0xB0-0xB7) and word (0xB8-0xBF) variants
///
/// The register is encoded in the low 3 bits of the opcode
/// The immediate value is in the src operand
pub fn mov_r_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let imm_value = cpu.read_operand(mem, &instr.src);
    cpu.write_operand(mem, &instr.dst, imm_value);
}

/// MOV r/m, imm - Move immediate to register/memory
/// Handles both byte (0xC6) and word (0xC7) variants
///
/// The destination (r/m) is in the dst operand, immediate is in the src operand
pub fn mov_rm_imm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let imm_value = cpu.read_operand(mem, &instr.src);
    cpu.write_operand(mem, &instr.dst, imm_value);
}

/// XCHG r/m, r - Exchange register with register/memory
/// Handles both byte (0x86) and word (0x87) variants
///
/// Swaps the values of the two operands
pub fn xchg_rm_r(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let dst_value = cpu.read_operand(mem, &instr.dst);
    let src_value = cpu.read_operand(mem, &instr.src);
    cpu.write_operand(mem, &instr.dst, src_value);
    cpu.write_operand(mem, &instr.src, dst_value);
}

/// XCHG AX, r16 - Exchange AX with a 16-bit register
/// Handles opcodes 0x91-0x97 (0x90 is NOP)
///
/// The register is encoded in the low 3 bits of the opcode
pub fn xchg_ax_r16(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let ax_value = cpu.read_reg16(0); // AX
    let r_value = cpu.read_operand(mem, &instr.dst);
    cpu.write_reg16(0, r_value);
    cpu.write_operand(mem, &instr.dst, ax_value);
}

/// LEA r16, m - Load Effective Address
/// Opcode: 0x8D
///
/// Calculates the effective address of the memory operand and loads it into
/// the destination register. Unlike MOV, this does NOT access memory - it just
/// calculates the offset portion of the address.
///
/// This instruction is commonly used for pointer arithmetic and address calculations.
/// No flags are affected.
pub fn lea(cpu: &mut Cpu, _mem: &mut MemoryBus, instr: &DecodedInstruction) {
    use crate::cpu::decode::OperandType;

    // LEA requires a memory operand as source
    // We need to calculate the effective address (offset) without accessing memory
    match instr.src.op_type {
        OperandType::Mem8 | OperandType::Mem16 => {
            // Check if this is direct addressing or indirect addressing
            // Direct addressing: value == 0xFF (sentinel), address in disp field
            // Indirect addressing: value 0-7 (base_index encoding)
            let ea = if instr.src.value == 0xFF {
                // Direct addressing [disp16]: address is in disp field
                instr.src.disp as u16
            } else {
                // Indirect addressing [BX+SI], etc.: calculate EA
                let base_index = instr.src.value as u8;
                let (_seg_idx, ea) = cpu.calculate_ea_from_operand(&instr.src, base_index);
                ea
            };

            // Store the effective address (offset) in the destination register
            cpu.write_operand(_mem, &instr.dst, ea);
        }
        OperandType::Direct => {
            // Direct addressing: just use the offset directly
            cpu.write_operand(_mem, &instr.dst, instr.src.value);
        }
        _ => {
            // LEA with register operand is invalid (though some assemblers allow it)
            panic!("LEA requires a memory operand");
        }
    }
}

/// MOV r/m16, Sreg - Move segment register to register/memory
/// Opcode: 0x8C
///
/// Copies the value from a segment register to a 16-bit register or memory location.
/// The segment register is specified in the reg field of the ModR/M byte.
/// This instruction is useful for saving segment register values.
/// No flags are affected.
pub fn mov_rm_sreg(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let seg_value = cpu.read_operand(mem, &instr.src);
    cpu.write_operand(mem, &instr.dst, seg_value);
}

/// MOV Sreg, r/m16 - Move register/memory to segment register
/// Opcode: 0x8E
///
/// Copies a 16-bit value from a register or memory location to a segment register.
/// The segment register is specified in the reg field of the ModR/M byte.
/// This instruction is used to set up segment registers.
/// No flags are affected.
///
/// Note: On the 8088, loading CS via this instruction is not allowed.
/// Some sources indicate the behavior is undefined if attempted.
pub fn mov_sreg_rm(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let src_value = cpu.read_operand(mem, &instr.src);
    cpu.write_operand(mem, &instr.dst, src_value);
}

/// MOV AL, moffs8 - Move byte at memory offset to AL
/// Opcode: 0xA0
///
/// Loads a byte from the memory location specified by a 16-bit offset into AL.
/// The offset is encoded directly in the instruction (no ModR/M byte).
/// The segment can be overridden with a segment prefix.
/// No flags are affected.
pub fn mov_al_moffs(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let src_value = cpu.read_operand(mem, &instr.src);
    cpu.write_reg8(0, src_value as u8); // AL
}

/// MOV AX, moffs16 - Move word at memory offset to AX
/// Opcode: 0xA1
///
/// Loads a word from the memory location specified by a 16-bit offset into AX.
/// The offset is encoded directly in the instruction (no ModR/M byte).
/// The segment can be overridden with a segment prefix.
/// No flags are affected.
pub fn mov_ax_moffs(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let src_value = cpu.read_operand(mem, &instr.src);
    cpu.write_reg16(0, src_value); // AX
}

/// MOV moffs8, AL - Move AL to byte at memory offset
/// Opcode: 0xA2
///
/// Stores the byte in AL to the memory location specified by a 16-bit offset.
/// The offset is encoded directly in the instruction (no ModR/M byte).
/// The segment can be overridden with a segment prefix.
/// No flags are affected.
pub fn mov_moffs_al(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let al_value = cpu.read_reg8(0); // AL
    cpu.write_operand(mem, &instr.dst, al_value as u16);
}

/// MOV moffs16, AX - Move AX to word at memory offset
/// Opcode: 0xA3
///
/// Stores the word in AX to the memory location specified by a 16-bit offset.
/// The offset is encoded directly in the instruction (no ModR/M byte).
/// The segment can be overridden with a segment prefix.
/// No flags are affected.
pub fn mov_moffs_ax(cpu: &mut Cpu, mem: &mut MemoryBus, instr: &DecodedInstruction) {
    let ax_value = cpu.read_reg16(0); // AX
    cpu.write_operand(mem, &instr.dst, ax_value);
}
