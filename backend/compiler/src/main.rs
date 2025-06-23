// Módulos que pertenecen al binario
mod grpc_services;
mod reflection;

// Usa el nombre del crate "compiler" para importar la lógica de la biblioteca
// Importa los servicios y los servidores gRPC desde el módulo local `grpc_services`
use crate::grpc_services::{LexerService, ParserService};
use crate::grpc_services::compiler::lexer_server::LexerServer;
use crate::grpc_services::compiler::parser_server::ParserServer;

use crate::reflection::FILE_DESCRIPTOR_SET;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let lexer_service = LexerService::default();
    let parser_service = ParserService::default();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(LexerServer::new(lexer_service))
        .add_service(ParserServer::new(parser_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
