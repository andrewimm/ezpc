# Architecture design for IBM PC Emulator

## Project Overview

Goal: build a high-performance, cycle-accurate implementation of the original
IBM PC, including Intel 8088 emulation. Accurate and fast implementation is
achieved by a three-tier execution system:
1. **Tier 1: Cold Path** - Direct dispatch with function pointers for first-time execution
2. **Tier 2: Warm Path** - Decode cache for frequently executed instructions
3. **Tier 3: Hot Path** - Compiled Basic Blocks for the hottest code

## Architecture

### CPU

CPU state is stored in a struct that tracks:
 - Registers (stored as an array for indexed access from opcodes)
 - Segments (stored as an array for indexed access from opcodes)
 - IP and Flag registers (u16)
 - Lazy flag eval state (store the last result and op type)
 - Total executed cycles (u64)
 - Cycles tracked for current instruction (reset at the beginning of each instruction)
 - Prefetch queue (array of bytes, current queue length, cycles spent filling queue)

The first time an instruction is encountered, it is decoded and executed using
direct dispatch to the appropriate handler. At the same time, the decoded
information is cached, so that decoding is unnecessary the next time the IP
register visits this instruction.

Visits to each instruction are recorded. After 100 visits to a specific code
path, a basic block is compiled, which can be run more efficiently the next
time the entry point is visited.

Cycle tracking is very important

### Memory Bus

The IBM PC has a specific layout of its memory bus. To simulate this, we use a
MemoryBus struct that has read/write byte/word methods. The MemoryBus has array
fields to emulate PC RAM (64KiB) and ROM. When peripherals are introduced, it
will also add the ability for MMIO.

### Emulator

Once the CPU is implemented, the full PC emulator should be implemented. This
will require creation of a window (winit), a graphics context (wgpu), and
an audio subsystem.

In order to achieve smooth emulation, the emulator uses dynamic rate control.
An audio sync system determines if there are too few (or too many) samples and
generates a speed multiplier which can modify CPU cycles per frame.

## Project Structure

Create and maintain the following module structure:
```
ezpc/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Library root
│   ├── cpu/
│   │   ├── mod.rs                # CPU module root
│   │   ├── state.rs              # CPU registers and flags
│   │   ├── flags.rs              # Flag computation (lazy evaluation)
│   │   ├── prefetch.rs           # Prefetch queue methods
│   │   ├── harness.rs            # Test harness (cpu state, memory bus)
│   │   ├── decode/
│   │   │   ├── mod.rs            # Decoder module root
│   │   │   ├── instruction.rs    # DecodedInstruction struct
│   │   │   ├── modrm.rs          # ModR/M byte parsing
│   │   │   └── operands.rs       # Operand decoding
│   │   ├── execute/
│   │   │   ├── mod.rs            # Execution module root
│   │   │   ├── arithmetic.rs     # ADD, SUB, INC, DEC, etc.
│   │   │   ├── control_flow.rs   # JMP, CALL, RET, Jcc, etc.
│   │   │   ├── data_transfer.rs  # MOV, XCHG, etc.
│   │   │   ├── handlers.rs       # Instruction handler functions, including invalid op handler
│   │   │   ├── logic.rs          # AND, OR, XOR, NOT, etc.
│   │   │   └── stack.rs          # PUSH, POP, etc.
│   │   ├── tier1/
│   │   │   ├── mod.rs         
│   │   │   ├── decode.rs         # CPU state methods for decoding the current instruction
│   │   │   └── dispatch.rs       # Direct dispatch table all opcodes
│   │   ├── tier2/
│   │   │   ├── mod.rs         
│   │   │   └── cache.rs          # Cache of decoded instructions
│   │   ├── tier3/
│   │   │   ├── mod.rs         
│   │   │   ├── block.rs          # Basic block structure
│   │   │   └── cache.rs          # Cache of decoded instructions
│   ├── memory.rs                 # Memory Bus struct (RAM and ROM arrays, MMIO)
└── tests/
    ├── cpu/                      # CPU tests
    │   ├── basic.rs              # Basic instruction tests
    │   ├── arithmetic.rs         # Arithmetic instruction tests
    │   └── control_flow.rs       # Jump/call/return tests
    ├── mod.rs
    ├── basic.rs                  # Basic instruction tests
    └── benchmarks.rs             # Performance benchmarks
```

Other emulator subsystems will be added when the CPU is complete.

## Phase 1: Core CPU State

[ ] Create CPU State Struct in `src/cpu/state.rs`, storing registers, segments, and lazy flag evaluation
[ ] Implement register access using the u8 values found in instructions
[ ] Implement memory access, using segment registers to compute the correct address for the memory bus
[ ] Implement lazy flag evaluation

## Phase 2: Instruction Decoding

[ ] Define DecodedInstruction struct for caching instruction details
[ ] Implement ModR/M decoder
[ ] Create operand decoder

## Phase 3: Instruction implementation

[ ] Create handler template, and handlers for invalid opcode, nop
[ ] Implement basic data transfer instructions (MOV, XCHG)
[ ] Implement basic stack operations (PUSH, POP)
[ ] Implement basic arithmetic (ADD, INC, DEC)
[ ] Implement basic control flow (JMP short/near, JZ, JNZ)

## Phase 4: Direct Dispatch (Tier 1)

[ ] Create dispatch table, with 256 entries pointing to dispatch handler functions
[ ] Implement tier 1 operand decoding
[ ] Implement test harness and tests for basic instructions

## Phase 4: Cycle counting

[ ] Add cycle tracking to CPU state (total cycles, current instruction cycles)
[ ] Implement prefetch queue for 8088 processor
[ ] Create timing tables (BASE_CYCLES, EA_CALC_CYCLES, transfer times)
[ ] Extend DecodedInstruction to include cycle costs
[ ] Update instruction handlers to track cycles during execution
[ ] Modify control flow to flush prefetch on branches
[ ] Add timing tests to verify cycle counts (not all instructions, just enough cases)

