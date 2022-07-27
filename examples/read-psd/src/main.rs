use std::env;
use std::fs::read;
use std::path::PathBuf;

use psd::Psd;

use image;
use serde_json;

fn main() -> Result<(), String> {
        for argument in env::args_os().skip(1) {
                let path = PathBuf::from(argument);

                let mut objdir = PathBuf::from("/tmp/");
                let file_name = path.file_name().ok_or("Not a file name")?;
                objdir.push(file_name);
                let bytes = read(&path).map_err(|err| format!("error opening file: {err}"))?;

                let psd = Psd::from_bytes(&bytes).unwrap();
                println!("Have {} layers", psd.layers().len());

                std::fs::create_dir_all(objdir.as_path()).unwrap();

                let mut generator = objdir.clone();

                generator.push("layout.json");

                std::fs::write(
                        generator.as_path(),
                        serde_json::to_string_pretty(&psd).unwrap(),
                )
                .unwrap();

                let mut rgba = objdir.clone();
                rgba.push("preview.png");

                let pixels = psd.rgba();
                image::save_buffer(
                        rgba.as_path(),
                        &pixels,
                        psd.width(),
                        psd.height(),
                        image::ColorType::Rgba8,
                )
                .unwrap();

                let mut layers = objdir.clone();
                layers.push("layers");
                std::fs::create_dir_all(layers.as_path()).unwrap();

                for layer in psd.layers() {
                        println!("layer: {}", layer.name());
                        let mut layer_file = layers.clone();
                        layer_file.push(layer.name());
                        layer_file.set_extension("png");

                        let pixels = layer.rgba();
                        image::save_buffer(
                                layer_file.as_path(),
                                &pixels,
                                psd.width(),
                                psd.height(),
                                image::ColorType::Rgba8,
                        )
                        .unwrap();
                }

                println!("written to {}", objdir.display());
        }

        Ok(())
}
