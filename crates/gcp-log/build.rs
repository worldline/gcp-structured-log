use std::{env, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("models.rs");

    let mut path = env::current_dir()?;

    loop {
        let mut candidate = path.clone();
        candidate.push("crates");
        candidate.push("models");
        candidate.push("src");
        candidate.push("models.rs");

        if candidate.try_exists()? {
            let content = fs::read_to_string(&candidate)?;
            fs::write(&dest_path, content).unwrap();

            println!("cargo:rerun-if-changed={}", candidate.to_str().unwrap());

            return Ok(());
        }

        if !path.pop() {
            panic!("Failed to locate models.rs in any parent directory");
        }
    }
}
