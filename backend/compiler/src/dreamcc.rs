use clap::Parser;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use compiler::lexer::LexicalAnalyzer;
use compiler::parser::parse_tokens;
use compiler::token::TokenType;
use compiler::semantic_analyzer::SemanticAnalyzer;
use compiler::llvm_compiler::compile_to_llvm_ir;

#[derive(Parser)]
#[command(name = "dreamcc")]
#[command(author = "Dream Language Team")]
#[command(version = "1.0")]
#[command(about = "Dream Language Compiler - Compiles .dream files to native executables", long_about = None)]
struct Cli {
    /// Input source file (.dream)
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// Output file name
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Output LLVM IR instead of executable
    #[arg(long = "emit-llvm")]
    emit_llvm: bool,

    /// Output assembly instead of executable
    #[arg(short = 'S', long = "emit-asm")]
    emit_asm: bool,

    /// Keep intermediate files
    #[arg(short = 'k', long = "keep-temps")]
    keep_temps: bool,

    /// Optimization level (0-3)
    #[arg(short = 'O', default_value = "0")]
    opt_level: u8,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Only run lexer (tokenization)
    #[arg(long = "lex-only")]
    lex_only: bool,

    /// Only run parser (syntax analysis)
    #[arg(long = "parse-only")]
    parse_only: bool,

    /// Only run semantic analyzer
    #[arg(long = "semantic-only")]
    semantic_only: bool,
}

struct CompilationContext {
    input_path: PathBuf,
    output_path: PathBuf,
    temp_dir: PathBuf,
    llvm_ir_path: PathBuf,
    bc_path: PathBuf,
    asm_path: PathBuf,
    obj_path: PathBuf,
}

impl CompilationContext {
    fn new(input: &Path, output: Option<&Path>) -> Self {
        let input_stem = input.file_stem().unwrap().to_str().unwrap();
        let temp_dir = PathBuf::from(format!("/tmp/dreamcc_{}", input_stem));
        
        let output_path = output
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(input_stem));

        CompilationContext {
            input_path: input.to_path_buf(),
            output_path,
            llvm_ir_path: temp_dir.join(format!("{}.ll", input_stem)),
            bc_path: temp_dir.join(format!("{}.bc", input_stem)),
            asm_path: temp_dir.join(format!("{}.s", input_stem)),
            obj_path: temp_dir.join(format!("{}.o", input_stem)),
            temp_dir,
        }
    }

    fn setup(&self) -> Result<(), String> {
        fs::create_dir_all(&self.temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))
    }

    fn cleanup(&self) {
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

fn print_stage(stage: &str, verbose: bool) {
    if verbose {
        println!("{} {}", "➤".green().bold(), stage.bold());
    }
}

fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg.red());
}

fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg.green());
}

fn compile(cli: Cli) -> Result<(), String> {
    // Read source file
    let source = fs::read_to_string(&cli.input)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    print_stage(&format!("Reading {}", cli.input.display()), cli.verbose);

    // Stage 1: Lexical Analysis
    print_stage("Lexical Analysis", cli.verbose);
    let mut lexer = LexicalAnalyzer::new(&source);
    let tokens = lexer.scan_tokens();

    if cli.verbose {
        let token_count = tokens.iter()
            .filter(|t| !matches!(t.token_type, 
                TokenType::Whitespace | TokenType::NewLine | 
                TokenType::CommentSingle | TokenType::CommentMultiLine))
            .count();
        println!("  {} tokens found", token_count);
    }

    if cli.lex_only {
        for token in tokens.iter().filter(|t| !matches!(t.token_type, 
            TokenType::Whitespace | TokenType::NewLine)) {
            println!("{:?} '{}'", token.token_type, token.lexeme);
        }
        return Ok(());
    }

    let filtered_tokens: Vec<_> = tokens
        .into_iter()
        .filter(|t| !matches!(
            t.token_type,
            TokenType::Whitespace | TokenType::NewLine | 
            TokenType::CommentSingle | TokenType::CommentMultiLine | TokenType::Unknown
        ))
        .collect();

    // Stage 2: Parsing
    print_stage("Parsing", cli.verbose);
    let parse_result = parse_tokens(&filtered_tokens);

    if !parse_result.errors.is_empty() {
        print_error("Syntax errors found:");
        for error in &parse_result.errors {
            eprintln!("  {:?}", error);
        }
        return Err("Compilation failed due to syntax errors".to_string());
    }

    if cli.verbose {
        println!("  {} declarations parsed", parse_result.ast.declarations.len());
    }

    if cli.parse_only {
        println!("{:#?}", parse_result.ast);
        return Ok(());
    }

    // Stage 3: Semantic Analysis
    print_stage("Semantic Analysis", cli.verbose);
    let mut semantic_analyzer = SemanticAnalyzer::new();
    semantic_analyzer.analyze(&parse_result.ast);

    if !semantic_analyzer.errors.is_empty() {
        print_error("Semantic errors found:");
        for error in &semantic_analyzer.errors {
            eprintln!("  {:?}", error);
        }
        return Err("Compilation failed due to semantic errors".to_string());
    }

    if cli.verbose {
        println!("  {} symbols in table", 
            semantic_analyzer.symbol_table.current_scope.symbols.len());
    }

    if cli.semantic_only {
        println!("Semantic analysis passed!");
        println!("Symbol table: {:#?}", semantic_analyzer.symbol_table);
        return Ok(());
    }

    // Setup compilation context
    let ctx = CompilationContext::new(&cli.input, cli.output.as_deref());
    ctx.setup()?;

    // Stage 4: LLVM IR Generation
    print_stage("LLVM IR Generation", cli.verbose);
    let llvm_ir = compile_to_llvm_ir(&parse_result.ast)?;

    // Write LLVM IR
    fs::write(&ctx.llvm_ir_path, &llvm_ir)
        .map_err(|e| format!("Failed to write LLVM IR: {}", e))?;

    if cli.verbose {
        println!("  LLVM IR written to {}", ctx.llvm_ir_path.display());
    }

    if cli.emit_llvm {
        let output = cli.output.as_ref()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                let mut p = ctx.input_path.clone();
                p.set_extension("ll");
                p
            });
        fs::copy(&ctx.llvm_ir_path, &output)
            .map_err(|e| format!("Failed to copy LLVM IR: {}", e))?;
        print_success(&format!("LLVM IR written to {}", output.display()));
        if !cli.keep_temps {
            ctx.cleanup();
        }
        return Ok(());
    }

    // Stage 5: LLVM Assembly (validation)
    print_stage("Assembling LLVM IR", cli.verbose);
    let status = Command::new("llvm-as-18")
        .args(&[
            ctx.llvm_ir_path.to_str().unwrap(),
            "-o",
            ctx.bc_path.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format!("Failed to run llvm-as: {}", e))?;

    if !status.success() {
        return Err("LLVM assembly failed - invalid IR generated".to_string());
    }

    // Stage 6: Optimization
    if cli.opt_level > 0 {
        print_stage(&format!("Optimizing (O{})", cli.opt_level), cli.verbose);
        let opt_level = format!("-O{}", cli.opt_level);
        let opt_bc_path = ctx.temp_dir.join("optimized.bc");
        
        let status = Command::new("opt-18")
            .args(&[
                &opt_level,
                ctx.bc_path.to_str().unwrap(),
                "-o",
                opt_bc_path.to_str().unwrap(),
            ])
            .status()
            .map_err(|e| format!("Failed to run opt: {}", e))?;

        if !status.success() {
            return Err("Optimization failed".to_string());
        }

        // Replace unoptimized bytecode with optimized
        fs::copy(&opt_bc_path, &ctx.bc_path)
            .map_err(|e| format!("Failed to copy optimized bytecode: {}", e))?;
    }

    // Stage 7: Assembly Generation
    print_stage("Generating Assembly", cli.verbose);
    let status = Command::new("llc-18")
        .args(&[
            ctx.bc_path.to_str().unwrap(),
            "-o",
            ctx.asm_path.to_str().unwrap(),
            "-filetype=asm",
        ])
        .status()
        .map_err(|e| format!("Failed to run llc: {}", e))?;

    if !status.success() {
        return Err("Assembly generation failed".to_string());
    }

    if cli.emit_asm {
        let output = cli.output.as_ref()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                let mut p = ctx.input_path.clone();
                p.set_extension("s");
                p
            });
        fs::copy(&ctx.asm_path, &output)
            .map_err(|e| format!("Failed to copy assembly: {}", e))?;
        print_success(&format!("Assembly written to {}", output.display()));
        if !cli.keep_temps {
            ctx.cleanup();
        }
        return Ok(());
    }

    // Stage 8: Object File Generation
    print_stage("Generating Object File", cli.verbose);
    let status = Command::new("llc-18")
        .args(&[
            ctx.bc_path.to_str().unwrap(),
            "-o",
            ctx.obj_path.to_str().unwrap(),
            "-filetype=obj",
        ])
        .status()
        .map_err(|e| format!("Failed to generate object file: {}", e))?;

    if !status.success() {
        return Err("Object file generation failed".to_string());
    }

    // Stage 9: Linking
    print_stage("Linking", cli.verbose);
    let status = Command::new("gcc")
        .args(&[
            ctx.obj_path.to_str().unwrap(),
            "-o",
            ctx.output_path.to_str().unwrap(),
            "-no-pie",  // Simpler linking
        ])
        .status()
        .map_err(|e| format!("Failed to link: {}", e))?;

    if !status.success() {
        return Err("Linking failed".to_string());
    }

    print_success(&format!("Executable created: {}", ctx.output_path.display()));

    // Cleanup temp files unless requested to keep
    if !cli.keep_temps {
        ctx.cleanup();
        if cli.verbose {
            println!("  Cleaned up temporary files");
        }
    } else if cli.verbose {
        println!("  Temporary files kept in {}", ctx.temp_dir.display());
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match compile(cli) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            print_error(&e);
            std::process::exit(1);
        }
    }
}
