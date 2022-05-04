use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::path::Path;

// https://doc.rust-lang.org/cargo/reference/build-scripts.html
fn main() -> Result<()> {
    // This tells cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=res/*");

    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("res/config/cfg.ron");
    paths_to_copy.push("res/config/materials.ron");
    paths_to_copy.push("res/config/meshes.ron");
    paths_to_copy.push("res/meshes");
    paths_to_copy.push("res/textures");
    copy_items(&paths_to_copy, out_dir.clone(), &copy_options)?;

    let conf_path = String::from(
        Path::new(&out_dir)
            .join("res/config/cfg.ron")
            .to_owned()
            .to_str()
            .unwrap_or(""),
    );

    println!("cargo:rustc-env=APP_CONF_FILE_PATH={}", conf_path);

    Ok(())
}
