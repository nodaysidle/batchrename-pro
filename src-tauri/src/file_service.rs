use crate::types::{FileInfo, FileStatus, FileType};
use image::ImageFormat;
use image::ImageReader;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::Path;

const AUDIO_EXTS: &[&str] = &["mp3", "wav", "flac", "m4a", "aac", "ogg", "wma", "aiff"];
const IMAGE_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "avif", "gif", "bmp", "tiff", "tif", "svg", "ico",
];
const VIDEO_EXTS: &[&str] = &["mp4", "webm", "mkv", "avi", "mov", "wmv", "flv", "m4v"];

pub fn detect_file_type(ext: &str) -> FileType {
    let ext_lower = ext.to_lowercase();
    if AUDIO_EXTS.contains(&ext_lower.as_str()) {
        FileType::Audio
    } else if IMAGE_EXTS.contains(&ext_lower.as_str()) {
        FileType::Image
    } else if VIDEO_EXTS.contains(&ext_lower.as_str()) {
        FileType::Video
    } else {
        FileType::Document
    }
}

pub fn validate_and_build_file_info(
    path_str: &str,
    hard_cap: u32,
    current_count: u32,
) -> Result<FileInfo, String> {
    if current_count >= hard_cap {
        return Err(format!("TOO_MANY_FILES: Exceeds hard cap of {}", hard_cap));
    }

    let path = Path::new(path_str);
    let canonical = path
        .canonicalize()
        .map_err(|_| format!("FILE_NOT_FOUND: {}", path_str))?;

    let metadata =
        std::fs::metadata(&canonical).map_err(|e| format!("PERMISSION_DENIED: {}", e))?;

    if metadata.is_dir() {
        return Err(format!("UNSUPPORTED_TYPE: {} is a directory", path_str));
    }

    let original_name = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let extension = canonical
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let file_type = detect_file_type(&extension);
    let thumbnail = generate_thumbnail(&canonical, &file_type);

    Ok(FileInfo {
        id: uuid::Uuid::new_v4().to_string(),
        original_name,
        original_path: canonical.to_string_lossy().to_string(),
        extension,
        size_bytes: metadata.len(),
        file_type,
        thumbnail_data_url: thumbnail,
        status: FileStatus::Pending,
        transformed_name: None,
        error: None,
    })
}

fn generate_thumbnail(path: &Path, file_type: &FileType) -> Option<String> {
    match file_type {
        FileType::Image => {
            let img = ImageReader::open(path).ok()?.decode().ok()?;
            let resized = img.resize(64, 64, image::imageops::FilterType::Lanczos3);
            let mut bytes: Vec<u8> = Vec::new();
            resized
                .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
                .ok()?;
            Some(format!("data:image/png;base64,{}", base64_encode(&bytes)))
        }
        _ => None,
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

pub fn create_backup(original_path: &str, backup_dir: &Path) -> Result<String, String> {
    std::fs::create_dir_all(backup_dir).map_err(|e| format!("BACKUP_FAILED: {}", e))?;

    let path = Path::new(original_path);
    let file_name = path
        .file_name()
        .ok_or_else(|| "BACKUP_FAILED: Invalid file name".to_string())?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    let path_hash = hasher.finish();
    let backup_name = format!(
        "{}_{:016x}_{}",
        timestamp,
        path_hash,
        file_name.to_string_lossy()
    );
    let backup_path = backup_dir.join(&backup_name);

    std::fs::copy(path, &backup_path).map_err(|e| format!("BACKUP_FAILED: {}", e))?;

    Ok(backup_path.to_string_lossy().to_string())
}

pub fn restore_from_backup(backup_path: &str, original_path: &str) -> Result<(), String> {
    std::fs::copy(backup_path, original_path).map_err(|e| format!("RESTORE_FAILED: {}", e))?;
    Ok(())
}
