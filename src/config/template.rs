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

    #[test]
    fn test_format_toml_value_string() {
        assert_eq!(
            format_toml_value(&Value::String("hello".into())),
            "\"hello\""
        );
    }

    #[test]
    fn test_format_toml_value_integer() {
        assert_eq!(format_toml_value(&Value::Integer(42)), "42");
    }

    #[test]
    fn test_format_toml_value_float() {
        assert_eq!(format_toml_value(&Value::Float(3.14)), "3.14");
    }

    #[test]
    fn test_format_toml_value_boolean() {
        assert_eq!(format_toml_value(&Value::Boolean(true)), "true");
        assert_eq!(format_toml_value(&Value::Boolean(false)), "false");
    }

    #[test]
    fn test_format_toml_value_array() {
        let arr = Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(format_toml_value(&arr), "[1, 2, 3]");
    }

    #[test]
    fn test_format_toml_value_array_strings() {
        let arr = Value::Array(vec![Value::String("a".into()), Value::String("b".into())]);
        assert_eq!(format_toml_value(&arr), "[\"a\", \"b\"]");
    }

    #[test]
    fn test_format_toml_value_table() {
        let mut map = toml::map::Map::new();
        map.insert("x".into(), Value::Integer(1));
        map.insert("y".into(), Value::Integer(2));
        let table = Value::Table(map);
        let result = format_toml_value(&table);
        // Order may vary
        assert!(result.contains("x = 1"));
        assert!(result.contains("y = 2"));
        assert!(result.starts_with('{'));
        assert!(result.ends_with('}'));
    }

    #[test]
    fn test_format_toml_value_empty_array() {
        let arr = Value::Array(vec![]);
        assert_eq!(format_toml_value(&arr), "[]");
    }

    #[test]
    fn test_parse_commented_section_header() {
        assert_eq!(
            parse_commented_section_header("# [style]"),
            Some(vec!["style".to_string()])
        );
        assert_eq!(
            parse_commented_section_header("# [api]"),
            Some(vec!["api".to_string()])
        );
        // Nested path
        assert_eq!(
            parse_commented_section_header("# [segments.options]"),
            Some(vec!["segments".to_string(), "options".to_string()])
        );
        // Not a section header
        assert_eq!(parse_commented_section_header("not a header"), None);
        // Array of tables is not a regular section
        assert_eq!(parse_commented_section_header("# [[segments]]"), None);
        // Missing bracket
        assert_eq!(parse_commented_section_header("# [style"), None);
    }

    #[test]
    fn test_is_commented_segments_header() {
        assert!(is_commented_segments_header("# [[segments]]"));
        assert!(!is_commented_segments_header("# [style]"));
        assert!(!is_commented_segments_header("[[segments]]"));
    }

    #[test]
    fn test_is_segment_subsection() {
        assert!(is_segment_subsection("# [segments.options]"));
        assert!(is_segment_subsection("# [segments.icon]"));
        assert!(!is_segment_subsection("# [style]"));
        assert!(!is_segment_subsection("# [api]"));
    }

    #[test]
    fn test_trim_comment_prefix() {
        assert_eq!(trim_comment_prefix("# key = value"), "key = value");
        assert_eq!(trim_comment_prefix("#key"), "key");
        assert_eq!(trim_comment_prefix("no comment"), "no comment");
    }

    #[test]
    fn test_get_user_value() {
        let mut root = toml::map::Map::new();
        let mut style = toml::map::Map::new();
        style.insert("mode".into(), Value::String("ascii".into()));
        root.insert("style".into(), Value::Table(style));
        let val = Value::Table(root);

        assert!(get_user_value(&val, &[], "nonexistent").is_none());
        assert!(get_user_value(&val, &["style".to_string()], "mode").is_some());
        assert!(get_user_value(&val, &["nonexistent".to_string()], "mode").is_none());
    }

    #[test]
    fn test_get_segment_field() {
        let mut seg = toml::map::Map::new();
        seg.insert("id".into(), Value::String("token_usage".into()));
        seg.insert("enabled".into(), Value::Boolean(false));
        let seg_val = Value::Table(seg);

        assert_eq!(
            get_segment_field(&seg_val, &[], "id").and_then(|v| v.as_str().map(String::from)),
            Some("token_usage".to_string())
        );
        assert!(get_segment_field(&seg_val, &[], "nonexistent").is_none());

        // With path
        let mut nested = toml::map::Map::new();
        let mut icon = toml::map::Map::new();
        icon.insert("emoji".into(), Value::String("🪙".into()));
        nested.insert("icon".into(), Value::Table(icon));
        let nested_val = Value::Table(nested);
        assert!(get_segment_field(&nested_val, &["segments.icon".to_string()], "emoji").is_some());
    }

    #[test]
    fn test_find_segment_value() {
        let mut root = toml::map::Map::new();
        let seg1 = {
            let mut m = toml::map::Map::new();
            m.insert("id".into(), Value::String("token_usage".into()));
            Value::Table(m)
        };
        let seg2 = {
            let mut m = toml::map::Map::new();
            m.insert("id".into(), Value::String("mcp_usage".into()));
            Value::Table(m)
        };
        root.insert("segments".into(), Value::Array(vec![seg1, seg2]));
        let val = Value::Table(root);

        assert!(find_segment_value(&val, "token_usage").is_some());
        assert!(find_segment_value(&val, "mcp_usage").is_some());
        assert!(find_segment_value(&val, "nonexistent").is_none());
    }

    #[test]
    fn test_find_segment_value_no_segments() {
        let val = Value::Table(toml::map::Map::new());
        assert!(find_segment_value(&val, "token_usage").is_none());
    }

    #[test]
    fn test_generate_overlay_with_segments() {
        let raw: Value = toml::from_str(
            r##"
[style]
mode = "ascii"
[[segments]]
id = "token_usage"
enabled = false
"##,
        )
        .unwrap();

        let result = generate_overlay(&raw);
        assert!(result.contains("mode = \"ascii\""));
        assert!(result.contains("[[segments]]"));
        assert!(result.contains("enabled = false"));
    }

    #[test]
    fn test_generate_overlay_preserves_user_segment_order() {
        let raw: Value = toml::from_str(
            r##"
[style]
mode = "auto"
[[segments]]
id = "mcp_usage"
enabled = false
[[segments]]
id = "token_usage"
enabled = false
"##,
        )
        .unwrap();

        let result = generate_overlay(&raw);
        let mcp_pos = result.find("mcp_usage").unwrap();
        let token_pos = result.find("token_usage").unwrap();
        assert!(mcp_pos < token_pos, "user segment order must be preserved");
    }

    #[test]
    fn test_section_has_data_in_path() {
        let lines = vec![
            "# [style]".to_string(),
            "# mode = \"auto\"".to_string(),
            "# separator = \" | \"".to_string(),
        ];
        let mut style = toml::map::Map::new();
        style.insert("mode".into(), Value::String("ascii".into()));
        let mut root = toml::map::Map::new();
        root.insert("style".into(), Value::Table(style));
        let user = Value::Table(root);

        assert!(section_has_data_in_path(
            &lines,
            &user,
            &["style".to_string()]
        ));
        assert!(!section_has_data_in_path(
            &lines,
            &user,
            &["api".to_string()]
        ));
    }

    #[test]
    fn test_render_segment_block_as_comment() {
        let block = SegmentBlock {
            id: "test".to_string(),
            lines: vec![
                "# [[segments]]".to_string(),
                "# id = \"test\"".to_string(),
                "# enabled = true".to_string(),
            ],
        };
        let result = render_segment_block(&block, None, true);
        // All lines should remain commented
        for line in result.lines() {
            if !line.is_empty() {
                assert!(line.starts_with('#'));
            }
        }
    }

    #[test]
    fn test_render_segment_block_uncommented() {
        let block = SegmentBlock {
            id: "token_usage".to_string(),
            lines: vec![
                "# [[segments]]".to_string(),
                "# id = \"token_usage\"".to_string(),
                "# enabled = true".to_string(),
            ],
        };
        let result = render_segment_block(&block, None, false);
        assert!(result.contains("[[segments]]"));
        assert!(result.contains("id = \"token_usage\""));
    }
}
