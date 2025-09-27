// Usa el nombre del crate "compiler" para importar la lógica de la biblioteca
use compiler::lexer::LexicalAnalyzer;
use compiler::parser::parse_tokens;
use compiler::token::{LexerToken, TokenType};
use std::fs;
use std::path::Path;

fn run_snapshot_test(file_path: &str) {
    let path = Path::new(file_path);
    let source = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("No se pudo leer el archivo de prueba: {}", file_path));

    let mut lexer = LexicalAnalyzer::new(&source);
    let tokens = lexer.scan_tokens();

    let filtered_tokens: Vec<LexerToken> = tokens
        .into_iter()
        .filter(|t| !matches!(t.token_type, TokenType::Whitespace | TokenType::NewLine | TokenType::CommentSingle | TokenType::CommentMultiLine | TokenType::Unknown))
        .collect();
    
    let result = parse_tokens(&filtered_tokens);

    let ast_string = format!("{:#?}", result.ast);
    
    // El nombre del snapshot se deriva del nombre del archivo
    insta::assert_snapshot!(insta::internals::AutoName, ast_string, file_path);
}

// Creamos una función de prueba explícita para cada archivo de caso.
// Esto es más robusto que usar la macro `glob!`.
#[test]
fn test_simple_let() {
    run_snapshot_test("tests/cases/01_simple_let.dreamc");
}

#[test]
fn test_functions() {
    run_snapshot_test("tests/cases/02_functions.dreamc");
}

#[test]
fn test_syntax_error() {
    run_snapshot_test("tests/cases/03_syntax_error.dreamc");
}

// Puedes añadir más pruebas aquí para otros archivos...
// #[test]
// fn test_functions() {
//     run_snapshot_test("tests/cases/02_functions.dreamc");
// }
