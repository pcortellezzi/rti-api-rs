use std::fs;
use std::io::Result;

const PROTO_PATH: &str = "src/proto";

fn main() -> Result<()> {
    let mut proto_files = vec![];

    for entry in fs::read_dir(PROTO_PATH).unwrap() {
        let entry_path = entry.unwrap().path();
        if entry_path.is_file() && entry_path.extension().is_some_and(|ext| ext == "proto") {
            proto_files.push(entry_path.display().to_string())
        }
    }

    let mut config = prost_build::Config::default();

    config.compile_protos(&proto_files, &[PROTO_PATH])?;
    Ok(())
}