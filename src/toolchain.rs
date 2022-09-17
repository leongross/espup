use crate::utils::*;
use anyhow::{bail, Result};
use espflash::Chip;
use std::path::Path;
use std::process::Stdio;

pub fn check_rust_installation(nightly_version: &str) -> Result<()> {
    match std::process::Command::new("rustup")
        .args(["toolchain", "list"])
        .stdout(Stdio::piped())
        .output()
    {
        Ok(child_output) => {
            println!("{} rustup found", INFO);
            let result = String::from_utf8_lossy(&child_output.stdout);
            if !result.contains(nightly_version) {
                println!("nightly toolchain not found");
                install_rust_nightly(nightly_version);
            } else {
                println!("{} {} toolchain found", INFO, nightly_version);
            }
        }
        Err(e) => {
            println!("{}Error: {}", ERROR, e);
            install_rustup()?;
        }
    }
    Ok(())
}

pub fn install_riscv_target(version: &str) {
    match std::process::Command::new("rustup")
        .arg("component")
        .arg("add")
        .arg("rust-src")
        .arg("--toolchain")
        .arg(version)
        .stdout(Stdio::piped())
        .output()
    {
        Ok(child_output) => {
            let result = String::from_utf8_lossy(&child_output.stdout);
            println!(
                "{} Rust-src for RiscV target installed suscesfully: {}",
                SPARKLE, result
            );
        }
        Err(e) => {
            println!(
                "{}  Rust-src for RiscV target installation failed: {}",
                ERROR, e
            );
        }
    }

    match std::process::Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("--toolchain")
        .arg(version)
        .arg("riscv32imc-unknown-none-elf")
        .stdout(Stdio::piped())
        .output()
    {
        Ok(child_output) => {
            let result = String::from_utf8_lossy(&child_output.stdout);
            println!("{} RiscV target installed suscesfully: {}", SPARKLE, result);
        }
        Err(e) => {
            println!("{} RiscV target installation failed: {}", ERROR, e);
        }
    }
}

pub fn install_rustup() -> Result<()> {
    #[cfg(windows)]
    let rustup_init_path = download_file(
        "https://win.rustup.rs/x86_64".to_string(),
        "rustup-init.exe",
        &get_dist_path("rustup"),
        false,
    )
    .unwrap();
    #[cfg(unix)]
    let rustup_init_path = download_file(
        "https://sh.rustup.rs".to_string(),
        "rustup-init.sh",
        &get_dist_path("rustup"),
        false,
    )
    .unwrap();
    println!("{} Installing rustup with nightly toolchain", WRENCH);
    let mut arguments: Vec<String> = [].to_vec();
    arguments.push(rustup_init_path);
    arguments.push("--default-toolchain".to_string());
    arguments.push("nightly".to_string());
    arguments.push("--profile".to_string());
    arguments.push("minimal".to_string());
    arguments.push("-y".to_string());
    run_command("/bin/bash", arguments, "".to_string())?;

    Ok(())
}

pub fn install_rust_nightly(version: &str) {
    println!("{} Installing {} toolchain", WRENCH, version);
    match std::process::Command::new("rustup")
        .arg("toolchain")
        .arg("install")
        .arg(version)
        .arg("--profile")
        .arg("minimal")
        .stdout(Stdio::piped())
        .output()
    {
        Ok(child_output) => {
            let result = String::from_utf8_lossy(&child_output.stdout);
            println!("{} Result: {}", SPARKLE, result);
        }
        Err(e) => {
            println!("{} Error: {}", ERROR, e);
        }
    }
}

pub fn install_extra_crate(crate_name: &str) {
    println!("{} Installing {} crate", WRENCH, crate_name);
    match std::process::Command::new("cargo")
        .arg("install")
        .arg(crate_name)
        .stdout(Stdio::piped())
        .output()
    {
        Ok(child_output) => {
            let result = String::from_utf8_lossy(&child_output.stdout);
            println!(
                "{} Crate {} installed suscesfully: {}",
                SPARKLE, crate_name, result
            );
        }
        Err(e) => {
            println!("{}  Crate {} installation failed: {}", ERROR, crate_name, e);
        }
    }
}

pub fn install_gcc_targets(targets: Vec<Chip>) -> Result<Vec<String>, String> {
    let mut exports: Vec<String> = Vec::new();
    for target in targets {
        match target {
            Chip::Esp32 => {
                install_gcc("xtensa-esp32-elf");
                exports.push(format!(
                    "export PATH={}:$PATH",
                    get_tool_path("xtensa-esp32-elf/bin")
                ));
            }
            Chip::Esp32s2 => {
                install_gcc("xtensa-esp32s2-elf");
                exports.push(format!(
                    "export PATH={}:$PATH",
                    get_tool_path("xtensa-esp32s2-elf/bin")
                ));
            }
            Chip::Esp32s3 => {
                install_gcc("xtensa-esp32s3-elf");
                exports.push(format!(
                    "export PATH={}:$PATH",
                    get_tool_path("xtensa-esp32s3-elf/bin")
                ));
            }
            Chip::Esp32c3 => {
                install_gcc("riscv32-esp-elf");
                exports.push(format!(
                    "export PATH={}:$PATH",
                    get_tool_path("riscv32-esp-elf/bin")
                ));
            }
            _ => {
                println!("{} Unknown target", ERROR)
            }
        }
    }
    Ok(exports)
}

pub fn install_gcc(gcc_target: &str) {
    let gcc_path = get_tool_path(gcc_target);
    // println!("gcc path: {}", gcc_path);
    let gcc_file = format!(
        "{}-gcc8_4_0-esp-2021r2-patch3-{}.tar.gz",
        gcc_target,
        get_gcc_arch(guess_host_triple::guess_host_triple().unwrap())
    );
    let gcc_dist_url = format!(
        "https://github.com/espressif/crosstool-NG/releases/download/esp-2021r2-patch3/{}",
        gcc_file
    );
    match prepare_package_strip_prefix(&gcc_dist_url, gcc_path, "") {
        Ok(_) => {
            println!("{} Package {} ready", SPARKLE, gcc_file);
        }
        Err(_e) => {
            println!("{} Unable to prepare {}", ERROR, gcc_file);
        }
    }
}

pub fn install_espidf(targets: &str, version: &str) -> Result<(), String> {
    let espidf_path = get_espidf_path(version);
    println!("{} ESP-IDF Path: {}", INFO, espidf_path);

    // #[cfg(windows)]
    // match prepare_package(
    //     "https://dl.espressif.com/dl/idf-git/idf-git-2.30.1-win64.zip".to_string(),
    //     get_dist_path("idf-git-2.30.1-win64.zip").as_str(),
    //     get_tool_path("idf-git/2.30.1".to_string()),
    // ) {
    //     Ok(_) => {
    //         println!("Ok");
    //     }
    //     Err(_e) => {
    //         println!("Failed");
    //     }
    // }
    // #[cfg(windows)]
    // match prepare_package(
    //     "https://dl.espressif.com/dl/idf-python/idf-python-3.8.7-embed-win64.zip".to_string(),
    //     get_dist_path("idf-python-3.8.7-embed-win64.zip").as_str(),
    //     get_tool_path("idf-python/3.8.7".to_string()),
    // ) {
    //     Ok(_) => {
    //         println!("Ok");
    //     }
    //     Err(_e) => {
    //         println!("Failed");
    //     }
    // }

    #[cfg(windows)]
    let git_path = get_tool_path("idf-git/2.30.1/cmd/git.exe".to_string());
    #[cfg(unix)]
    let git_path = "/usr/bin/git".to_string();

    // TODO: See if needed
    // update_property("gitPath".to_string(), git_path.clone());

    #[cfg(windows)]
    let python_path = get_tool_path("idf-python/3.8.7/python.exe".to_string());
    #[cfg(unix)]
    let python_path = "/usr/bin/python3".to_string();

    // let virtual_env_path = get_python_env_path("4.4", "3.8");
    // TODO: Use any git crate?
    if !Path::new(&espidf_path).exists() {
        let mut arguments: Vec<String> = [].to_vec();
        arguments.push("clone".to_string());
        arguments.push("--jobs".to_string());
        arguments.push("8".to_string());
        arguments.push("--branch".to_string());
        arguments.push(version.to_string());
        arguments.push("--depth".to_string());
        arguments.push("1".to_string());
        arguments.push("--shallow-submodules".to_string());
        arguments.push("--recursive".to_string());
        arguments.push("https://github.com/espressif/esp-idf.git".to_string());
        arguments.push(espidf_path.clone());
        println!("{} Dowloading esp-idf {}", DOWNLOAD, version);
        match run_command(&git_path, arguments, "".to_string()) {
            Ok(_) => {
                println!("{} Cloned esp-idf suscessfuly", SPARKLE);
            }
            Err(_e) => {
                println!("{} Cloned esp-idf failed", ERROR);
            }
        }
    }
    println!(
        "{} Installing esp-idf for {} with {}/install.sh",
        WRENCH, targets, espidf_path
    );
    let install_script_path = format!("{}/install.sh", espidf_path);
    let mut arguments: Vec<String> = [].to_vec();
    arguments.push(targets.to_string());
    match run_command(&install_script_path, arguments, "".to_string()) {
        Ok(_) => {
            println!("{} ESP-IDF installation succeeded", SPARKLE);
        }
        Err(_e) => {
            println!("{} ESP-IDF installation failed", ERROR);
        }
    }

    println!("{} Installing CMake", WRENCH);
    let mut arguments: Vec<String> = [].to_vec();
    let idf_tools_scritp_path = format!("{}/tools/idf_tools.py", espidf_path);
    arguments.push(idf_tools_scritp_path);
    arguments.push("install".to_string());
    arguments.push("cmake".to_string());
    match run_command(&python_path, arguments, "".to_string()) {
        Ok(_) => {
            println!("{} CMake installation succeeded", SPARKLE);
        }
        Err(_e) => {
            println!("{} CMake installation failed", ERROR);
        }
    }

    Ok(())
}