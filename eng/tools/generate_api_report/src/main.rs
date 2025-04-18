// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

use rustdoc_types::Crate;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    // Verify we're running from the repository root
    if !Path::new("eng/tools/generate_api_report").exists() {
        return Err(
            "This tool must be run from the root of the azure-sdk-for-rust repository. Use: cargo run --manifest-path eng/tools/generate_api_report/Cargo.toml -- --package {package_name}".into(),
        );
    }

    // Get the package name from command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 || args[1] != "--package" {
        eprintln!("Usage: {} --package <package_name>", args[0]);
        std::process::exit(1);
    }
    let package_name = &args[2];
    let path_str = format!("./target/doc/{}.json", package_name);
    let path = Path::new(&path_str);

    // Call cargo +nightly rustdoc to generate the JSON file
    let channel = env!("TOOLCHAIN_CHANNEL");
    let mut command = Command::new("cargo");
    command
        .arg(format!("+{channel}"))
        .arg("rustdoc")
        .arg("-Z")
        .arg("unstable-options")
        .arg("--output-format")
        .arg("json")
        .arg("--package")
        .arg(package_name)
        .arg("--all-features");
    println!(
        "Running: {} {}",
        command.get_program().to_string_lossy(),
        command
            .get_args()
            .collect::<Vec<&OsStr>>()
            .join(OsStr::new(" "))
            .to_string_lossy(),
    );
    let output = command.output()?;

    if !output.status.success() {
        eprintln!(
            "Failed to generate JSON file: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::process::exit(1);
    }

    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    println!("Processing rustdoc output for package: {}", package_name);

    let mut root: Crate = serde_json::from_str(&contents)?;

    // Remove items
    // 1. with item.inner.impl.is_synthetic set to true - [auto traits]
    // 2. with item.inner.impl.blanket_impl is not null - [blanket impls]
    root.index.retain(|_id, item| {
        if let rustdoc_types::ItemEnum::Impl(impl_item) = &item.inner {
            // Filter out auto traits
            if impl_item.is_synthetic {
                return false;
            }
            // Filter out blanket implementations
            if impl_item.blanket_impl.is_some() {
                return false;
            }
        }
        true
    });

    // Clear unnecessary fields in the Crate structure
    // 1. external_crates
    // 2. span in all items
    root.external_crates.clear();
    for (_id, item) in root.index.iter_mut() {
        // Reset span to default empty value
        item.span = Default::default();
    }

    // Navigate to Cargo.toml and get the path for the package
    let cargo_toml_path = Path::new("Cargo.toml");
    let cargo_toml_content = std::fs::read_to_string(cargo_toml_path)?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)?;

    let package_path = cargo_toml
        .get("workspace")
        .and_then(|ws| ws.get("members"))
        .and_then(|members| members.as_array())
        .and_then(|members| {
            members.iter().find_map(|member| {
                if member.as_str()?.ends_with(package_name) {
                    Some(member.as_str()?.to_string())
                } else {
                    None
                }
            })
        })
        .ok_or("Package path not found in Cargo.toml")?;

    // Create the review/ folder under the obtained path if it doesn't exist
    let review_folder_path = Path::new(&package_path).join("review");
    if !review_folder_path.exists() {
        std::fs::create_dir_all(&review_folder_path)?;
    }

    // Create the package_name.rust.json in the review/ folder
    let output_path_str = review_folder_path.join(format!("{}.rust.json", package_name));
    let output_path = Path::new(&output_path_str);
    let mut output_file = File::create(output_path)?;
    serde_json::to_writer(&mut output_file, &root)?;

    println!("File has been generated at: {}", output_path_str.display());

    Ok(())
}
