fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Genera el código de Rust para los archivos .proto
    // y lo coloca en el directorio de salida estándar de Cargo (`OUT_DIR`).
    tonic_build::configure()
        .build_server(true)
        // La siguiente línea es opcional pero útil para herramientas de reflexión como grpcurl
        .file_descriptor_set_path(
            std::env::var("OUT_DIR").unwrap() + "/compiler_descriptor.bin",
        )
        .compile(
            &["proto/lexer.proto", "proto/parser.proto"], // Lista de archivos a compilar
            &["proto"], // Directorio donde buscar imports
        )?;

    Ok(())
}
