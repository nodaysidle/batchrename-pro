use crate::types::{FileInfo, FileStatus, FileType};
use std::path::Path;
use uuid::Uuid;

const AUDIO_EXTS: &[&str] = &["mp3", "wav", "flac", "m4a", "ogg", "aac", "wma"];
const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "avif", "bmp", "tiff", "gif", "ico"];
const VIDEO_EXTS: &[&str] = &["mp4", "webm", "mkv", "avi", "mov", "wmv", "flv"];

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

pub fn generate_thumbnail(path: &Path, file_type: &FileType) -> Option<String> {
    if *file_type != FileType::Image {
        return None;
    }

    match image::open(path) {
        Ok(img) => {
            let thumb = img.thumbnail(64, 64);
            let mut buf = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut buf);
            thumb
                .write_to(&mut cursor, image::ImageFormat::Png)
                .ok()?;
            let b64 = base64_encode(&buf);
            Some(format!("data:image/png;base64,{}", b64))
        }
        Err(e) => {
            tracing::warn!("Failed to generate thumbnail for {:?}: {}", path, e);
            None
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(4 * (data.len() / 3 + 1));
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

pub fn validate_and_create_file_info(paths: &[String], hard_cap: u32) -> Result<Vec<FileInfo>, String> {
    if paths.len() as u32 > hard_cap {
        return Err(format!(
            "TOO_MANY_FILES: Cannot add {} files, maximum is {}",
            paths.len(),
            hard_cap
        ));
    }

    let mut files = Vec::with_capacity(paths.len());

    for path_str in paths {
        let path = Path::new(path_str);

        let canonical = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => {
                return Err(format!("FILE_NOT_FOUND: Path does not exist: {}", path_str));
            }
        };

        if !canonical.is_file() {
            continue; // Skip directories silently
        }

        let metadata = std::fs::metadata(&canonical)
            .map_err(|_| format!("PERMISSION_DENIED: Cannot read file: {}", path_str))?;

        let file_name = canonical
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let extension = canonical
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        let file_type = detect_file_type(&extension);
        let thumbnail = generate_thumbnail(&canonical, &file_type);

        files.push(FileInfo {
            id: Uuid::new_v4().to_string(),
            original_name: file_name,
            original_path: canonical.to_string_lossy().to_string(),
            extension,
            size_bytes: metadata.len(),
            file_type,
            thumbnail_data_url: thumbnail,
            status: FileStatus::Pending,
        });
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type() {
        assert_eq!(detect_file_type("mp3"), FileType::Audio);
        assert_eq!(detect_file_type("MP3"), FileType::Audio);
        assert_eq!(detect_file_type("jpg"), FileType::Image);
        assert_eq!(detect_file_type("mp4"), FileType::Video);
        assert_eq!(detect_file_type("txt"), FileType::Document);
        assert_eq!(detect_file_type("pdf"), FileType::Document);
    }

    #[test]
    fn test_hard_cap_enforcement() {
        let paths: Vec<String> = (0..10).map(|i| format!("/tmp/file_{}.txt", i)).collect();
        let result = validate_and_create_file_info(&paths, 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("TOO_MANY_FILES"));
    }
}
