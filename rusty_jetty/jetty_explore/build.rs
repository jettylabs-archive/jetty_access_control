use std::path::Path;
use std::process::Command;

fn main() {
    let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let web_project_path = Path::new(&project_dir).join("web/");

    let output = Command::new("npm")
        .args(["install"])
        .current_dir(web_project_path.clone())
        .output()
        .unwrap();

    // If this fails, make sure you have npm installed
    assert!(
        output.status.success(),
        "Command failed:\n{}",
        std::str::from_utf8(&output.stdout).unwrap()
    );

    // Note that there are a number of downsides to this approach, the comments
    // below detail how to improve the portability of these commands.
    let output = Command::new("npx")
        .args(["quasar", "build"])
        .current_dir(web_project_path)
        .output()
        .unwrap();

    // If this fails, make sure you have the quasar cli installed
    assert!(
        output.status.success(),
        "Command failed:\n{}",
        std::str::from_utf8(&output.stdout).unwrap()
    );

    println!("cargo:rerun-if-changed=web/src/");
    println!("cargo:rerun-if-changed=web/public/");
    println!("cargo:rerun-if-changed=web/quasar.config.js");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/postcss.config.js");
}