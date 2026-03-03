use std::path::Path;

/// Validate if a source format can be converted to the target format.
pub fn validate_conversion(source_ext: &str, target_ext: &str) -> bool {
    let src = source_ext.to_lowercase();
    let tgt = target_ext.to_lowercase();

    if src == tgt {
        return false;
    }

    let image_formats = ["jpg", "jpeg", "png", "webp", "avif", "bmp", "gif", "tiff"];
    let audio_formats = ["mp3", "wav", "flac", "m4a", "ogg", "aac"];
    let video_formats = ["mp4", "webm", "mkv", "avi", "mov"];

    let src_is_image = image_formats.contains(&src.as_str());
    let tgt_is_image = image_formats.contains(&tgt.as_str());
    let src_is_audio = audio_formats.contains(&src.as_str());
    let tgt_is_audio = audio_formats.contains(&tgt.as_str());
    let src_is_video = video_formats.contains(&src.as_str());
    let tgt_is_video = video_formats.contains(&tgt.as_str());

    // Can only convert within the same media category
    (src_is_image && tgt_is_image)
        || (src_is_audio && tgt_is_audio)
        || (src_is_video && tgt_is_video)
}

/// Convert an image file using the pure-Rust `image` crate.
pub fn convert_image(
    input: &Path,
    output: &Path,
    quality: Option<u8>,
    resize: Option<(u32, u32, bool)>,
) -> Result<(), String> {
    let img = image::open(input)
        .map_err(|e| format!("CONVERSION_ERROR: Failed to open image: {}", e))?;

    let img = if let Some((width, height, maintain_aspect)) = resize {
        if maintain_aspect {
            img.resize(width, height, image::imageops::FilterType::Lanczos3)
        } else {
            img.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
        }
    } else {
        img
    };

    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => {
            let q = quality.unwrap_or(85);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                std::fs::File::create(output)
                    .map_err(|e| format!("CONVERSION_ERROR: {}", e))?,
                q,
            );
            img.write_with_encoder(encoder)
                .map_err(|e| format!("CONVERSION_ERROR: {}", e))?;
        }
        "png" => {
            img.save(output)
                .map_err(|e| format!("CONVERSION_ERROR: {}", e))?;
        }
        "webp" => {
            img.save(output)
                .map_err(|e| format!("CONVERSION_ERROR: {}", e))?;
        }
        "bmp" => {
            img.save(output)
                .map_err(|e| format!("CONVERSION_ERROR: {}", e))?;
        }
        "gif" => {
            img.save(output)
                .map_err(|e| format!("CONVERSION_ERROR: {}", e))?;
        }
        "tiff" => {
            img.save(output)
                .map_err(|e| format!("CONVERSION_ERROR: {}", e))?;
        }
        "avif" => {
            img.save(output)
                .map_err(|e| format!("CONVERSION_ERROR: AVIF encoding: {}", e))?;
        }
        _ => {
            return Err(format!(
                "UNSUPPORTED_CONVERSION: Cannot convert to '{}'",
                ext
            ));
        }
    }

    Ok(())
}

/// Stub for audio conversion.
/// Full implementation requires ffmpeg-next crate bindings.
pub fn convert_audio(
    _input: &Path,
    _output: &Path,
    _quality: Option<u8>,
) -> Result<(), String> {
    Err("UNSUPPORTED_CONVERSION: Audio conversion requires ffmpeg (not yet linked)".to_string())
}

/// Stub for video conversion.
/// Full implementation requires ffmpeg-next crate bindings.
pub fn convert_video(
    _input: &Path,
    _output: &Path,
    _codec: Option<&str>,
    _bitrate: Option<&str>,
) -> Result<(), String> {
    Err("UNSUPPORTED_CONVERSION: Video conversion requires ffmpeg (not yet linked)".to_string())
}

/// Get supported target formats for a given source extension.
pub fn supported_targets(source_ext: &str) -> Vec<&'static str> {
    let ext = source_ext.to_lowercase();

    let image_formats = &["jpg", "png", "webp", "avif", "bmp", "gif", "tiff"];
    let audio_formats = &["mp3", "wav", "flac", "m4a"];
    let video_formats = &["mp4", "webm", "mkv"];

    if image_formats.contains(&ext.as_str()) || ext == "jpeg" {
        image_formats
            .iter()
            .copied()
            .filter(|f| *f != ext.as_str() && *f != "jpeg")
            .collect()
    } else if audio_formats.contains(&ext.as_str()) {
        audio_formats
            .iter()
            .copied()
            .filter(|f| *f != ext.as_str())
            .collect()
    } else if video_formats.contains(&ext.as_str()) {
        video_formats
            .iter()
            .copied()
            .filter(|f| *f != ext.as_str())
            .collect()
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_conversion() {
        assert!(validate_conversion("jpg", "png"));
        assert!(validate_conversion("png", "webp"));
        assert!(validate_conversion("mp3", "wav"));
        assert!(!validate_conversion("jpg", "mp3")); // cross-category
        assert!(!validate_conversion("png", "png")); // same format
    }

    #[test]
    fn test_supported_targets() {
        let targets = supported_targets("jpg");
        assert!(targets.contains(&"png"));
        assert!(targets.contains(&"webp"));
        assert!(!targets.contains(&"jpg"));

        let targets = supported_targets("mp3");
        assert!(targets.contains(&"wav"));
        assert!(targets.contains(&"flac"));
    }
}
