# Claude Code Instructions for 8088 IBM PC Emulator

## Project Overview

This is a high-performance, cycle-accurate implementation of the original
IBM PC, based around an Intel 8088 processor. High-performance emulation is
achieved by a three-tier execution system:
- **Tier 1 (Cold)**: Direct dispatch for first-time execution
- **Tier 2 (Warm)**: Decode cache for frequently executed code
- **Tier 3 (Hot)**: Compiled basic blocks for hottest code paths

The goal is to build an "insanely efficient" emulator that progressively
optimizes code as it executes.

## Primary Reference Document

**ALWAYS read `ARCH.md` first** before starting any implementation work.
This document contains:
- Architecture and design
- Detailed implementation phases
- Module structure and organization
- Testing strategy
- Performance targets

When implementing any feature, refer to the relevant section in ARCH.md for
guidance.

## Git Workflow

### Commit Standards

**CRITICAL**: Make incremental, atomic commits that represent single logical
changes. For example, when adding a new CPU instruction, the commit should
include opcode decoding, implementation logic, and tests proving it works.

#### Good Commit Examples:
```
✓ Add CPU state structure with register arrays
✓ Implement inline register access methods
✓ Add ModR/M byte decoder
✓ Implement MOV r/m8, r8 handler (0x88)
✓ Add decode cache
✓ Implement tier 2 execution path
```

#### Bad Commit Examples:
```
✗ Implement all of Phase 1
✗ Add instructions
✗ Fix stuff
✗ WIP
✗ Updates
```

### Commit Guidelines

1. **One complete logical change per commit**
   - Adding a single struct/function
   - Implementing one instruction handler, with tests
   - Fixing a specific bug
   - Adding a specific test

2. **Commit message format**:
   ```
   <type>: <concise description>
   
   Optional longer description as needed, to explain why the change was made
   or any important details.
   ```

3. **Commit types**
   - `feat`: New feature or functionality
   - `fix`: Bug fix
   - `refactor`: Code restructuring without behavior changes
   - `test`: Adding or updating tests
   - `docs`: Documentation changes
   - `perf`: Performance improvements
   - `style`: Code style/formatting changes

### Before Each Commit

- Run `cargo check` to ensure code compiles
- Run `cargo test` to make sure tests still pass, even if tests don't cover the
  area that was changed
- Run `cargo fmt` to format code, if any Rust files were modified
- Verify your change is complete and functional

### After Each Commit

- Run `git status` and make sure there are no uncommitted changes. If there are,
add them to the most recent commit or create a new commit as appropriate.

## Implementation Approach

### Phase-by-Phase Development

Follow the ARCH.md phases in order:

#### CPU Implementation:

1. **Phase 1**: CPU core (state, registers, memory flags)
2. **Phase 2**: Instruction decoding
3. **Phase 3**: Initial instruction handlers (start with ~20 core instructions)
4. **Phase 4**: Tier 1 execution (cold path)
5. **Phase 5**: Cycle counting (basic instruction cycles, memory timing, prefetch queue)
6. **Phase 6**: Tier 2 execution (decode cache)
7. **Phase 7**: Tier 3 (basic blocks)
8. **Phase 8**: Profiler / Benchmarking
9. **Phase 9**: Implement all instructions

#### Emulator Implementation:

1. **Phase 10**: 

### When Starting a New Phase

1. Read the relevant phase section in ARCH.md completely
2. Create the module structure first
3. Implement basic structures/types
4. Add implementations incrementally
5. Test each component as you go
6. Commit frequently

### Code Quality Standards

1. **Use Rust idioms**
   - Prefer `Option<T>` over nullable values
   - Use `Result<T, E>`  for operations that can fail
   - Leverage pattern matching
   - Use iterator methods where appropriate

2. **Performance considerations**
   - Use `#[inline(always)]` on hot path functions
   - Use `unsafe` only when necessary and document why (for example, array access when bounds won't be exceeded)
   - Prefer array indexing over HashMap lookups in hot paths
   - Keep functions small for better inlining

3. **Documentation**:
   - Add doc comments (`///`) for public functions and structs
   - Include examples in doc comments *when helpful*
   - Document invariants and safety requirements for unsafe code
   - Comment non-obvious algorithm choices

4. **Error handling**
   - Use `panic!` for unrecoverable errors
   - Use `debug_assert!` for invariant checking
   - Provide clear error messages with context

## Testing Strategy

### Test Organization

Tests should be organized by functionality:
- `src/tests/cpu/basic.rs` - Basic instruction tests
- `src/tests/cpu/arithmetic.rs` - Arithmetic instruction tests
- `src/tests/cpu/control_flow.rs` - Jump/call/return tests
- `src/tests/benchmarks.rs` - Performance benchmarks
- `src/tests/memory.rs` - Memory access tests

### Test Requirements

1. **Write tests as you implement**
   - Add at least one test per instruction handler
   - Test both register and memory operands
   - Test edge cases (overflow, zero, etc.)

2. **Test structure**
   ```rust
   fn test_instruction_name() {
       // CpuHarness is not a full emulator, but contains just enough elements
       // to check CPU execution and correctness
       let mut harness = CpuHarness::new();
       // Setup
       harness.load_program(&[/* bytecode */], 0);
       // [Include a comment with the assembly corresponding to the bytecode]
       // eg: "MOV AX, 0x55; NOP"

       // Execute
       harness.step();

       // Verify
       assert_eq!(harness.cpu.regs[0], expected_value);

       // [include more `step` and assertions as needed]
   }
   ```

3. **Run tests frequently**:
   ```bash
   cargo test           # All tests
   cargo test test_name # Specific test
   cargo test --release # Optimized tests
   ```

4. **One behavior, one test**:
   Each test should cover one expected behavior. If you are testing different
   behaviors (eg. flags after arithmetic instructions), each case should get
   its own test.

## Building and Running

### Development Build
```bash
cargo build
cargo run
```

### Release Build (for benchmarking)
```bash
cargo build --release
cargo run --release
```

### Running Tests
```bash
cargo test
cargo test -- --nocapture  # show println! output
```

### Benchmarking
```bash
cargo test --release benchmarks -- --nocapture
```

## Project Structure Reference

```
ezpc/
├── src/
└── ARCH.md               # Detailed implementation plan
```

## Development Workflow

### Starting a New Feature

1. Read relevant setion in ARCH.md
2. Implement incrementally with one or more commits
3. Add tests, and verify with `cargo test`, for each change
4. Commit final changes

### Example Session: Implementing an Instruction

```bash
# 1. Create handler with instruction implementation
vim src/cpu/execute/arithmetic.rs

# 2. Add operand decoding
vim src/cpu/tier1/decode.rs

# 3. Add to dispatch table
vim src/cpu/tier1/dispatch.rs

# 4. Add tests
vim src/tests/cpu/arithmetic.rs

# 5. Verify everything works
cargo test test_sub_rm8_r8
cargo test

# 6. Commit changes
git add src/cpu/execute/arithmetic.rs
git add src/cpu/tier1/decode.rs
git add src/cpu/tier1/dispatch.rs
git add src/tests/cpu/arithmetic.rs
git commit -m "feat: implement SUB r/m8, r8 instruction with tests"
```

## Communication Style

When working on this project:

1. **Be explicit about what you're doing**
   - "I'm implementing the MOV instruction handler"
   - "Adding tests for arithmetic operations"
   - "Refactoring the decode cache for better performance"

2. **Explain non-obvious decisions**
   - "Using u32 for last_result to handle carry flag computation"
   - "Inlining this function because it's called millions of times per second"

3. **Reference ARCH.md sections**
   - "Following Phase 3 from ARCH.md"
   - "Implementing the decode cache as described in Phase 5"

4. **Report progress**:
   - "Completed phase 1: CPU state implementation"
   - "Added 15/256 instruction handlers"
   - "Tier 2 decode cache achieving 95% hit rate in tests"

5. **Ask for clarification when needed**:
   - "The ARCH.md shows X, but should we also handle Y?"
   - "Should we implement the full 8088 instruction set or start with a subset?"

## Performance Monitoring

Performance will be critical for this project once most of the CPU has been
implemented. For now, just make design decisions that will maximize performance.
We will track key metrics later.

## Common Patterns

### Adding a New Instruction

1. Create handler in appropriate `cpu/execute/` module
2. Update `decode_operands_for_opcode` in `cpu/tier1/decode.rs` if needed
3. Add to dispatch table in `cpu/tier1/dispatch.rs`
4. Add tests in appropriate test module
5. Verify with `cargo test`

### Adding a New Module

1. Create file: `src/module_name/mod.rs`
2. Add to parent: `pub mod module_name;` in parent module
3. Export public items: `pub use module_name::ItemName;`
4. Document module purpose at top of file

### Debugging tips

1. **Print CPU state**:
   ```rust
   println!("AX={:04X} CX={:04X} IP={:04X}", 
            cpu.regs[0], cpu.regs[1], cpu.ip);
   ```

2. **Trace instruction execution**:
   ```rust
   println!("Executing {:02X} at {:04X}", opcode, ip);
   ```

3. **Verify flags**:
   ```rust
   println!("Flags: {:016b}", cpu.compute_flags());
   ```

## Priorities

1. **Correctness first** - Get it working right
2. **Test coverage** - Verify it works
3. **Performance** - Make it fast (but only after 1 & 2)
4. **Code quality** - Keep it maintainable

## Red Flags to Avoid

- ❌ Committing broken/non-compiling code
- ❌ Large commits with many unrelated changes
- ❌ Implementing features not in ARCH.md without discussion
- ❌ Skipping tests
- ❌ Using `unwrap()` without checking if panic is acceptable
- ❌ Copying code without understanding it
- ❌ Premature optimization before measuring

## Green Flags to Embrace

- ✅ Small, focused commits with clear messages
- ✅ Tests that verify behavior
- ✅ Following the ARCH.md structure
- ✅ Measuring before optimizing
- ✅ Clear comments explaining "why" not "what"
- ✅ Using Rust idioms and best practices
- ✅ Asking questions when uncertain

## Questions?

If anything is unclear or you need guidance:
1. Check ARCH.md first
2. Look for similar existing implementations in the codebase
3. Ask for clarification with specific context

Remember: Small commits, frequent testing, follow the ARCH.md, and prioritize
correctness over performance initially.
