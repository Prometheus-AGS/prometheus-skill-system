use anyhow::Result;
use colored::Colorize;
use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

pub fn run(update: bool) -> Result<()> {
    let lock_path = Path::new("Skills.lock");

    if update {
        println!("{}", "🔒 Updating Skills.lock checksums...".bold());
        update_checksums(lock_path)?;
    } else {
        println!("{}", "🔒 Verifying skill integrity...".bold());
        verify_checksums(lock_path)?;
    }

    Ok(())
}

fn compute_dir_checksum(dir: &Path) -> Result<String> {
    let mut hasher = Sha256::new();

    let mut files: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    files.sort();

    for file in files {
        let relative = file.strip_prefix(dir).unwrap_or(&file);
        hasher.update(relative.to_string_lossy().as_bytes());
        if let Ok(content) = std::fs::read(&file) {
            hasher.update(&content);
        }
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn update_checksums(lock_path: &Path) -> Result<()> {
    let skills_dir = Path::new("skills");
    if !skills_dir.exists() {
        println!("  No skills/ directory found.");
        return Ok(());
    }

    let mut lock_entries = Vec::new();

    for entry in std::fs::read_dir(skills_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let category = entry.path();
            for skill in std::fs::read_dir(&category)? {
                let skill = skill?;
                if skill.file_type()?.is_dir() {
                    let checksum = compute_dir_checksum(&skill.path())?;
                    let name = skill.file_name().to_string_lossy().to_string();
                    lock_entries.push(format!("{}={}", name, checksum));
                }
            }
        }
    }

    lock_entries.sort();
    let content = lock_entries.join("\n") + "\n";
    std::fs::write(lock_path, content)?;
    println!("  {} Updated {} entries", "✅".green(), lock_entries.len());

    Ok(())
}

fn verify_checksums(lock_path: &Path) -> Result<()> {
    if !lock_path.exists() {
        println!("  {} Skills.lock not found. Run with --update to create.", "⚠️".yellow());
        return Ok(());
    }

    let lock_content = std::fs::read_to_string(lock_path)?;
    let mut mismatches = 0;

    for line in lock_content.lines() {
        if let Some((name, expected)) = line.split_once('=') {
            // Find the skill directory
            let found = find_skill_dir(name);
            match found {
                Some(dir) => {
                    let actual = compute_dir_checksum(&dir)?;
                    if actual == expected {
                        println!("  {} {}", "✅".green(), name);
                    } else {
                        println!("  {} {} — checksum mismatch", "❌".red(), name);
                        mismatches += 1;
                    }
                }
                None => {
                    println!("  {} {} — not found", "❌".red(), name);
                    mismatches += 1;
                }
            }
        }
    }

    if mismatches > 0 {
        println!("\n  {} {} integrity check(s) failed", "❌".red(), mismatches);
        std::process::exit(1);
    } else {
        println!("\n  {} All checksums verified", "✅".green());
    }

    Ok(())
}

fn find_skill_dir(name: &str) -> Option<std::path::PathBuf> {
    let skills_dir = Path::new("skills");
    for entry in std::fs::read_dir(skills_dir).ok()? {
        let entry = entry.ok()?;
        let category = entry.path();
        let skill_dir = category.join(name);
        if skill_dir.exists() {
            return Some(skill_dir);
        }
    }
    None
}
