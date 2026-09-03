use crate::file_service;
use crate::types::{CaseTransform, FileInfo, PreviewPair, RenameMode, RenamePattern};
use chrono::Local;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// One filesystem hop in a planned apply. Swaps/cycles may produce a temp hop
/// (`create_backup` is true only on the file's first hop from the original path).
#[derive(Debug, Clone)]
pub struct RenameStep {
    pub file_index: usize,
    pub file_id: String,
    pub from: PathBuf,
    pub to: PathBuf,
    pub create_backup: bool,
}

struct PlannedMove {
    index: usize,
    file_id: String,
    from: PathBuf,
    to: PathBuf,
    create_backup: bool,
    ext: String,
}

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

    // Detect conflicts inside the batch. Mark both the first output and later
    // duplicates so the UI can block the whole unsafe set.
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut duplicate_pairs = Vec::new();
    for (i, preview) in previews.iter().enumerate() {
        let parent = Path::new(&files[i].original_path)
            .parent()
            .unwrap_or(Path::new(""));
        let key = parent
            .join(&preview.transformed_name)
            .to_string_lossy()
            .to_string();
        if let Some(&first_idx) = seen.get(&key) {
            duplicate_pairs.push((first_idx, i));
        } else {
            seen.insert(key, i);
        }
    }
    for (first_idx, duplicate_idx) in duplicate_pairs {
        previews[first_idx].has_conflict = true;
        previews[first_idx].conflict_reason =
            Some(format!("Duplicates file #{}", duplicate_idx + 1));
        previews[duplicate_idx].has_conflict = true;
        previews[duplicate_idx].conflict_reason =
            Some(format!("Duplicates file #{}", first_idx + 1));
    }

    mark_occupancy_conflicts(files, &mut previews);

    Ok(previews)
}

fn dest_path(file: &FileInfo, transformed_name: &str) -> PathBuf {
    Path::new(&file.original_path)
        .parent()
        .unwrap_or(Path::new(""))
        .join(transformed_name)
}

/// Occupancy is a preview rule: an existing target is fatal only when it is
/// *not* being vacated by another file in this batch. Intra-batch chains
/// (A→B while B→C) and swaps are planned at apply time, not blocked here.
pub(crate) fn mark_occupancy_conflicts(files: &[FileInfo], previews: &mut [PreviewPair]) {
    let mut source_index: HashMap<String, usize> = HashMap::new();
    for (i, file) in files.iter().enumerate() {
        source_index.insert(file_service::path_key(Path::new(&file.original_path)), i);
    }

    let moving: Vec<bool> = files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let dest = dest_path(file, &previews[i].transformed_name);
            file_service::path_key(Path::new(&file.original_path)) != file_service::path_key(&dest)
        })
        .collect();

    for i in 0..previews.len() {
        if previews[i].has_conflict {
            continue;
        }
        let original_path = Path::new(&files[i].original_path);
        let target_path = dest_path(&files[i], &previews[i].transformed_name);
        if !target_path.exists() {
            continue;
        }
        if file_service::paths_refer_to_same_file(original_path, &target_path) {
            continue;
        }

        let occupant_idx = source_index.get(&file_service::path_key(&target_path)).copied();
        if let Some(occ) = occupant_idx {
            if occ != i && moving[occ] {
                // Occupant is in this batch and will move away — plan, don't block.
                continue;
            }
        }

        previews[i].has_conflict = true;
        previews[i].conflict_reason = Some("Target already exists".into());
    }
}

/// Build a sequential apply order: reverse-occupancy (vacate destinations
/// first). Cycles (swaps) get a unique temp hop in the same directory.
pub fn plan_renames(files: &[FileInfo], previews: &[PreviewPair]) -> Result<Vec<RenameStep>, String> {
    if files.len() != previews.len() {
        return Err("PLAN_ERROR: Preview count does not match file count".into());
    }
    if previews.iter().any(|p| p.has_conflict) {
        return Err("CONFLICTS_DETECTED: Resolve preview conflicts before applying".into());
    }

    let mut remaining: Vec<PlannedMove> = Vec::new();
    for (i, file) in files.iter().enumerate() {
        let from = PathBuf::from(&file.original_path);
        let to = dest_path(file, &previews[i].transformed_name);
        if file_service::path_key(&from) == file_service::path_key(&to) {
            continue;
        }
        remaining.push(PlannedMove {
            index: i,
            file_id: file.id.clone(),
            from,
            to,
            create_backup: true,
            ext: file.extension.clone(),
        });
    }

    let mut steps = Vec::new();
    while !remaining.is_empty() {
        let ready_idx = find_ready_indices(&remaining);
        if ready_idx.is_empty() {
            // Cycle: hop the first remaining file to a temp name so its
            // original path is vacated and the rest of the cycle can proceed.
            let mut hop = remaining.remove(0);
            let parent = hop
                .from
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            let temp = file_service::unique_temp_path(&parent, &hop.ext);
            steps.push(RenameStep {
                file_index: hop.index,
                file_id: hop.file_id.clone(),
                from: hop.from.clone(),
                to: temp.clone(),
                create_backup: hop.create_backup,
            });
            hop.from = temp;
            hop.create_backup = false;
            remaining.insert(0, hop);
            continue;
        }

        let ready_set: HashSet<usize> = ready_idx.into_iter().collect();
        let mut next_remaining = Vec::new();
        for (i, mv) in remaining.into_iter().enumerate() {
            if ready_set.contains(&i) {
                steps.push(RenameStep {
                    file_index: mv.index,
                    file_id: mv.file_id,
                    from: mv.from,
                    to: mv.to,
                    create_backup: mv.create_backup,
                });
            } else {
                next_remaining.push(mv);
            }
        }
        remaining = next_remaining;
    }

    Ok(steps)
}

fn find_ready_indices(moves: &[PlannedMove]) -> Vec<usize> {
    let occupied: HashMap<String, usize> = moves
        .iter()
        .enumerate()
        .map(|(i, mv)| (file_service::path_key(&mv.from), i))
        .collect();

    moves
        .iter()
        .enumerate()
        .filter(|(i, mv)| match occupied.get(&file_service::path_key(&mv.to)) {
            None => true,
            Some(&occ) => occ == *i,
        })
        .map(|(i, _)| i)
        .collect()
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

    // Apply prefix/suffix. Numbering mode already embeds prefix/suffix in
    // apply_numbering, so applying them again here would double them up.
    if !matches!(pattern.mode, RenameMode::Numbering) {
        if let Some(prefix) = &pattern.prefix {
            result = format!("{}{}", prefix, result);
        }
        if let Some(suffix) = &pattern.suffix {
            result = format!("{}{}", result, suffix);
        }
    }

    // Validate the stem before re-adding extensions. A pattern that produces
    // ".txt" is still an empty user-visible name and must not be applied.
    if result.trim().is_empty() {
        return Err("EMPTY_RESULT: Pattern produces empty filename".into());
    }

    // Re-add extension
    if !file.extension.is_empty() {
        result = format!("{}.{}", result, file.extension);
    }

    // Validate: no path separators
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

    let replace = pattern.regex_replace.as_deref().unwrap_or("");

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

    let prefix = pattern
        .prefix
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("file");
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
