//! Configuration template generation.
//!
//! This module handles generating configuration files by overlaying
//! user values onto the commented template.

use std::fmt::Write;
use toml::Value;

/// Generate a new config file by overlaying user values onto the commented template.
///
/// The template has all fields commented out. This function produces a valid TOML
/// file where user-customized fields are uncommented, and all others stay commented.
/// Segment blocks are reordered to match user segment order.
pub fn generate_overlay(user_values: &Value) -> String {
    let template = include_str!("../config_template.toml");
    let lines: Vec<&str> = template.lines().collect();

    // Phase 1: Parse template into sections and segment blocks
    let (sections, segment_blocks) = parse_template(&lines);

    // Phase 2: Build user segment ID order
    let user_segment_order: Vec<String> = user_values
        .get("segments")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.get("id").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Phase 3: Generate output
    let mut output = String::new();

    // Output non-segment sections
    for section in &sections {
        output.push_str(&render_section(section, user_values));
    }

    // Output segment blocks in user order, then remaining
    if !segment_blocks.is_empty() {
        output.push('\n');
        let mut emitted_ids = std::collections::HashSet::new();

        for user_id in &user_segment_order {
            if let Some(block) = segment_blocks.iter().find(|b| &b.id == user_id) {
                let seg_value = find_segment_value(user_values, user_id);
                output.push_str(&render_segment_block(block, seg_value, false));
                emitted_ids.insert(user_id.clone());
            }
        }

        for block in &segment_blocks {
            if !emitted_ids.contains(&block.id) {
                output.push_str(&render_segment_block(block, None, true));
            }
        }
    }

    // Strip leading/trailing whitespace and ensure trailing newline
    let trimmed = output.trim_start();
    if trimmed.is_empty() {
        template.to_string()
    } else {
        format!("{trimmed}\n")
    }
}

// -- Template Parsing --

/// A section of the template containing non-segment key-value lines.
///
/// Sections are the parts of the template between segment blocks.
struct Section {
    /// Lines belonging to this section (before the next section or segment block)
    lines: Vec<String>,
}

struct SegmentBlock {
    /// The segment ID (extracted from `# id = "..."`)
    id: String,
    /// All lines of this block
    lines: Vec<String>,
}

/// Parses the template into sections and segment blocks.
///
/// Returns a tuple of (sections, `segment_blocks`) where sections contain
/// non-segment template lines, and `segment_blocks` contain segment-specific
/// key-value pairs grouped by segment ID.
fn parse_template(lines: &[&str]) -> (Vec<Section>, Vec<SegmentBlock>) {
    let mut sections = Vec::new();
    let mut segment_blocks = Vec::new();

    let mut current_section_lines: Vec<String> = Vec::new();
    let mut current_segment_lines: Vec<String> = Vec::new();
    let mut current_segment_id: Option<String> = None;
    let mut in_segments = false;

    for line in lines {
        let trimmed = line.trim();

        // Detect commented array-of-tables header for segments
        if is_commented_segments_header(trimmed) {
            if in_segments {
                // Finalize previous segment block
                if let Some(id) = current_segment_id.take() {
                    segment_blocks.push(SegmentBlock {
                        id,
                        lines: std::mem::take(&mut current_segment_lines),
                    });
                }
            }
            in_segments = true;
            current_segment_lines = vec![line.to_string()];
            continue;
        }

        if in_segments {
            // Check if this line starts a new non-segment section
            if parse_commented_section_header(trimmed).is_some() && !is_segment_subsection(trimmed)
            {
                // Finalize current segment block
                if let Some(id) = current_segment_id.take() {
                    segment_blocks.push(SegmentBlock {
                        id,
                        lines: std::mem::take(&mut current_segment_lines),
                    });
                }
                in_segments = false;
                current_section_lines.push(line.to_string());
                continue;
            }

            // Extract segment ID from `# id = "xxx"` line
            if let Some(key_val) = parse_commented_kv(trimmed) {
                if key_val.key == "id" {
                    let id = key_val.value.trim_matches('"').to_string();
                    current_segment_id = Some(id);
                }
            }

            current_segment_lines.push(line.to_string());
            continue;
        }

        current_section_lines.push(line.to_string());
    }

    // Finalize last segment block
    if in_segments {
        if let Some(id) = current_segment_id.take() {
            segment_blocks.push(SegmentBlock {
                id,
                lines: std::mem::take(&mut current_segment_lines),
            });
        }
    }

    if !current_section_lines.is_empty() {
        sections.push(Section {
            lines: current_section_lines,
        });
    }

    (sections, segment_blocks)
}

// -- Rendering --

/// Renders a template section by overlaying user values onto commented template lines.
///
/// Uncomments keys that have user-provided values and preserves the commented
/// format for keys that use default values.
fn render_section(section: &Section, user_values: &Value) -> String {
    let mut output = String::new();
    let mut current_path: Vec<String> = Vec::new();

    for line in &section.lines {
        let trimmed = line.trim();

        if let Some(header) = parse_commented_section_header(trimmed) {
            current_path = header;
            // Check if this section has any user data
            let has_data = section_has_data_in_path(&section.lines, user_values, &current_path);
            if has_data {
                // Uncomment the section header
                let uncommented = trim_comment_prefix(trimmed);
                output.push_str(&uncommented);
                output.push('\n');
            } else {
                output.push_str(line);
                output.push('\n');
            }
            continue;
        }

        if let Some(kv) = parse_commented_kv(trimmed) {
            if let Some(user_val) = get_user_value(user_values, &current_path, &kv.key) {
                let formatted = format_toml_value(user_val);
                let indent = line.len() - trimmed.len();
                output.push_str(&" ".repeat(indent));
                write!(output, "{} = {}", kv.key, formatted).unwrap();
                output.push('\n');
            } else {
                output.push_str(line);
                output.push('\n');
            }
            continue;
        }

        output.push_str(line);
        output.push('\n');
    }

    output
}

/// Renders a segment block by overlaying user values onto commented template lines.
///
/// When `as_comment` is true, renders as fully commented (unused segment).
/// When false, uncomments keys that have user-provided values.
fn render_segment_block(
    block: &SegmentBlock,
    seg_value: Option<&Value>,
    as_comment: bool,
) -> String {
    let mut output = String::new();
    let mut current_path: Vec<String> = Vec::new();

    for line in &block.lines {
        let trimmed = line.trim();

        if is_commented_segments_header(trimmed) {
            if as_comment {
                output.push_str(line);
                output.push('\n');
            } else {
                // Uncomment: `# [[segments]]` → `[[segments]]`
                output.push_str("[[segments]]");
                output.push('\n');
            }
            current_path = vec![];
            continue;
        }

        if let Some(header) = parse_commented_section_header(trimmed) {
            if is_segment_subsection(trimmed) {
                current_path = header;
                if as_comment {
                    output.push_str(line);
                    output.push('\n');
                } else {
                    let uncommented = trim_comment_prefix(trimmed);
                    output.push_str(&uncommented);
                    output.push('\n');
                }
            } else {
                current_path = header;
                output.push_str(line);
                output.push('\n');
            }
            continue;
        }

        if let Some(kv) = parse_commented_kv(trimmed) {
            if as_comment {
                output.push_str(line);
                output.push('\n');
            } else if let Some(val) =
                seg_value.and_then(|v| get_segment_field(v, &current_path, &kv.key))
            {
                let indent = line.len() - trimmed.len();
                output.push_str(&" ".repeat(indent));
                write!(output, "{} = {}", kv.key, format_toml_value(val)).unwrap();
                output.push('\n');
            } else {
                output.push_str(line);
                output.push('\n');
            }
            continue;
        }

        output.push_str(line);
        output.push('\n');
    }

    output
}

// -- Helpers --

struct KvPair {
    key: String,
    value: String,
}

/// Parses a commented key-value line into a [`KvPair`].
///
/// Expects format `# key = value`. Returns None if the line doesn't
/// start with "# " or doesn't contain " = ".
fn parse_commented_kv(trimmed: &str) -> Option<KvPair> {
    if !trimmed.starts_with("# ") {
        return None;
    }
    let content = &trimmed[2..];
    let eq_pos = content.find(" = ")?;
    Some(KvPair {
        key: content[..eq_pos].to_string(),
        value: content[eq_pos + 3..].to_string(),
    })
}

fn parse_commented_section_header(trimmed: &str) -> Option<Vec<String>> {
    if !trimmed.starts_with("# [") {
        return None;
    }
    let content = &trimmed[2..];
    if content.starts_with("[[") {
        return None; // [[segments]] is not a regular section header
    }
    let end = content.find(']')?;
    let path_str = &content[1..end];
    Some(path_str.split('.').map(String::from).collect())
}

fn is_commented_segments_header(trimmed: &str) -> bool {
    trimmed == "# [[segments]]"
}

fn is_segment_subsection(trimmed: &str) -> bool {
    trimmed
        .strip_prefix("# [")
        .is_some_and(|content| content.starts_with("segments."))
}

fn trim_comment_prefix(trimmed: &str) -> String {
    trimmed
        .strip_prefix("# ")
        .or_else(|| trimmed.strip_prefix('#'))
        .unwrap_or(trimmed)
        .to_string()
}

fn get_user_value<'a>(values: &'a Value, path: &[String], key: &str) -> Option<&'a Value> {
    let mut current = values;
    for p in path {
        current = current.get(p)?;
    }
    current.get(key)
}

fn get_segment_field<'a>(seg_value: &'a Value, path: &[String], key: &str) -> Option<&'a Value> {
    if path.is_empty() {
        seg_value.get(key)
    } else {
        let mut current = seg_value;
        for p in path {
            let cleaned = p.strip_prefix("segments.").unwrap_or(p);
            current = current.get(cleaned)?;
        }
        current.get(key)
    }
}

fn find_segment_value<'a>(values: &'a Value, id: &str) -> Option<&'a Value> {
    values
        .get("segments")
        .and_then(|v| v.as_array())?
        .iter()
        .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id))
}

fn section_has_data_in_path(lines: &[String], user_values: &Value, path: &[String]) -> bool {
    let mut check_path = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if let Some(header) = parse_commented_section_header(trimmed) {
            check_path = header;
        } else if let Some(kv) = parse_commented_kv(trimmed) {
            if check_path == path && get_user_value(user_values, &check_path, &kv.key).is_some() {
                return true;
            }
        }
    }
    false
}

fn format_toml_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{s}\""),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => format!("{f}"),
        Value::Boolean(b) => b.to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_toml_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Table(t) => {
            let items: Vec<String> = t
                .iter()
                .map(|(k, v)| format!("{} = {}", k, format_toml_value(v)))
                .collect();
            format!("{{ {} }}", items.join(", "))
        }
        Value::Datetime(dt) => dt.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_overlay() {
        let user = Value::Table(toml::map::Map::new());
        let result = generate_overlay(&user);
        // All lines should remain commented
        for line in result.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('=') {
                // ok: comment, blank, or decoration
            } else {
                panic!("Unexpected uncommented line: {}", trimmed);
            }
        }
    }

    #[test]
    fn test_single_value_overlay() {
        let mut user = toml::map::Map::new();
        let mut style = toml::map::Map::new();
        style.insert("mode".into(), Value::String("ascii".into()));
        user.insert("style".into(), Value::Table(style));

        let result = generate_overlay(&Value::Table(user));

        assert!(result.contains("mode = \"ascii\""));
        assert!(result.contains("# separator = \" | \""));
    }

    #[test]
    fn test_parse_commented_kv() {
        assert!(parse_commented_kv("# not a kv line").is_none());
        assert!(parse_commented_kv("mode = \"auto\"").is_none());

        let kv = parse_commented_kv("# mode = \"auto\"").unwrap();
        assert_eq!(kv.key, "mode");
        assert_eq!(kv.value, "\"auto\"");
    }
}
