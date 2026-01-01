//! Tests for instruction decoding (ModR/M and operand decoding)

use ezpc::cpu::decode::{AddressingMode, ModRM, Operand, OperandType};
use ezpc::cpu::Cpu;
use ezpc::memory::MemoryBus;

#[test]
fn test_modrm_register_direct() {
    // ModR/M byte: mod=11, reg=001 (CX), r/m=010 (DX)
    let modrm_byte = 0b11_001_010;
    let modrm = ModRM::decode(modrm_byte);

    assert_eq!(modrm.mod_bits, 0b11);
    assert_eq!(modrm.reg, 0b001);
    assert_eq!(modrm.rm, 0b010);
    assert!(modrm.is_register_direct());
    assert_eq!(modrm.get_rm_register(), 0b010);

    match modrm.mode {
        AddressingMode::RegisterDirect { rm_reg } => {
            assert_eq!(rm_reg, 0b010);
        }
        _ => panic!("Expected RegisterDirect mode"),
    }
}

#[test]
fn test_modrm_memory_indirect() {
    // ModR/M byte: mod=00, reg=000, r/m=111 ([BX])
    let modrm_byte = 0b00_000_111;
    let modrm = ModRM::decode(modrm_byte);

    assert_eq!(modrm.mod_bits, 0b00);
    assert_eq!(modrm.reg, 0b000);
    assert_eq!(modrm.rm, 0b111);
    assert!(!modrm.is_register_direct());

    match modrm.mode {
        AddressingMode::MemoryIndirect { base_index } => {
            assert_eq!(base_index, 0b111);
        }
        _ => panic!("Expected MemoryIndirect mode"),
    }
}

#[test]
fn test_modrm_memory_disp8() {
    // ModR/M byte: mod=01, reg=011, r/m=110 ([BP+disp8])
    let modrm_byte = 0b01_011_110;
    let modrm = ModRM::decode(modrm_byte).with_disp8(0x10);

    assert_eq!(modrm.mod_bits, 0b01);
    assert_eq!(modrm.reg, 0b011);
    assert_eq!(modrm.rm, 0b110);

    match modrm.mode {
        AddressingMode::MemoryDisp8 { base_index, disp } => {
            assert_eq!(base_index, 0b110);
            assert_eq!(disp, 0x10);
        }
        _ => panic!("Expected MemoryDisp8 mode"),
    }
}

#[test]
fn test_modrm_memory_disp16() {
    // ModR/M byte: mod=10, reg=100, r/m=101 ([DI+disp16])
    let modrm_byte = 0b10_100_101;
    let modrm = ModRM::decode(modrm_byte).with_disp16(0x1234);

    assert_eq!(modrm.mod_bits, 0b10);
    assert_eq!(modrm.reg, 0b100);
    assert_eq!(modrm.rm, 0b101);

    match modrm.mode {
        AddressingMode::MemoryDisp16 { base_index, disp } => {
            assert_eq!(base_index, 0b101);
            assert_eq!(disp, 0x1234);
        }
        _ => panic!("Expected MemoryDisp16 mode"),
    }
}

#[test]
fn test_modrm_direct_address() {
    // ModR/M byte: mod=00, reg=000, r/m=110 (direct address)
    let modrm_byte = 0b00_000_110;
    let modrm = ModRM::decode(modrm_byte).with_direct_addr(0x5678);

    assert_eq!(modrm.mod_bits, 0b00);
    assert_eq!(modrm.reg, 0b000);
    assert_eq!(modrm.rm, 0b110);

    match modrm.mode {
        AddressingMode::DirectAddress { addr } => {
            assert_eq!(addr, 0x5678);
        }
        _ => panic!("Expected DirectAddress mode"),
    }
}

#[test]
fn test_modrm_calculate_address_direct() {
    let mut cpu = Cpu::new();
    cpu.write_reg16(3, 0x1000); // BX

    // [BX] - mod=00, r/m=111
    let modrm_byte = 0b00_000_111;
    let modrm = ModRM::decode(modrm_byte);

    let (seg, offset) = modrm.calculate_address(&cpu);
    assert_eq!(seg, 3); // DS default
    assert_eq!(offset, 0x1000);
}

#[test]
fn test_modrm_calculate_address_bx_si() {
    let mut cpu = Cpu::new();
    cpu.write_reg16(3, 0x1000); // BX
    cpu.write_reg16(6, 0x0200); // SI

    // [BX+SI] - mod=00, r/m=000
    let modrm_byte = 0b00_000_000;
    let modrm = ModRM::decode(modrm_byte);

    let (seg, offset) = modrm.calculate_address(&cpu);
    assert_eq!(seg, 3); // DS default
    assert_eq!(offset, 0x1200);
}

#[test]
fn test_modrm_calculate_address_bp_di_disp8() {
    let mut cpu = Cpu::new();
    cpu.write_reg16(5, 0x2000); // BP
    cpu.write_reg16(7, 0x0100); // DI

    // [BP+DI+0x10] - mod=01, r/m=011
    let modrm_byte = 0b01_000_011;
    let modrm = ModRM::decode(modrm_byte).with_disp8(0x10);

    let (seg, offset) = modrm.calculate_address(&cpu);
    assert_eq!(seg, 2); // SS default for BP-based addressing
    assert_eq!(offset, 0x2110);
}

#[test]
fn test_modrm_calculate_address_si_disp16() {
    let mut cpu = Cpu::new();
    cpu.write_reg16(6, 0x1000); // SI

    // [SI+0x1234] - mod=10, r/m=100
    let modrm_byte = 0b10_000_100;
    let modrm = ModRM::decode(modrm_byte).with_disp16(0x1234);

    let (seg, offset) = modrm.calculate_address(&cpu);
    assert_eq!(seg, 3); // DS default
    assert_eq!(offset, 0x2234);
}

#[test]
fn test_cpu_fetch_u8() {
    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::new();

    // Set CS:IP to 0000:0100
    cpu.write_seg(1, 0x0000);
    cpu.ip = 0x0100;

    // Write test bytes to memory
    mem.write_u8(0x0100, 0x42);
    mem.write_u8(0x0101, 0xAB);

    assert_eq!(cpu.fetch_u8(&mem), 0x42);
    assert_eq!(cpu.ip, 0x0101);
    assert_eq!(cpu.fetch_u8(&mem), 0xAB);
    assert_eq!(cpu.ip, 0x0102);
}

#[test]
fn test_cpu_fetch_u16() {
    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::new();

    cpu.write_seg(1, 0x0000);
    cpu.ip = 0x0100;

    // Write little-endian word: 0x3412
    mem.write_u8(0x0100, 0x12);
    mem.write_u8(0x0101, 0x34);

    assert_eq!(cpu.fetch_u16(&mem), 0x3412);
    assert_eq!(cpu.ip, 0x0102);
}

#[test]
fn test_cpu_fetch_i8() {
    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::new();

    cpu.write_seg(1, 0x0000);
    cpu.ip = 0x0100;

    mem.write_u8(0x0100, 0xFF); // -1
    mem.write_u8(0x0101, 0x7F); // 127

    assert_eq!(cpu.fetch_i8(&mem), -1);
    assert_eq!(cpu.fetch_i8(&mem), 127);
}

#[test]
fn test_cpu_fetch_i16() {
    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::new();

    cpu.write_seg(1, 0x0000);
    cpu.ip = 0x0100;

    // Write little-endian word: 0xFFFF (-1)
    mem.write_u8(0x0100, 0xFF);
    mem.write_u8(0x0101, 0xFF);

    assert_eq!(cpu.fetch_i16(&mem), -1);
}

#[test]
fn test_cpu_decode_modrm_register_direct() {
    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::new();

    cpu.write_seg(1, 0x0000);
    cpu.ip = 0x0100;

    // ModR/M byte: mod=11, reg=001, r/m=010
    mem.write_u8(0x0100, 0b11_001_010);

    let modrm = cpu.decode_modrm(&mem);
    assert_eq!(modrm.reg, 0b001);
    assert_eq!(modrm.rm, 0b010);
    assert!(modrm.is_register_direct());
    assert_eq!(cpu.ip, 0x0101); // IP advanced by 1
}

#[test]
fn test_cpu_decode_modrm_with_disp8() {
    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::new();

    cpu.write_seg(1, 0x0000);
    cpu.ip = 0x0100;

    // ModR/M byte: mod=01, reg=000, r/m=110
    mem.write_u8(0x0100, 0b01_000_110);
    // Displacement: 0x20
    mem.write_u8(0x0101, 0x20);

    let modrm = cpu.decode_modrm(&mem);
    assert_eq!(cpu.ip, 0x0102); // IP advanced by 2

    match modrm.mode {
        AddressingMode::MemoryDisp8 { base_index, disp } => {
            assert_eq!(base_index, 0b110);
            assert_eq!(disp, 0x20);
        }
        _ => panic!("Expected MemoryDisp8"),
    }
}

#[test]
fn test_cpu_decode_modrm_with_disp16() {
    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::new();

    cpu.write_seg(1, 0x0000);
    cpu.ip = 0x0100;

    // ModR/M byte: mod=10, reg=000, r/m=111
    mem.write_u8(0x0100, 0b10_000_111);
    // Displacement: 0x1234 (little-endian)
    mem.write_u8(0x0101, 0x34);
    mem.write_u8(0x0102, 0x12);

    let modrm = cpu.decode_modrm(&mem);
    assert_eq!(cpu.ip, 0x0103); // IP advanced by 3

    match modrm.mode {
        AddressingMode::MemoryDisp16 { base_index, disp } => {
            assert_eq!(base_index, 0b111);
            assert_eq!(disp, 0x1234);
        }
        _ => panic!("Expected MemoryDisp16"),
    }
}

#[test]
fn test_cpu_decode_modrm_direct_address() {
    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::new();

    cpu.write_seg(1, 0x0000);
    cpu.ip = 0x0100;

    // ModR/M byte: mod=00, reg=000, r/m=110 (special case)
    mem.write_u8(0x0100, 0b00_000_110);
    // Direct address: 0x5678 (little-endian)
    mem.write_u8(0x0101, 0x78);
    mem.write_u8(0x0102, 0x56);

    let modrm = cpu.decode_modrm(&mem);
    assert_eq!(cpu.ip, 0x0103); // IP advanced by 3

    match modrm.mode {
        AddressingMode::DirectAddress { addr } => {
            assert_eq!(addr, 0x5678);
        }
        _ => panic!("Expected DirectAddress"),
    }
}

#[test]
fn test_decode_reg_operand() {
    // 8-bit register operands
    let op = Cpu::decode_reg_operand(0, true);
    assert_eq!(op.op_type, OperandType::Reg8);
    assert_eq!(op.value, 0); // AL

    let op = Cpu::decode_reg_operand(7, true);
    assert_eq!(op.op_type, OperandType::Reg8);
    assert_eq!(op.value, 7); // BH

    // 16-bit register operands
    let op = Cpu::decode_reg_operand(0, false);
    assert_eq!(op.op_type, OperandType::Reg16);
    assert_eq!(op.value, 0); // AX

    let op = Cpu::decode_reg_operand(7, false);
    assert_eq!(op.op_type, OperandType::Reg16);
    assert_eq!(op.value, 7); // DI
}

#[test]
fn test_decode_rm_operand_register() {
    // Register direct mode, 8-bit
    let modrm = ModRM::decode(0b11_000_010); // DL
    let op = Cpu::decode_rm_operand(&modrm, true);
    assert_eq!(op.op_type, OperandType::Reg8);
    assert_eq!(op.value, 2);

    // Register direct mode, 16-bit
    let modrm = ModRM::decode(0b11_000_101); // BP
    let op = Cpu::decode_rm_operand(&modrm, false);
    assert_eq!(op.op_type, OperandType::Reg16);
    assert_eq!(op.value, 5);
}

#[test]
fn test_decode_rm_operand_memory() {
    // Memory indirect, 8-bit
    let modrm = ModRM::decode(0b00_000_111); // [BX]
    let op = Cpu::decode_rm_operand(&modrm, true);
    assert_eq!(op.op_type, OperandType::Mem8);
    assert_eq!(op.value, 0b111);

    // Memory indirect, 16-bit
    let modrm = ModRM::decode(0b00_000_000); // [BX+SI]
    let op = Cpu::decode_rm_operand(&modrm, false);
    assert_eq!(op.op_type, OperandType::Mem16);
    assert_eq!(op.value, 0b000);
}

#[test]
fn test_operand_constructors() {
    let op = Operand::none();
    assert_eq!(op.op_type, OperandType::None);

    let op = Operand::reg8(3);
    assert_eq!(op.op_type, OperandType::Reg8);
    assert_eq!(op.value, 3);

    let op = Operand::reg16(5);
    assert_eq!(op.op_type, OperandType::Reg16);
    assert_eq!(op.value, 5);

    let op = Operand::imm8(0x42);
    assert_eq!(op.op_type, OperandType::Imm8);
    assert_eq!(op.value, 0x42);

    let op = Operand::imm16(0x1234);
    assert_eq!(op.op_type, OperandType::Imm16);
    assert_eq!(op.value, 0x1234);

    let op = Operand::rel8(-10);
    assert_eq!(op.op_type, OperandType::Rel8);

    let op = Operand::seg(2);
    assert_eq!(op.op_type, OperandType::SegReg);
    assert_eq!(op.value, 2);
}
