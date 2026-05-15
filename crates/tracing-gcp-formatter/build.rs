use std::{env, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("models.rs");

    let content = fs::read_to_string("../models/src/models.rs")?;
    fs::write(&dest_path, content).unwrap();

    println!("cargo:rerun-if-changed=../models/src/models.rs");

    Ok(())
}
