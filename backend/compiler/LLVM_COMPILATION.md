# LLVM IR Compilation

This document describes the LLVM IR code generation functionality in the Dream Language compiler.

## Overview

The compiler now supports full LLVM IR code generation, allowing Dream Language programs to be compiled to native machine code through LLVM's powerful optimization and code generation infrastructure.

## Features

### Supported Language Features

- ✅ **Functions**: Function declarations with parameters and return types
- ✅ **Variables**: Local variable declarations with type inference
- ✅ **Constants**: Constant declarations
- ✅ **Binary Operations**: Arithmetic (`+`, `-`, `*`, `/`), comparison (`<`, `>`, `<=`, `>=`, `==`, `!=`), logical (`&&`, `||`)
- ✅ **Unary Operations**: Negation (`-`), logical NOT (`!`)
- ✅ **Control Flow**: If/else statements, while loops, do-until loops
- ✅ **Function Calls**: Calling functions with arguments
- ✅ **Type System**: Int, Float, Bool, String, Void

### Code Generation

The LLVM compiler (`src/llvm_compiler.rs`) generates optimized LLVM IR code with:

- Stack allocation for local variables
- SSA (Static Single Assignment) form
- Proper control flow graphs
- Function calling conventions
- Type safety

## Usage

### Via gRPC Service

The `LlvmTranslate` RPC endpoint compiles source code to LLVM IR:

```protobuf
service Compiler {
  rpc LlvmTranslate(CompilerRequest) returns (LLVMTranslateResponse);
}

message CompilerRequest {
  string source = 1;
}

message LLVMTranslateResponse {
  string llvm_ir = 1;
}
```

### Programmatically

```rust
use compiler::ast::Program;
use compiler::llvm_compiler::compile_to_llvm_ir;

// After parsing your source code to an AST:
match compile_to_llvm_ir(&program) {
    Ok(llvm_ir) => {
        // llvm_ir contains the generated LLVM IR as a string
        println!("{}", llvm_ir);
    }
    Err(e) => {
        eprintln!("Compilation error: {}", e);
    }
}
```

## Example

### Source Code

```dream
fn factorial(n: Int) -> Int {
    let result: Int = 1;
    return n * result;
}

fn main() -> Int {
    let x: Int = 5;
    return factorial(x);
}
```

### Generated LLVM IR

```llvm
; ModuleID = 'dream_compiler'
source_filename = "dream_compiler"

define i64 @factorial(i64 %0) {
entry:
  %result = alloca i64, align 8
  %n = alloca i64, align 8
  store i64 %0, ptr %n, align 4
  store i64 1, ptr %result, align 4
  %n1 = load i64, ptr %n, align 4
  %result2 = load i64, ptr %result, align 4
  %tmpmul = mul i64 %n1, %result2
  ret i64 %tmpmul
}

define i64 @main() {
entry:
  %x = alloca i64, align 8
  store i64 5, ptr %x, align 4
  %x1 = load i64, ptr %x, align 4
  %tmp = call i64 @factorial(i64 %x1)
  ret i64 %tmp
}
```

### Optimized IR (with -O3)

```llvm
define i64 @factorial(i64 returned %0) local_unnamed_addr #0 {
entry:
  ret i64 %0
}

define noundef i64 @main() local_unnamed_addr #0 {
entry:
  ret i64 5
}
```

Notice how LLVM's optimizer completely simplified the computation!

## Compilation Pipeline

1. **Lexical Analysis**: Source code → Tokens
2. **Parsing**: Tokens → AST
3. **Semantic Analysis**: AST → Type-checked AST with symbol table
4. **LLVM IR Generation**: AST → LLVM IR
5. **LLVM Optimization**: LLVM IR → Optimized LLVM IR (optional)
6. **Code Generation**: LLVM IR → Machine Code (via LLC)

## Using LLVM Tools

Once LLVM IR is generated, you can use LLVM's toolchain:

### Validate IR
```bash
llvm-as-18 output.ll -o output.bc
```

### Optimize
```bash
opt-18 -O3 output.bc -o output_opt.bc
llvm-dis-18 output_opt.bc -o output_opt.ll
```

### Compile to Assembly
```bash
llc-18 output.bc -o output.s -filetype=asm
```

### Compile to Object File
```bash
llc-18 output.bc -o output.o -filetype=obj
```

### Link and Create Executable
```bash
clang output.o -o program
./program
echo $?  # Print return code
```

## Implementation Details

### Type Mapping

| Dream Type | LLVM Type |
|------------|-----------|
| Int        | i64       |
| Float      | double    |
| Bool       | i1        |
| String     | i8*       |
| Void       | void      |

### Architecture

The LLVM compiler is implemented in `src/llvm_compiler.rs` using the `inkwell` crate, which provides safe Rust bindings to LLVM.

Key components:
- **Compiler struct**: Maintains compilation state (context, builder, module, variables)
- **compile()**: Entry point that processes all declarations
- **compile_function()**: Generates code for function declarations
- **compile_expression()**: Handles expression code generation
- **compile_statement()**: Handles statement code generation

### Optimizations

The compiler includes a pass manager that can run LLVM optimization passes on generated functions. Currently configured with basic optimizations.

## Testing

Run the included test to verify LLVM compilation:

```bash
cd backend/compiler
cargo build

# Create test program
cat > /tmp/test.rs << 'EOF'
use compiler::ast::*;
use compiler::llvm_compiler::compile_to_llvm_ir;

fn main() {
    // Create your AST here
    let program = Program { declarations: vec![] };
    
    match compile_to_llvm_ir(&program) {
        Ok(ir) => println!("{}", ir),
        Err(e) => eprintln!("Error: {}", e),
    }
}
EOF

rustc --edition 2021 -L target/debug/deps /tmp/test.rs -o /tmp/test \
    --extern compiler=target/debug/libcompiler.rlib
/tmp/test
```

## Future Enhancements

Potential improvements:
- [ ] For loop code generation
- [ ] Array and object literal support
- [ ] Struct type support
- [ ] Member access expressions
- [ ] String operations
- [ ] More sophisticated optimization passes
- [ ] Debug information generation (DWARF)
- [ ] JIT compilation support
- [ ] Link-time optimization (LTO)

## Dependencies

- `inkwell` 0.6.0 - LLVM bindings for Rust
- `llvm-sys` 170.0.0 - Low-level LLVM bindings
- LLVM 17.0+ - The LLVM compiler infrastructure

## References

- [LLVM Language Reference](https://llvm.org/docs/LangRef.html)
- [Inkwell Documentation](https://thedan64.github.io/inkwell/)
- [LLVM Programmer's Manual](https://llvm.org/docs/ProgrammersManual.html)
