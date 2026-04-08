//! GSD Statusline Integration Module
//!
//! This module provides utilities to detect and fix GSD statusline integration issues.
//! It ensures glm-plan-usage is properly integrated into the GSD statusline.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// GSD statusline integration marker
const GSD_MARKER: &str = "// GLM Plan Usage integration";
const GLM_PATH_LINE: &str = "const glmPath = path.join(homeDir, '.claude', 'glm-plan-usage', 'glm-plan-usage');";

/// Check and fix GSD statusline integration
///
/// Returns Ok(true) if changes were made, Ok(false) if already up to date
pub fn fix_integration(verbose: bool) -> Result<bool> {
    let statusline_path = get_gsd_statusline_path();

    if !statusline_path.exists() {
        if verbose {
            eprintln!("GSD statusline not found at: {}", statusline_path.display());
        }
        return Err(anyhow::anyhow!(
            "GSD statusline not found. Make sure GSD is installed."
        ));
    }

    let content = fs::read_to_string(&statusline_path)
        .with_context(|| format!("Failed to read {}", statusline_path.display()))?;

    // Check if already integrated
    if content.contains(GSD_MARKER) && content.contains(GLM_PATH_LINE) {
        if verbose {
            eprintln!("GLM integration already present in GSD statusline");
        }

        // Verify the integration is correct
        if verify_integration(&content) {
            return Ok(false); // Already up to date
        } else {
            if verbose {
                eprintln!("Integration found but incomplete, fixing...");
            }
        }
    }

    // Create backup
    let backup_path = PathBuf::from(format!("{}.bak", statusline_path.display()));
    fs::copy(&statusline_path, &backup_path)
        .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;

    if verbose {
        eprintln!("Backup created: {}", backup_path.display());
    }

    // Apply fix
    let new_content = apply_integration(&content);

    fs::write(&statusline_path, &new_content)
        .with_context(|| format!("Failed to write {}", statusline_path.display()))?;

    if verbose {
        eprintln!("GSD statusline updated: {}", statusline_path.display());
    }

    Ok(true)
}

/// Get the path to GSD statusline script
fn get_gsd_statusline_path() -> PathBuf {
    dirs::home_dir()
        .expect("No home directory found")
        .join(".claude")
        .join("hooks")
        .join("gsd-statusline.js")
}

/// Verify if the integration is complete and correct
fn verify_integration(content: &str) -> bool {
    // Check for all required components
    content.contains(GSD_MARKER)
        && content.contains("spawnSync")
        && content.contains("glm-plan-usage")
        && content.contains(".env")
        && content.contains("' │ ' + glmResult.stdout")
}

/// Apply GLM integration to GSD statusline content
fn apply_integration(content: &str) -> String {
    // If already has marker, remove old integration first
    let content = if content.contains(GSD_MARKER) {
        remove_old_integration(content)
    } else {
        content.to_string()
    };

    // Find the position to insert (before "// Output" line or before the final output logic)
    let insert_marker = "// Output";

    if let Some(idx) = content.find(insert_marker) {
        let (before, after) = content.split_at(idx);

        // Find the end of "const dirname = path.basename(dir);" line
        let dirname_marker = "const dirname = path.basename(dir);";
        if after.find(dirname_marker).is_some() {
            let insert_pos = idx + dirname_marker.len();
            let mut result = content.clone();
            result.insert_str(insert_pos, get_integration_code());
            result
        } else {
            // Fallback: insert before "// Output"
            format!("{}{}{}", before, get_integration_code(), after)
        }
    } else {
        // Fallback: append before the closing of the module
        format!("{}\n{}", content, get_integration_code())
    }
}

/// Remove old GLM integration code
fn remove_old_integration(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_integration = false;
    let mut brace_count = 0;

    for line in lines {
        if line.contains(GSD_MARKER) {
            in_integration = true;
            continue;
        }

        if in_integration {
            // Count braces to find the end of the integration block
            brace_count += line.matches('{').count();
            brace_count -= line.matches('}').count();

            // End of integration block
            if brace_count <= 0 && line.trim().starts_with("}") {
                in_integration = false;
                brace_count = 0;
            }
            continue;
        }

        result.push(line);
    }

    result.join("\n")
}

/// Get the GLM integration code to insert
fn get_integration_code() -> &'static str {
    r#"

    // GLM Plan Usage integration
    let glmUsage = '';
    try {
      const { spawnSync } = require('child_process');
      const glmPath = path.join(homeDir, '.claude', 'glm-plan-usage', 'glm-plan-usage');
      const envPath = path.join(homeDir, '.claude', 'glm-plan-usage', '.env');

      if (fs.existsSync(glmPath) && fs.existsSync(envPath)) {
        // Load .env file and merge with process env
        const envContent = fs.readFileSync(envPath, 'utf8');
        const glmEnv = { ...process.env };
        envContent.split('\n').forEach(line => {
          const eqIdx = line.indexOf('=');
          if (eqIdx > 0) {
            const key = line.substring(0, eqIdx).trim();
            const value = line.substring(eqIdx + 1).trim();
            if (key && value) {
              glmEnv[key] = value;
            }
          }
        });

        // Pass same input to glm-plan-usage via stdin
        const glmResult = spawnSync(glmPath, [], {
          input: JSON.stringify(data),
          encoding: 'utf8',
          timeout: 4000,
          env: glmEnv
        });
        if (glmResult.stdout && glmResult.stdout.trim()) {
          // Use same separator as GSD (│) for consistency
          glmUsage = ' │ ' + glmResult.stdout.trim();
        }
      }
    } catch (e) {
      // Silent fail - don't break statusline if glm-plan-usage fails
    }
"#
}

/// Check for common integration issues
#[allow(dead_code)]
pub fn diagnose_issues(verbose: bool) -> Vec<String> {
    let mut issues = Vec::new();

    let statusline_path = get_gsd_statusline_path();
    let env_path = dirs::home_dir()
        .expect("No home directory found")
        .join(".claude")
        .join("glm-plan-usage")
        .join(".env");

    // Check 1: GSD statusline exists
    if !statusline_path.exists() {
        issues.push("GSD statusline not found. Is GSD installed?".to_string());
    } else {
        // Check 2: GLM integration present
        if let Ok(content) = fs::read_to_string(&statusline_path) {
            if !content.contains(GSD_MARKER) {
                issues.push("GLM integration not found in GSD statusline. Run: glm-plan-usage --fix-gsd".to_string());
            }

            // Check 3: Correct separator
            if content.contains(" | ") && !content.contains("' │ ' + glmResult.stdout") {
                issues.push("Inconsistent separator detected. Run: glm-plan-usage --fix-gsd".to_string());
            }
        }
    }

    // Check 4: .env file exists
    if !env_path.exists() {
        issues.push("GLM .env file not found. Run: glm-plan-usage --setup".to_string());
    }

    // Check 5: Binary exists
    let binary_path = dirs::home_dir()
        .expect("No home directory found")
        .join(".claude")
        .join("glm-plan-usage")
        .join("glm-plan-usage");

    if !binary_path.exists() {
        issues.push("GLM binary not found. Run: cargo build --release && cargo install --path .".to_string());
    }

    if verbose && !issues.is_empty() {
        eprintln!("Diagnosed issues:");
        for (i, issue) in issues.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, issue);
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_integration_complete() {
        let content = r#"
            // GLM Plan Usage integration
            let glmUsage = '';
            const { spawnSync } = require('child_process');
            const glmPath = path.join(homeDir, '.claude', 'glm-plan-usage', 'glm-plan-usage');
            const envPath = path.join(homeDir, '.claude', 'glm-plan-usage', '.env');
            const glmResult = spawnSync(glmPath, [], { input: JSON.stringify(data) });
            glmUsage = ' │ ' + glmResult.stdout.trim();
        "#;
        assert!(verify_integration(content));
    }

    #[test]
    fn test_verify_integration_missing_marker() {
        let content = r#"
            const glmPath = path.join(homeDir, 'glm-plan-usage');
        "#;
        assert!(!verify_integration(content));
    }

    #[test]
    fn test_apply_integration_adds_code() {
        let content = r#"// Output
const dirname = path.basename(dir);
if (task) {
    process.stdout.write(model + " | " + task);
}
"#;
        let result = apply_integration(content);
        assert!(result.contains("// GLM Plan Usage integration"));
        assert!(result.contains("glmUsage"));
    }
}
