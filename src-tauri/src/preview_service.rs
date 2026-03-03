use crate::types::*;
use regex::RegexBuilder;
use std::collections::HashMap;

/// Generate rename previews for all files based on the given pattern.
/// Pure in-memory — no disk I/O.
pub fn generate_previews(
    files: &[FileInfo],
    pattern: &RenamePattern,
) -> Result<Vec<PreviewResult>, String> {
    validate_pattern(pattern)?;

    let previews: Vec<PreviewResult> = files
        .iter()
        .enumerate()
        .map(|(idx, file)| {
            let stem = file
                .original_name
                .rsplit_once('.')
                .map(|(s, _)| s)
                .unwrap_or(&file.original_name);

            let ext = &file.extension;

            let new_stem = match &pattern.mode {
                RenameMode::Regex => apply_regex(stem, pattern)?,
                RenameMode::Template => apply_template(stem, ext, pattern, idx),
                RenameMode::Numbering => apply_numbering(stem, pattern, idx),
            };

            let new_stem = apply_case_transform(&new_stem, pattern.case_transform.as_ref());

            let transformed_name = if ext.is_empty() {
                new_stem
            } else {
                format!("{}.{}", new_stem, ext)
            };

            if transformed_name.trim().is_empty()
                || transformed_name.trim() == format!(".{}", ext)
            {
                return Err("EMPTY_RESULT: Pattern produces empty filename".to_string());
            }

            Ok(PreviewResult {
                file_id: file.id.clone(),
                original_name: file.original_name.clone(),
                transformed_name,
                has_conflict: false,
                conflict_reason: None,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(detect_conflicts(previews))
}

/// Validate the rename pattern before applying.
pub fn validate_pattern(pattern: &RenamePattern) -> Result<(), String> {
    match &pattern.mode {
        RenameMode::Regex => {
            let find = pattern
                .regex_find
                .as_deref()
                .ok_or("INVALID_REGEX: regex_find is required for regex mode")?;

            RegexBuilder::new(find)
                .size_limit(1 << 20) // 1MB limit for ReDoS prevention
                .build()
                .map_err(|e| format!("INVALID_REGEX: {}", e))?;

            // regex_replace defaults to empty string (delete matches)
            Ok(())
        }
        RenameMode::Template => {
            let tmpl = pattern
                .template
                .as_deref()
                .ok_or("INVALID_TEMPLATE: template is required for template mode")?;

            // Validate only known placeholders
            let allowed = ["{date}", "{number}", "{original}", "{ext}", "{parent}"];
            let mut i = 0;
            let bytes = tmpl.as_bytes();
            while i < bytes.len() {
                if bytes[i] == b'{' {
                    if let Some(end) = tmpl[i..].find('}') {
                        let placeholder = &tmpl[i..i + end + 1];
                        if !allowed.contains(&placeholder) {
                            return Err(format!(
                                "INVALID_TEMPLATE: Unknown placeholder '{}'",
                                placeholder
                            ));
                        }
                        i += end + 1;
                    } else {
                        return Err("INVALID_TEMPLATE: Unclosed brace in template".to_string());
                    }
                } else {
                    i += 1;
                }
            }
            Ok(())
        }
        RenameMode::Numbering => Ok(()),
    }
}

fn apply_regex(stem: &str, pattern: &RenamePattern) -> Result<String, String> {
    let find = pattern.regex_find.as_deref().unwrap_or("");
    let replace = pattern.regex_replace.as_deref().unwrap_or("");

    let re = RegexBuilder::new(find)
        .size_limit(1 << 20)
        .build()
        .map_err(|e| format!("INVALID_REGEX: {}", e))?;

    Ok(re.replace_all(stem, replace).to_string())
}

fn apply_template(stem: &str, ext: &str, pattern: &RenamePattern, index: usize) -> String {
    let tmpl = pattern.template.as_deref().unwrap_or("{original}");
    let start = pattern.start_number.unwrap_or(1) as usize;
    let pad = pattern.zero_pad.unwrap_or(1) as usize;
    let num = start + index;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    tmpl.replace("{original}", stem)
        .replace("{ext}", ext)
        .replace("{date}", &today)
        .replace("{number}", &format!("{:0>width$}", num, width = pad))
        .replace("{parent}", "")
}

fn apply_numbering(stem: &str, pattern: &RenamePattern, index: usize) -> String {
    let start = pattern.start_number.unwrap_or(1) as usize;
    let pad = pattern.zero_pad.unwrap_or(1) as usize;
    let prefix = pattern.prefix.as_deref().unwrap_or("");
    let suffix = pattern.suffix.as_deref().unwrap_or("");
    let num = start + index;

    format!(
        "{}{}{}{}",
        prefix,
        format!("{:0>width$}", num, width = pad),
        suffix,
        if stem.is_empty() { String::new() } else { format!("_{}", stem) }
    )
}

fn apply_case_transform(s: &str, transform: Option<&CaseTransform>) -> String {
    match transform {
        Some(CaseTransform::Upper) => s.to_uppercase(),
        Some(CaseTransform::Lower) => s.to_lowercase(),
        Some(CaseTransform::Title) => s
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => {
                        let upper: String = c.to_uppercase().collect();
                        format!("{}{}", upper, chars.as_str().to_lowercase())
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        Some(CaseTransform::None) | None => s.to_string(),
    }
}

fn detect_conflicts(mut previews: Vec<PreviewResult>) -> Vec<PreviewResult> {
    let mut name_count: HashMap<String, usize> = HashMap::new();
    for p in &previews {
        *name_count.entry(p.transformed_name.to_lowercase()).or_insert(0) += 1;
    }

    for p in &mut previews {
        let count = name_count.get(&p.transformed_name.to_lowercase()).copied().unwrap_or(0);
        if count > 1 {
            p.has_conflict = true;
            p.conflict_reason = Some("Duplicate output name".to_string());
        }
    }

    previews
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_file(id: &str, name: &str) -> FileInfo {
        let (_stem, ext) = name.rsplit_once('.').unwrap_or((name, ""));
        FileInfo {
            id: id.to_string(),
            original_name: name.to_string(),
            original_path: format!("/tmp/{}", name),
            extension: ext.to_string(),
            size_bytes: 1000,
            file_type: FileType::Document,
            thumbnail_data_url: None,
            status: FileStatus::Pending,
        }
    }

    #[test]
    fn test_regex_rename() {
        let files = vec![
            make_file("1", "photo_001.jpg"),
            make_file("2", "photo_002.jpg"),
        ];
        let pattern = RenamePattern {
            mode: RenameMode::Regex,
            regex_find: Some("photo_(\\d+)".to_string()),
            regex_replace: Some("image_$1".to_string()),
            template: None,
            start_number: None,
            zero_pad: None,
            prefix: None,
            suffix: None,
            case_transform: None,
        };

        let result = generate_previews(&files, &pattern).unwrap();
        assert_eq!(result[0].transformed_name, "image_001.jpg");
        assert_eq!(result[1].transformed_name, "image_002.jpg");
        assert!(!result[0].has_conflict);
    }

    #[test]
    fn test_template_rename() {
        let files = vec![
            make_file("1", "song.mp3"),
            make_file("2", "track.mp3"),
        ];
        let pattern = RenamePattern {
            mode: RenameMode::Template,
            regex_find: None,
            regex_replace: None,
            template: Some("{original}_{number}".to_string()),
            start_number: Some(1),
            zero_pad: Some(3),
            prefix: None,
            suffix: None,
            case_transform: None,
        };

        let result = generate_previews(&files, &pattern).unwrap();
        assert_eq!(result[0].transformed_name, "song_001.mp3");
        assert_eq!(result[1].transformed_name, "track_002.mp3");
    }

    #[test]
    fn test_numbering_rename() {
        let files = vec![
            make_file("1", "a.txt"),
            make_file("2", "b.txt"),
        ];
        let pattern = RenamePattern {
            mode: RenameMode::Numbering,
            regex_find: None,
            regex_replace: None,
            template: None,
            start_number: Some(5),
            zero_pad: Some(2),
            prefix: Some("file_".to_string()),
            suffix: None,
            case_transform: None,
        };

        let result = generate_previews(&files, &pattern).unwrap();
        assert_eq!(result[0].transformed_name, "file_05_a.txt");
        assert_eq!(result[1].transformed_name, "file_06_b.txt");
    }

    #[test]
    fn test_case_transform_upper() {
        let files = vec![make_file("1", "hello world.txt")];
        let pattern = RenamePattern {
            mode: RenameMode::Regex,
            regex_find: Some("(.+)".to_string()),
            regex_replace: Some("$1".to_string()),
            template: None,
            start_number: None,
            zero_pad: None,
            prefix: None,
            suffix: None,
            case_transform: Some(CaseTransform::Upper),
        };

        let result = generate_previews(&files, &pattern).unwrap();
        assert_eq!(result[0].transformed_name, "HELLO WORLD.txt");
    }

    #[test]
    fn test_conflict_detection() {
        let files = vec![
            make_file("1", "a.txt"),
            make_file("2", "b.txt"),
        ];
        let pattern = RenamePattern {
            mode: RenameMode::Regex,
            regex_find: Some(".+".to_string()),
            regex_replace: Some("same".to_string()),
            template: None,
            start_number: None,
            zero_pad: None,
            prefix: None,
            suffix: None,
            case_transform: None,
        };

        let result = generate_previews(&files, &pattern).unwrap();
        assert!(result[0].has_conflict);
        assert!(result[1].has_conflict);
        assert_eq!(result[0].conflict_reason.as_deref(), Some("Duplicate output name"));
    }

    #[test]
    fn test_invalid_regex() {
        let files = vec![make_file("1", "test.txt")];
        let pattern = RenamePattern {
            mode: RenameMode::Regex,
            regex_find: Some("[invalid".to_string()),
            regex_replace: None,
            template: None,
            start_number: None,
            zero_pad: None,
            prefix: None,
            suffix: None,
            case_transform: None,
        };

        let result = generate_previews(&files, &pattern);
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("INVALID_REGEX:"));
    }

    #[test]
    fn test_invalid_template_placeholder() {
        let files = vec![make_file("1", "test.txt")];
        let pattern = RenamePattern {
            mode: RenameMode::Template,
            regex_find: None,
            regex_replace: None,
            template: Some("{unknown}".to_string()),
            start_number: None,
            zero_pad: None,
            prefix: None,
            suffix: None,
            case_transform: None,
        };

        let result = generate_previews(&files, &pattern);
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("INVALID_TEMPLATE:"));
    }

    #[test]
    fn test_performance_500_files() {
        let files: Vec<FileInfo> = (0..500)
            .map(|i| make_file(&format!("{}", i), &format!("file_{:04}.txt", i)))
            .collect();
        let pattern = RenamePattern {
            mode: RenameMode::Regex,
            regex_find: Some("file_(\\d+)".to_string()),
            regex_replace: Some("renamed_$1".to_string()),
            template: None,
            start_number: None,
            zero_pad: None,
            prefix: None,
            suffix: None,
            case_transform: None,
        };

        let start = std::time::Instant::now();
        let result = generate_previews(&files, &pattern).unwrap();
        let elapsed = start.elapsed();

        assert_eq!(result.len(), 500);
        // Performance target is <100ms in release builds; debug mode is much slower
        #[cfg(not(debug_assertions))]
        assert!(elapsed.as_millis() < 100, "Preview took {}ms, should be under 100ms", elapsed.as_millis());
        let _ = elapsed;
    }
}
