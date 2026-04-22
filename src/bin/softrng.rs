use anyhow::{anyhow, bail, Context, Result};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

// Embedded assets
const DEFAULT_MANUAL: &str = include_str!("../../assets/manual.txt");
const MODULE_RNG_TEST: &str = include_str!("../../assets/modules/RNG_test");
const MODULE_RNG_OUTPUT: &str = include_str!("../../assets/modules/RNG_output");
const MODULE_DIEHARDER: &str = include_str!("../../assets/modules/dieharder");
const MODULE_SOFTRNG: &str = include_str!("../../assets/modules/softrng");

// Possible config root directories (priority order)
const USER_CONFIG_ROOT: &str = ".etc/softrng";   // relative to $HOME
const SYSTEM_CONFIG_ROOT: &str = "/etc/softrng";

// Subdirectories
const DIR_MODULES: &str = "modules";
const DIR_HELP: &str = "help";

// Shortcut directories
const SYSTEM_BIN_DIR: &str = "/usr/local/bin";
const USER_BIN_DIR: &str = ".local/bin";         // relative to $HOME

/// Determine the active configuration root directory.
/// Returns (path, is_system_install) where is_system_install indicates
/// whether shortcuts should go to /usr/local/bin (true) or ~/.local/bin (false).
fn determine_config_root() -> Result<(PathBuf, bool)> {
    let home = env::var("HOME").context("HOME environment variable not set")?;
    let user_root = PathBuf::from(&home).join(USER_CONFIG_ROOT);
    let system_root = PathBuf::from(SYSTEM_CONFIG_ROOT);

    // 1. Use existing directory if found
    if user_root.exists() {
        return Ok((user_root, false));
    }
    if system_root.exists() {
        return Ok((system_root, true));
    }

    // 2. None exist: try to create system-wide directory
    match fs::create_dir_all(&system_root) {
        Ok(()) => {
            eprintln!("Created system-wide config at {}", system_root.display());
            Ok((system_root, true))
        }
        Err(e) => {
            eprintln!("Cannot create {}: {}", system_root.display(), e);
            eprintln!("Falling back to user installation at {}", user_root.display());
            fs::create_dir_all(&user_root)
                .with_context(|| format!("Failed to create user config at {}", user_root.display()))?;
            Ok((user_root, false))
        }
    }
}

/// Get the directory where shortcuts should be created.
fn shortcut_dir(is_system: bool) -> Result<PathBuf> {
    if is_system {
        Ok(PathBuf::from(SYSTEM_BIN_DIR))
    } else {
        let home = env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(&home).join(USER_BIN_DIR))
    }
}

/// Ensure that the shortcut directory exists and is in PATH (warn for user install).
fn ensure_shortcut_dir(is_system: bool) -> Result<()> {
    let dir = shortcut_dir(is_system)?;
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create shortcut directory {}", dir.display()))?;
    }
    if !is_system {
        // Warn if ~/.local/bin is not in PATH
        let path_var = env::var("PATH").unwrap_or_default();
        if !path_var.split(':').any(|p| p == dir.to_str().unwrap_or("")) {
            eprintln!(
                "WARNING: {} is not in your PATH. Add it to your shell profile to use shortcuts.",
                dir.display()
            );
        }
    }
    Ok(())
}

/// Check if a file exists.
fn file_exists(path: &Path) -> bool {
    path.exists()
}

/// Check if a directory exists.
fn dir_exists(path: &Path) -> bool {
    path.is_dir()
}

/// Check if a command exists in PATH.
fn cmd_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Open the manual with `less`.
fn show_manual(config_root: &Path) -> Result<()> {
    let manual_path = config_root.join("manual.txt");
    if !manual_path.exists() {
        bail!("Manual not found at {}", manual_path.display());
    }
    Command::new("less")
        .arg(manual_path)
        .status()
        .context("Failed to run 'less'")?;
    Ok(())
}

/// Create a shell script shortcut.
fn create_shortcut(shortcut_dir: &Path, alias_name: &str, alias_command: &str) -> Result<()> {
    let shortcut_path = shortcut_dir.join(alias_name);
    let script = format!("#!/bin/sh\n{} $@\n", alias_command);

    fs::write(&shortcut_path, script)
        .with_context(|| format!("Failed to write shortcut '{}'", shortcut_path.display()))?;

    let mut perms = fs::metadata(&shortcut_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&shortcut_path, perms)
        .with_context(|| format!("Failed to chmod +x '{}'", shortcut_path.display()))?;

    Ok(())
}

/// Remove a shortcut.
fn remove_shortcut(shortcut_dir: &Path, alias_name: &str) -> Result<()> {
    let shortcut_path = shortcut_dir.join(alias_name);
    if shortcut_path.exists() {
        fs::remove_file(&shortcut_path)
            .with_context(|| format!("Failed to remove '{}'", shortcut_path.display()))?;
    }
    Ok(())
}

/// Remove all config files and the config root.
fn remove_config(config_root: &Path) -> Result<()> {
    if config_root.exists() {
        fs::remove_dir_all(config_root)?;
    }
    Ok(())
}

/// Process one module definition file.
fn scan_one_db(config_root: &Path, shortcut_dir: &Path, target_cmd: &str, db_file: &Path) -> Result<()> {
    let content = fs::read_to_string(db_file)
        .with_context(|| format!("Failed to read module file '{}'", db_file.display()))?;

    let cmd_exists = cmd_exists(target_cmd);
    if !cmd_exists {
        print!("Cleaning {} ", target_cmd);
    } else {
        print!("Adding {} ", target_cmd);
    }
    std::io::stdout().flush().unwrap();

    let mut errors = 0;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let short_name = parts[0];
        let short_cmd = parts[1..].join(" ");

        if !cmd_exists {
            if let Err(e) = remove_shortcut(shortcut_dir, short_name) {
                eprint!("! ({}: {})", short_name, e);
                errors += 1;
            } else {
                print!(".");
            }
        } else {
            if let Err(e) = create_shortcut(shortcut_dir, short_name, &short_cmd) {
                eprint!("! ({}: {})", short_name, e);
                errors += 1;
            } else {
                print!(".");
            }
        }
        std::io::stdout().flush().unwrap();
    }
    println!();
    if errors > 0 {
        eprintln!("{} errors occurred.", errors);
    }
    Ok(())
}

/// Delete shortcuts defined in a module file (used during uninstall).
fn delete_one_db(shortcut_dir: &Path, target_cmd: &str, db_file: &Path) -> Result<()> {
    print!("Uninstall shortcuts for {} ", target_cmd);
    std::io::stdout().flush().unwrap();

    let content = fs::read_to_string(db_file)
        .with_context(|| format!("Failed to read module file '{}'", db_file.display()))?;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let short_name = parts[0];
        if let Err(e) = remove_shortcut(shortcut_dir, short_name) {
            eprint!("! ({}: {})", short_name, e);
        } else {
            print!(".");
        }
        std::io::stdout().flush().unwrap();
    }
    println!();
    Ok(())
}

/// Refresh all module shortcuts.
fn refresh(config_root: &Path, shortcut_dir: &Path) -> Result<()> {
    let modules_dir = config_root.join(DIR_MODULES);
    if !modules_dir.exists() {
        println!("No modules directory found. Nothing to refresh.");
        return Ok(());
    }
    for entry in fs::read_dir(&modules_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename.starts_with('.') {
                    continue;
                }
                scan_one_db(config_root, shortcut_dir, filename, &path)?;
            }
        }
    }
    Ok(())
}

/// Uninstall: remove all shortcuts and config files.
fn uninstall(config_root: &Path, shortcut_dir: &Path) -> Result<()> {
    println!("Uninstall");
    let modules_dir = config_root.join(DIR_MODULES);
    if modules_dir.exists() {
        for entry in fs::read_dir(&modules_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    if filename.starts_with('.') {
                        continue;
                    }
                    delete_one_db(shortcut_dir, filename, &path)?;
                }
            }
        }
    }
    remove_config(config_root)?;
    Ok(())
}

/// Write default config files (embedded assets) to the config root.
fn write_default_configs(config_root: &Path) -> Result<()> {
    fs::create_dir_all(config_root)?;
    fs::create_dir_all(config_root.join(DIR_MODULES))?;
    fs::create_dir_all(config_root.join(DIR_HELP))?;

    // Write manual
    let manual_path = config_root.join("manual.txt");
    if !manual_path.exists() {
        fs::write(&manual_path, DEFAULT_MANUAL)?;
    }

    // Write module definition files
    let modules = [
        ("RNG_test.txt", MODULE_RNG_TEST),
        ("RNG_output.txt", MODULE_RNG_OUTPUT),
        ("dieharder.txt", MODULE_DIEHARDER),
        ("softrng.txt", MODULE_SOFTRNG),
    ];
    for (name, content) in modules {
        let path = config_root.join(DIR_MODULES).join(name);
        if !path.exists() {
            fs::write(&path, content)?;
        }
    }
    Ok(())
}

/// Open the configuration directory (platform-specific).
fn open_config_dir(config_root: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(config_root).status()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(config_root).status()?;
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        eprintln!("Opening directory not supported on this OS.");
    }
    Ok(())
}

fn print_usage() {
    println!("\nSoftRNG\n\nusage:");
    println!("  softrng manual    - Read the manual.");
    println!("  softrng install   - Refresh shortcuts.");
    println!("  softrng uninstall - Remove all shortcuts.");
    println!("  softrng open      - Open the configuration folder.");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    for arg in &args[1..] {
        match arg.as_str() {
            "open" => {
                let (config_root, _) = determine_config_root()?;
                open_config_dir(&config_root)?;
            }
            "install" => {
                let (config_root, is_system) = determine_config_root()?;
                write_default_configs(&config_root)?;
                ensure_shortcut_dir(is_system)?;
                let shortcut_dir = shortcut_dir(is_system)?;
                refresh(&config_root, &shortcut_dir)?;
            }
            "uninstall" => {
                let (config_root, is_system) = determine_config_root()?;
                let shortcut_dir = shortcut_dir(is_system)?;
                uninstall(&config_root, &shortcut_dir)?;
            }
            "manual" => {
                let (config_root, _) = determine_config_root()?;
                show_manual(&config_root)?;
            }
            _ => {
                eprintln!("Unknown command: {}", arg);
                print_usage();
                bail!("Invalid command");
            }
        }
    }
    Ok(())
}