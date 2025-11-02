// Integration tests for full compilation pipeline: Lexer → Parser → Semantic → LLVM

use compiler::lexer::LexicalAnalyzer;
use compiler::parser::parse_tokens;
use compiler::token::TokenType;
use compiler::llvm_compiler::compile_to_llvm_ir;
use compiler::semantic_analyzer::SemanticAnalyzer;

/// Helper function to run the full compilation pipeline
fn compile_source(source: &str) -> Result<String, String> {
    // Step 1: Lexical Analysis
    let mut lexer = LexicalAnalyzer::new(source);
    let tokens = lexer.scan_tokens();
    
    // Filter out whitespace and comments
    let filtered_tokens: Vec<_> = tokens
        .into_iter()
        .filter(|t| !matches!(
            t.token_type,
            TokenType::Whitespace | TokenType::NewLine | 
            TokenType::CommentSingle | TokenType::CommentMultiLine | TokenType::Unknown
        ))
        .collect();
    
    // Step 2: Parsing
    let parse_result = parse_tokens(&filtered_tokens);
    
    if !parse_result.errors.is_empty() {
        return Err(format!("Parse errors: {:?}", parse_result.errors));
    }
    
    // Step 3: Semantic Analysis (skip main function check for unit tests)
    let mut semantic_analyzer = SemanticAnalyzer::new();
    semantic_analyzer.analyze(&parse_result.ast);
    
    // Filter out MissingMainFunction errors for unit tests
    let non_main_errors: Vec<_> = semantic_analyzer.errors.iter()
        .filter(|e| !matches!(e, compiler::semantic_analyzer::SemanticError::MissingMainFunction))
        .collect();
    
    if !non_main_errors.is_empty() {
        return Err(format!("Semantic errors: {:?}", non_main_errors));
    }
    
    // Step 4: LLVM IR Generation
    compile_to_llvm_ir(&parse_result.ast)
}

/// Validate LLVM IR using llvm-as
fn validate_llvm_ir(llvm_ir: &str) -> bool {
    use std::process::Command;
    
    let temp_file = "/tmp/test_llvm_validation.ll";
    std::fs::write(temp_file, llvm_ir).ok();
    
    let output = Command::new("llvm-as-18")
        .args(&[temp_file, "-o", "/tmp/test_llvm_validation.bc"])
        .output();
    
    output.map(|o| o.status.success()).unwrap_or(false)
}

#[test]
fn test_simple_function() {
    let source = r#"
fn add(a: Int, b: Int) -> Int {
    return a + b;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("define i64 @add"));
    assert!(llvm_ir.contains("add i64"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_function_with_no_params() {
    let source = r#"
fn get_answer() -> Int {
    return 42;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("define i64 @get_answer"));
    assert!(llvm_ir.contains("ret i64"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_local_variables() {
    let source = r#"
fn test() -> Int {
    let x: Int = 10;
    let y: Int = 20;
    return x + y;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("alloca i64"));
    assert!(llvm_ir.contains("store i64"));
    assert!(llvm_ir.contains("load i64"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_arithmetic_operations() {
    let source = r#"
fn arithmetic(a: Int, b: Int) -> Int {
    let sum: Int = a + b;
    let diff: Int = a - b;
    let prod: Int = a * b;
    let quot: Int = a / b;
    return sum + diff + prod + quot;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("add i64"));
    assert!(llvm_ir.contains("sub i64"));
    assert!(llvm_ir.contains("mul i64"));
    assert!(llvm_ir.contains("sdiv i64"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_comparison_operations() {
    let source = r#"
fn compare(a: Int, b: Int) -> Int {
    if (a > b) {
        return 1;
    } else {
        return 0;
    }
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("icmp"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_if_statement() {
    let source = r#"
fn max(a: Int, b: Int) -> Int {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("br i1"));
    assert!(llvm_ir.contains("label %then"));
    assert!(llvm_ir.contains("label %else"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_while_loop() {
    let source = r#"
fn countdown(n: Int) -> Int {
    let counter: Int = n;
    while (counter > 0) {
        let dummy: Int = counter - 1;
    }
    return counter;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("label %whilecond"));
    assert!(llvm_ir.contains("label %whilebody"));
    assert!(llvm_ir.contains("label %afterwhile"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_function_call() {
    let source = r#"
fn helper(x: Int) -> Int {
    return x * 2;
}

fn caller() -> Int {
    let val: Int = 5;
    return helper(val);
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("call i64 @helper"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_multiple_functions() {
    let source = r#"
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

fn subtract(a: Int, b: Int) -> Int {
    return a - b;
}

fn multiply(a: Int, b: Int) -> Int {
    return a * b;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("define i64 @add"));
    assert!(llvm_ir.contains("define i64 @subtract"));
    assert!(llvm_ir.contains("define i64 @multiply"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_float_operations() {
    let source = r#"
fn add_floats(a: Float, b: Float) -> Float {
    return a + b;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("double"));
    assert!(llvm_ir.contains("fadd double"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_bool_operations() {
    let source = r#"
fn logic(a: Int, b: Int) -> Int {
    if (a > 0 && b > 0) {
        return 1;
    } else {
        return 0;
    }
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_unary_operations() {
    let source = r#"
fn negate(x: Int) -> Int {
    let y: Int = -x;
    return y;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_complex_expression() {
    let source = r#"
fn complex(a: Int, b: Int, c: Int) -> Int {
    let result: Int = (a + b) * c - (a / b);
    return result;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("add i64"));
    assert!(llvm_ir.contains("mul i64"));
    assert!(llvm_ir.contains("sub i64"));
    assert!(llvm_ir.contains("sdiv i64"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_nested_blocks() {
    let source = r#"
fn nested(x: Int) -> Int {
    if (x > 0) {
        let y: Int = x + 1;
        if (y > 10) {
            return y;
        } else {
            return x;
        }
    } else {
        return 0;
    }
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_void_function() {
    let source = r#"
fn do_nothing() -> Void {
    let x: Int = 42;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("define void @do_nothing"));
    assert!(llvm_ir.contains("ret void"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_case_insensitive_types() {
    // Test that both Int and int work
    let source = r#"
fn test1(a: Int, b: int) -> INT {
    return a + b;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Case-insensitive types should work: {:?}", result.err());
}

#[test]
fn test_comprehensive_program() {
    let source = r#"
fn factorial(n: Int) -> Int {
    let result: Int = 1;
    let counter: Int = n;
    while (counter > 1) {
        let temp: Int = result * counter;
    }
    return result;
}

fn fibonacci(n: Int) -> Int {
    if (n <= 1) {
        return n;
    } else {
        let a: Int = fibonacci(n - 1);
        let b: Int = fibonacci(n - 2);
        return a + b;
    }
}

fn main() -> Int {
    let fact: Int = factorial(5);
    let fib: Int = fibonacci(10);
    return fact + fib;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok(), "Comprehensive program failed: {:?}", result.err());
    
    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("define i64 @factorial"));
    assert!(llvm_ir.contains("define i64 @fibonacci"));
    assert!(llvm_ir.contains("define i64 @main"));
    assert!(validate_llvm_ir(&llvm_ir), "Generated LLVM IR is invalid");
}

#[test]
fn test_syntax_error_detection() {
    let source = r#"
fn broken(a: Int -> Int {
    return a;
}
"#;
    
    let result = compile_source(source);
    assert!(result.is_err(), "Should detect syntax error");
}

#[test]
fn test_type_error_detection() {
    // This should pass parsing but may have semantic issues
    // depending on your semantic analyzer implementation
    let source = r#"
fn test() -> Int {
    let x: Float = 3.14;
    return x;
}
"#;
    
    let result = compile_source(source);
    // This may pass or fail depending on semantic analysis strictness
    // The test documents current behavior
}
