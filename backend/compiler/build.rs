use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| "src/proto".to_string());

    tonic_build::configure()
        .build_server(true) // Ensure server code is generated
        .file_descriptor_set_path(Path::new(&out_dir).join("lexer_descriptor.bin"))
        .out_dir(&out_dir)
        .compile(&["proto/lexer.proto"], &["proto"])?;
    Ok(())
}