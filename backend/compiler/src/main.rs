mod ast;
mod grpc_services;
mod lexer;
mod parser;
mod token;
mod reflection; // <-- Add this
mod semantic_analyzer;
mod symbol_table;

use crate::grpc_services::{CompilerService, LexerService, ParserService, compiler::compiler_server::CompilerServer, compiler::lexer_server::LexerServer, compiler::parser_server::ParserServer};
use tonic::transport::Server;
use reflection::FILE_DESCRIPTOR_SET;

// Función principal asíncrona que inicia el servidor gRPC.
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?; // Define la dirección y puerto del servidor.
    let lexer_service = LexerService::default(); // Crea una instancia del servicio.
    let parser_service = ParserService::default(); // Crea una instancia del servicio de análisis sintáctico.
    let compiler_service = CompilerService::default(); // Crea una instancia del servicio de compilador.

    // Configura el servicio de reflexión gRPC para permitir la introspección del servicio.
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    // Construye e inicia el servidor gRPC.
    Server::builder()
        .add_service(LexerServer::new(lexer_service)) // Añade el servicio Lexer.
        .add_service(ParserServer::new(parser_service))
        .add_service(CompilerServer::new(compiler_service))
        .add_service(reflection_service) // Añade el servicio de reflexión.
        .serve(addr) // Inicia el servidor en la dirección especificada.
        .await?;

    println!("Server listening on {}", addr); // Mensaje de confirmación.

    Ok(())
}