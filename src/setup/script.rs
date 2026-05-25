//! Combined statusline script templates and file writing.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Write the combined statusline script to disk.
pub fn write_combined_script(path: &Path) -> Result<()> {
    let content = combined_script_content();

    fs::write(path, content)
        .with_context(|| format!("Failed to write combined script to {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(path, perms)
            .with_context(|| format!("Failed to set permissions on {}", path.display()))?;
    };

    Ok(())
}

#[cfg(not(windows))]
fn combined_script_content() -> &'static str {
    r#"#!/bin/bash

# Combined statusline script for glm-plan-usage + CCometixLine
INPUT=$(cat)

CCLINE_OUTPUT=$(echo "$INPUT" | ~/.claude/ccline/ccline 2>/dev/null)
GLM_OUTPUT=$(echo "$INPUT" | ~/.claude/glm-plan-usage/glm-plan-usage 2>/dev/null)

OUTPUT=""

if [ -n "$CCLINE_OUTPUT" ]; then
    OUTPUT="$CCLINE_OUTPUT"
fi

if [ -n "$GLM_OUTPUT" ]; then
    if [ -n "$OUTPUT" ]; then
        OUTPUT="$OUTPUT | $GLM_OUTPUT"
    else
        OUTPUT="$GLM_OUTPUT"
    fi
fi

if [ -n "$OUTPUT" ]; then
    printf "%s" "$OUTPUT"
fi
"#
}

#[cfg(windows)]
fn combined_script_content() -> &'static str {
    r#"# Combined statusline script for glm-plan-usage + CCometixLine
$InputString = [Console]::In.ReadToEnd()

$CclineOutput = $InputString | & "$env:USERPROFILE\.claude\ccline\ccline.exe" 2>$null
$GlmOutput = $InputString | & "$env:USERPROFILE\.claude\glm-plan-usage\glm-plan-usage.exe" 2>$null

$Output = ""

if (-not [string]::IsNullOrEmpty($CclineOutput)) {
    $Output = $CclineOutput
}

if (-not [string]::IsNullOrEmpty($GlmOutput)) {
    if (-not [string]::IsNullOrEmpty($Output)) {
        $Output = "$Output | $GlmOutput"
    } else {
        $Output = $GlmOutput
    }
}

if (-not [string]::IsNullOrEmpty($Output)) {
    Write-Host -NoNewline $Output
}
"#
}
