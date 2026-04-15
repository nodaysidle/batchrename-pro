use crate::types::{FileInfo, PreviewPair, RenameMode, RenamePattern, CaseTransform};
use regex::Regex;
use chrono::Local;

pub fn generate_previews(
    files: &[FileInfo],
    pattern: &RenamePattern,
) -> Result<Vec<PreviewPair>, String> {
    let mut previews = Vec::with_capacity(files.len());

    for (i, file) in files.iter().enumerate() {
        let new_name = apply_pattern(file, pattern, i)?;
        previews.push(PreviewPair {
            file_id: file.id.clone(),
            original_name: file.original_name.clone(),
            transformed_name: new_name,
            has_conflict: false,
            conflict_reason: None,
        });
    }

    // Detect conflicts
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, preview) in previews.iter_mut().enumerate() {
        if let Some(&first_idx) = seen.get(&preview.transformed_name) {
            preview.has_conflict = true;
            preview.conflict_reason = Some(format!("Duplicates file #{}", first_idx + 1));
        } else {
            seen.insert(preview.transformed_name.clone(), i);
        }
    }

    Ok(previews)
}

fn apply_pattern(file: &FileInfo, pattern: &RenamePattern, index: usize) -> Result<String, String> {
    let stem = Path::new(&file.original_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&file.original_name);

    let mut result = match &pattern.mode {
        RenameMode::Regex => apply_regex(stem, pattern)?,
        RenameMode::Template => apply_template(stem, &file.extension, pattern, index)?,
        RenameMode::Numbering => apply_numbering(pattern, index)?,
    };

    // Apply case transform
    result = apply_case_transform(&result, &pattern.case_transform);

    // Apply prefix/suffix
    if let Some(prefix) = &pattern.prefix {
        result = format!("{}{}", prefix, result);
    }
    if let Some(suffix) = &pattern.suffix {
        result = format!("{}{}", result, suffix);
    }

    // Re-add extension
    if !file.extension.is_empty() {
        result = format!("{}.{}", result, file.extension);
    }

    // Validate: no empty names, no path separators
    if result.trim().is_empty() {
        return Err("EMPTY_RESULT: Pattern produces empty filename".into());
    }
    if result.contains('/') || result.contains('\\') {
        return Err("INVALID_RESULT: Filename contains path separator".into());
    }

    Ok(result)
}

fn apply_regex(stem: &str, pattern: &RenamePattern) -> Result<String, String> {
    let find = pattern
        .regex_find
        .as_ref()
        .ok_or_else(|| "INVALID_REGEX: Missing regex_find".to_string())?;

    let re = Regex::new(find).map_err(|e| format!("INVALID_REGEX: {}", e))?;

    let replace = pattern
        .regex_replace
        .as_deref()
        .unwrap_or("");

    Ok(re.replace_all(stem, replace).to_string())
}

fn apply_template(
    stem: &str,
    ext: &str,
    pattern: &RenamePattern,
    index: usize,
) -> Result<String, String> {
    let template = pattern
        .template
        .as_ref()
        .ok_or_else(|| "INVALID_TEMPLATE: Missing template".to_string())?;

    let mut result = template.clone();

    // {original}
    result = result.replace("{original}", stem);

    // {ext}
    result = result.replace("{ext}", ext);

    // {date} — today's date YYYY-MM-DD
    let today = Local::now().format("%Y-%m-%d").to_string();
    result = result.replace("{date}", &today);

    // {number}
    let start = pattern.start_number.unwrap_or(1) as usize;
    let num = start + index;
    let pad = pattern.zero_pad.unwrap_or(0) as usize;
    if pad > 0 {
        result = result.replace("{number}", &format!("{:0width$}", num, width = pad));
    } else {
        result = result.replace("{number}", &num.to_string());
    }

    // {parent}
    result = result.replace("{parent}", "folder");

    // Check for unknown placeholders
    if result.contains('{') && result.contains('}') {
        return Err("INVALID_TEMPLATE: Contains unknown {placeholder}".into());
    }

    Ok(result)
}

fn apply_numbering(pattern: &RenamePattern, index: usize) -> Result<String, String> {
    let start = pattern.start_number.unwrap_or(1) as usize;
    let num = start + index;
    let pad = pattern.zero_pad.unwrap_or(0) as usize;

    let prefix = pattern.prefix.as_deref().unwrap_or("file");
    let suffix = pattern.suffix.as_deref().unwrap_or("");

    if pad > 0 {
        Ok(format!("{}{:0width$}{}", prefix, num, suffix, width = pad))
    } else {
        Ok(format!("{}{}{}", prefix, num, suffix))
    }
}

fn apply_case_transform(s: &str, transform: &CaseTransform) -> String {
    match transform {
        CaseTransform::None => s.to_string(),
        CaseTransform::Upper => s.to_uppercase(),
        CaseTransform::Lower => s.to_lowercase(),
        CaseTransform::Title => s
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

use std::path::Path;
