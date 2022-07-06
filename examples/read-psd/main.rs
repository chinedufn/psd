use std::env;
use std::fs::read;
use std::path::PathBuf;

use psd::Psd;

use serde_json;

fn main() -> Result<(), String> {
    for argument in env::args_os().skip(1) {
        let path = PathBuf::from(argument);

        let mut objdir = PathBuf::from("/home/lczaplinski/Sync/Photoshop/psd-rs/");
        let file_name = path.file_name().ok_or("Not a file name")?;
        objdir.push(file_name);
        let bytes = read(&path).map_err(|err| format!("error opening file: {err}"))?;

        let psd = Psd::from_bytes(&bytes).unwrap();
        println!("Have {} layers", psd.layers().len());

        std::fs::create_dir_all(objdir.as_path()).unwrap();

        objdir.push("generator.json");

        std::fs::write(
            objdir.as_path(),
            serde_json::to_string_pretty(&psd).unwrap(),
        )
        .unwrap();

        println!("written {}", objdir.display());
    }

    Ok(())
}
