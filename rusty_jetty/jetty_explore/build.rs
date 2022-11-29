use std::path::Path;
use std::process::Command;

#[cfg(windows)]
pub const NPM: &'static str = "npm.cmd";
#[cfg(windows)]
pub const NPX: &'static str = "npx.cmd";

#[cfg(not(windows))]
pub const NPM: &str = "npm";
#[cfg(not(windows))]
pub const NPX: &str = "npx";

fn main() {
    let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let web_project_path = Path::new(&project_dir).join("web/");

    let output = Command::new(NPM)
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

    let profile = std::env::var("PROFILE").unwrap();
    let mut build_cmd = vec!["quasar", "build"];
    if profile == "debug" {
        build_cmd.push("--debug");
    }
    // Note that there are a number of downsides to this approach, the comments
    // below detail how to improve the portability of these commands.
    let output = Command::new(NPX)
        .args(build_cmd)
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
