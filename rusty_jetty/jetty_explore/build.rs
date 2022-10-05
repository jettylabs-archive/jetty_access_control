use std::path::Path;
use std::process::Command;

fn main() {
    let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let web_project_path = Path::new(&project_dir).join("web/");

    // Note that there are a number of downsides to this approach, the comments
    // below detail how to improve the portability of these commands.
    Command::new("yarn")
        .args(["quasar", "build"])
        .current_dir(web_project_path)
        .status()
        .unwrap();

    println!("cargo:rerun-if-changed=web/src/");
    println!("cargo:rerun-if-changed=web/public/");
    println!("cargo:rerun-if-changed=web/quasar.config.js");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/postcss.config.js");
}
