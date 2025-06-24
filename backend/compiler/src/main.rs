mod ast;
mod grpc_services;
mod lexer;
mod parser;
mod token;
mod reflection; // <-- Add this

use crate::grpc_services::{LexerService, ParserService, compiler::lexer_server::LexerServer, compiler::parser_server::ParserServer};
use crate::lexer::LexicalAnalyzer;
use crate::parser::parse_tokens;
use crate::token::{LexerToken, TokenType};
use tonic::transport::Server;
use reflection::FILE_DESCRIPTOR_SET;

fn create_tokens(source: &str) -> Vec<LexerToken> {
    let mut lexer = LexicalAnalyzer::new(source);
    let tokens = lexer.scan_tokens();

    tokens
        .into_iter()
        .filter(|token| {
            !matches!(
                token.token_type,
                TokenType::Whitespace
                    | TokenType::NewLine
                    | TokenType::CommentSingle
                    | TokenType::CommentMultiLine
            )
        })
        .collect()
}

// Función principal asíncrona que inicia el servidor gRPC.
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?; // Define la dirección y puerto del servidor.
    let lexer_service = LexerService::default(); // Crea una instancia del servicio.
    let parser_service = ParserService::default(); // Crea una instancia del servicio de análisis sintáctico.

    let someCode = r#"
        fn main() -> int{
        do {
                y = (y + 1) * 2 + 1;
                
                while x > 7 {
                    x = 6 + 8 / 9 * 8 / 3;
                    mas = 36 / 7;
                }
            } until y == 5;
        }
        "#;

    let tokens = create_tokens(someCode);

    let result = parse_tokens(&tokens);

    println!("{:#?}", result);

    // Configura el servicio de reflexión gRPC para permitir la introspección del servicio.
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    // Construye e inicia el servidor gRPC.
    Server::builder()
        .add_service(LexerServer::new(lexer_service)) // Añade el servicio Lexer.
        .add_service(ParserServer::new(parser_service))
        .add_service(reflection_service) // Añade el servicio de reflexión.
        .serve(addr) // Inicia el servidor en la dirección especificada.
        .await?;

    println!("Server listening on {}", addr); // Mensaje de confirmación.

    Ok(())
}
