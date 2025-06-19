mod ast;
mod grpc_lexer;
mod grpc_parser;
mod lexer;
mod parser;
mod token;
mod reflection; // <-- Add this

use crate::grpc_lexer::lexer::lexer_server::LexerServer;
use crate::grpc_lexer::{LexerService};
use crate::grpc_parser::ParserService;
use crate::lexer::LexicalAnalyzer;
use crate::parser::parse_tokens;
use crate::token::{LexerToken, TokenType};
use tonic::transport::Server;
use crate::grpc_parser::parser::parser_server::ParserServer;
use reflection::FILE_DESCRIPTOR_SET;
pub mod lexer_proto {
    tonic::include_proto!("lexer"); // Incluye el código Rust generado desde lexer.proto.
    
    // Constante que contiene el descriptor del archivo .proto, usado para reflexión del servicio gRPC.
    pub (crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("lexer_descriptor");
}

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
        fn main() {
            let x = 10; // Variable de tipo entero
            let y = 20.5; // Variable de tipo flotante
            let z = "Hola, mundo"; // Variable de tipo cadena

            if x < y { // Comparación entre entero y flotante convertido a entero
                println("{}", z); // Imprime la cadena
            }
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
