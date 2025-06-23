// En archivo: backend/compiler/src/grpc_lexer.rs

use tonic::{Request, Response, Status};
pub mod lexer {
    tonic::include_proto!("lexer"); 
}

use crate::lexer::LexicalAnalyzer; 
use crate::token::TokenType; // Asegúrate que esta importación exista y sea correcta.
use lexer::{lexer_server::{Lexer}, AnalyzeRequest, Token, TokenList};

#[derive(Debug, Default)]
pub struct LexerService {}

// Implementación del trait Lexer para LexerService.
#[tonic::async_trait]
impl Lexer for LexerService {
    // Método asíncrono que maneja las solicitudes de análisis léxico.
    async fn analyze(&self, request: Request<AnalyzeRequest>) -> std::result::Result<Response<TokenList>, Status> {
        println!("Got request: {:?}", request); // Imprime la solicitud recibida (para depuración).
        
        let input_str = request.into_inner().input; // Extrae el string de entrada de la solicitud.
        let mut analyzer = LexicalAnalyzer::new(&input_str); // Crea una nueva instancia del analizador.
        let tokens = analyzer.scan_tokens(); // Escanea los tokens.
        
        // --- CORRECCIÓN APLICADA AQUÍ ---
        // Convierte los LexerToken de Rust al formato Token de Protobuf, 
        // filtrando todos los tipos no deseados.
        let token_list_proto = tokens.into_iter()
            .filter(|t| !matches!(
                t.token_type,
                TokenType::Whitespace 
                | TokenType::NewLine 
                | TokenType::CommentSingle 
                | TokenType::CommentMultiLine
            ))
            .map(|t| Token {
                token_type: t.token_type.to_string(),
                lexeme: t.lexeme,
                line: t.line as u32,
                column: t.column as u32,
            })
            .collect::<Vec<_>>();
        
        // Devuelve la lista de tokens en una respuesta gRPC.
        Ok(Response::new(TokenList { tokens: token_list_proto }))
    }
}
