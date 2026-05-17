use smac_core::assets::AssetPack;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pack = AssetPack::new();
    let data_dir = Path::new("data");

    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().unwrap().to_str().unwrap();
            let data = fs::read(&path)?;
            println!("Packing {}...", name);
            pack.add_file(name, data);
        }
    }

    let bytes = pack.to_bytes()?;
    fs::write("assets.pack", bytes)?;
    println!(
        "Successfully created assets.pack ({} bytes).",
        fs::metadata("assets.pack")?.len()
    );

    Ok(())
}
