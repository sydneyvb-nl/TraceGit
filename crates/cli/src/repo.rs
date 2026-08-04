//! Repository lifecycle commands: `init` and `doctor` (setup checks plus best
//! effort AI-tool detection).

use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use anyhow::Result;

use tellur_core::storage::{RepoStorage, TraceIndex};

use crate::connect::ensure_repo_git_automation;

pub(crate) async fn cmd_init(profile: &str) -> Result<()> {
    validate_init_profile(profile)?;
    let storage = RepoStorage::discover()?;
    if storage.is_initialized() {
        let hooks_dir = ensure_repo_git_automation(&storage)?;
        println!("Tellur already initialized; Git automation is active.");
        println!("  Hooks: {}", hooks_dir.display());
        return Ok(());
    }

    storage.init()?;
    let hooks_dir = ensure_repo_git_automation(&storage)?;
    println!("✓ Tellur initialized (profile: {})", profile);
    println!("  Config: {}", storage.config_path.display());
    println!("  Policies: {}", storage.policies_dir.display());
    println!("  Traces: {}", storage.traces_dir.display());
    println!("  Git automation: {}", hooks_dir.display());
    println!("  Capture starts automatically through configured agents and editors.");
    Ok(())
}

fn validate_init_profile(profile: &str) -> Result<()> {
    match profile {
        "default" | "team" | "oss-maintainer" => Ok(()),
        other => anyhow::bail!(
            "unsupported init profile `{other}` (expected: default, team, oss-maintainer)"
        ),
    }
}

pub(crate) async fn cmd_doctor() -> Result<()> {
    let storage = RepoStorage::discover()?;

    println!("Tellur Doctor");
    println!("═══════════════");
    println!();

    // Check config
    if storage.is_initialized() {
        println!("✓ Config found");
    } else {
        println!("✗ Config not found — run `tellur init` first");
    }

    // Check policies
    match list_dir_entries_with_extension(&storage.policies_dir, "yml") {
        Ok(policies) => {
            println!(
                "✓ {} polic{} found",
                policies.len(),
                if policies.len() == 1 { "y" } else { "ies" }
            );
            for p in &policies {
                println!(
                    "  - {}",
                    p.file_name().unwrap_or_default().to_string_lossy()
                );
            }
        }
        Err(e) => {
            println!("⚠ Could not inspect policies directory: {e}");
        }
    }

    // Check index
    if storage.index_path.exists() {
        let index = TraceIndex::open(&storage.index_path)?;
        let events = index.event_count()?;
        let sessions = index.session_count()?;
        println!("✓ Index found ({} events, {} sessions)", events, sessions);
    } else {
        println!("⚠ No index yet");
    }

    // Check traces
    if storage.traces_dir.exists() {
        match list_dir_entries_with_extension(&storage.traces_dir, "jsonl") {
            Ok(trace_files) => println!("✓ Traces directory ({} log files)", trace_files.len()),
            Err(e) => println!("⚠ Could not inspect traces directory: {e}"),
        }
    }

    // Detect AI tools
    println!();
    println!("AI Tool Detection:");
    let mut detected = 0;

    // Check for Claude Code
    if std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
        .join(".claude")
        .exists()
    {
        detected += 1;
        println!("  ✓ Claude Code (~/.claude found)");
    }

    // Check for Cursor
    if storage.root.join(".cursor").exists() {
        detected += 1;
        println!("  ✓ Cursor (.cursor/ found)");
    }

    // Check for Aider
    if executable_on_path("aider") {
        detected += 1;
        println!("  ✓ Aider (installed)");
    }

    // Check for Codex CLI
    if std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
        .join(".codex")
        .exists()
        || executable_on_path("codex")
    {
        detected += 1;
        println!("  ✓ Codex CLI (~/.codex or codex binary found)");
    }

    // Check for common Copilot workspace config
    if storage
        .root
        .join(".github")
        .join("copilot-instructions.md")
        .exists()
    {
        detected += 1;
        println!("  ✓ GitHub Copilot instructions (.github/copilot-instructions.md found)");
    }

    if detected == 0 {
        println!("  No AI coding tools detected");
    }

    println!();
    if storage.is_initialized() {
        println!("Setup looks good. Configured agents and editors capture automatically.");
    }

    Ok(())
}

fn list_dir_entries_with_extension(dir: &Path, extension: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == extension) {
            entries.push(path);
        }
    }
    entries.sort();
    Ok(entries)
}

fn executable_on_path(name: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    executable_on_paths(name, &paths)
}

fn executable_on_paths(name: &str, paths: &std::ffi::OsStr) -> bool {
    std::env::split_paths(paths).any(|dir| {
        executable_candidates(name).any(|candidate| {
            let path = dir.join(candidate);
            is_executable_file(&path)
        })
    })
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
        && path
            .metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn executable_candidates(name: &str) -> impl Iterator<Item = String> + '_ {
    #[cfg(windows)]
    {
        let pathext = std::env::var_os("PATHEXT")
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".to_string());
        let mut names = vec![name.to_string()];
        names.extend(
            pathext
                .split(';')
                .filter(|ext| !ext.is_empty())
                .map(move |ext| format!("{name}{ext}")),
        );
        names.into_iter()
    }
    #[cfg(not(windows))]
    {
        std::iter::once(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn executable_detection_requires_execute_bit_on_unix() {
        let dir = std::env::temp_dir().join(format!(
            "tellur-path-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("codex");
        std::fs::write(&file, "#!/bin/sh\nexit 0\n").unwrap();

        assert!(!executable_on_paths("codex", dir.as_os_str()));

        let mut perms = std::fs::metadata(&file).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&file, perms).unwrap();
        assert!(executable_on_paths("codex", dir.as_os_str()));

        let _ = std::fs::remove_dir_all(dir);
    }
}
