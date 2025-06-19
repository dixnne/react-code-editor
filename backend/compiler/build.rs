use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| "src/proto".to_string());

    tonic_build::configure()
        .build_server(true)
        .file_descriptor_set_path(Path::new(&out_dir).join("combined_descriptor.bin"))
        .out_dir(&out_dir)
        .compile(&["proto/lexer.proto", "proto/parser.proto"], &["proto"])?;

    Ok(())
}
