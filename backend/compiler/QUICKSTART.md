# Quick Start: LLVM Compilation

## Build the Compiler

```bash
cd backend/compiler
cargo build
```

## Run the Compiler Service

```bash
cargo run
```

The gRPC server will start on `127.0.0.1:50051`.

## Using the LlvmTranslate Endpoint

### With grpcurl

```bash
# Compile source code to LLVM IR
grpcurl -plaintext -d '{
  "source": "fn add(a: Int, b: Int) -> Int { return a + b; }"
}' localhost:50051 compiler.Compiler/LlvmTranslate
```

### Programmatically (Rust)

```rust
use compiler::llvm_compiler::compile_to_llvm_ir;
use compiler::ast::*;

// Create your AST
let program = Program {
    declarations: vec![
        Declaration::Function(Function {
            name: Identifier { name: "add".to_string(), line: 1, column: 1 },
            parameters: vec![
                Parameter { 
                    name: Identifier { name: "a".to_string(), line: 1, column: 8 },
                    param_type: Type::Int 
                },
                Parameter { 
                    name: Identifier { name: "b".to_string(), line: 1, column: 16 },
                    param_type: Type::Int 
                },
            ],
            return_type: Type::Int,
            body: Block {
                statements: vec![
                    Declaration::Statement(Statement::Return(ReturnStatement {
                        value: Expression::Binary {
                            left: Box::new(Expression::Identifier(
                                Identifier { name: "a".to_string(), line: 1, column: 37 }
                            )),
                            op: BinaryOp::Plus,
                            right: Box::new(Expression::Identifier(
                                Identifier { name: "b".to_string(), line: 1, column: 41 }
                            )),
                        },
                    })),
                ],
            },
        }),
    ],
};

// Compile to LLVM IR
match compile_to_llvm_ir(&program) {
    Ok(llvm_ir) => {
        println!("{}", llvm_ir);
        // Save to file
        std::fs::write("output.ll", llvm_ir).unwrap();
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Process LLVM IR

Once you have the LLVM IR, you can use LLVM tools:

```bash
# Validate
llvm-as-18 output.ll -o output.bc

# Optimize
opt-18 -O3 output.bc -o output_opt.bc

# View optimized IR
llvm-dis-18 output_opt.bc -o output_opt.ll

# Compile to assembly
llc-18 output.bc -o output.s

# Compile to object file
llc-18 output.bc -o output.o -filetype=obj

# Link to executable (requires a main function)
clang output.o -o program

# Run
./program
echo $?  # Print exit code
```

## Example Workflow

```bash
# 1. Start the compiler service
cargo run &

# 2. Send code to compile
grpcurl -plaintext -d '{
  "source": "fn main() -> Int { return 42; }"
}' localhost:50051 compiler.Compiler/LlvmTranslate > response.json

# 3. Extract LLVM IR
cat response.json | jq -r '.llvmIr' > program.ll

# 4. Compile to executable
llvm-as-18 program.ll -o program.bc
llc-18 program.bc -o program.o -filetype=obj
clang program.o -o program

# 5. Run
./program
echo $?  # Should print: 42
```

## Supported Features

✅ Functions with parameters and return types
✅ Local variables
✅ Arithmetic operations (+, -, *, /)
✅ Comparison operations (<, >, <=, >=, ==, !=)
✅ Logical operations (&&, ||, !)
✅ If/else statements
✅ While loops
✅ Do-until loops
✅ Function calls
✅ Return statements

## Troubleshooting

### "Module verification failed"
The generated IR is invalid. Check that your AST is well-formed and types match.

### "Undefined variable"
Variable was used before being declared or is out of scope.

### "Undefined function"
Function was called before being declared. Declare functions before calling them.

### Build errors with LLVM
Make sure you have LLVM 17+ installed:
```bash
llvm-config-18 --version  # Should show 18.x.x
```

## Documentation

- [LLVM_COMPILATION.md](./LLVM_COMPILATION.md) - Full documentation
- [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md) - Implementation details
- [proto/parser.proto](./proto/parser.proto) - gRPC service definitions
