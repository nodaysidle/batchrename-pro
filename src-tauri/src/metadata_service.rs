use id3::TagLike;
use std::collections::HashMap;
use std::path::Path;

/// Read ID3v2 tags from an audio file.
pub fn read_id3_tags(path: &Path) -> Result<HashMap<String, String>, String> {
    let tag = id3::Tag::read_from_path(path)
        .map_err(|e| format!("NO_METADATA: {}", e))?;

    let mut tags = HashMap::new();

    if let Some(v) = tag.title() {
        tags.insert("title".to_string(), v.to_string());
    }
    if let Some(v) = tag.artist() {
        tags.insert("artist".to_string(), v.to_string());
    }
    if let Some(v) = tag.album() {
        tags.insert("album".to_string(), v.to_string());
    }
    if let Some(v) = tag.year() {
        tags.insert("year".to_string(), v.to_string());
    }
    if let Some(v) = tag.track() {
        tags.insert("track".to_string(), v.to_string());
    }
    if let Some(v) = tag.genre_parsed() {
        tags.insert("genre".to_string(), v.to_string());
    }

    Ok(tags)
}

/// Write ID3v2 tags to an audio file.
pub fn write_id3_tags(path: &Path, tags: &HashMap<String, Option<String>>) -> Result<(), String> {
    let mut tag = id3::Tag::read_from_path(path).unwrap_or_default();

    for (key, value) in tags {
        match key.as_str() {
            "title" => {
                if let Some(v) = value {
                    tag.set_title(v.clone());
                } else {
                    tag.remove_title();
                }
            }
            "artist" => {
                if let Some(v) = value {
                    tag.set_artist(v.clone());
                } else {
                    tag.remove_artist();
                }
            }
            "album" => {
                if let Some(v) = value {
                    tag.set_album(v.clone());
                } else {
                    tag.remove_album();
                }
            }
            "year" => {
                if let Some(v) = value {
                    if let Ok(y) = v.parse::<i32>() {
                        tag.set_year(y);
                    }
                }
            }
            "track" => {
                if let Some(v) = value {
                    if let Ok(t) = v.parse::<u32>() {
                        tag.set_track(t);
                    }
                }
            }
            "genre" => {
                if let Some(v) = value {
                    tag.set_genre(v.clone());
                } else {
                    tag.remove_genre();
                }
            }
            _ => {}
        }
    }

    tag.write_to_path(path, id3::Version::Id3v24)
        .map_err(|e| format!("WRITE_FAILED: {}", e))?;
    Ok(())
}

/// Strip all ID3 tags from a file.
pub fn strip_id3(path: &Path) -> Result<(), String> {
    id3::Tag::remove_from_path(path)
        .map_err(|e| format!("WRITE_FAILED: {}", e))?;
    Ok(())
}

/// Read EXIF data from an image file.
pub fn read_exif(path: &Path) -> Result<HashMap<String, String>, String> {
    let file = std::fs::File::open(path)
        .map_err(|e| format!("NO_METADATA: {}", e))?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exif = exif::Reader::new()
        .read_from_container(&mut bufreader)
        .map_err(|e| format!("NO_METADATA: {}", e))?;

    let mut tags = HashMap::new();
    for field in exif.fields() {
        let key = field.tag.to_string();
        let value = field.display_value().with_unit(&exif).to_string();
        tags.insert(key, value);
    }

    Ok(tags)
}

/// Strip EXIF data from an image by re-encoding without metadata.
pub fn strip_exif(path: &Path) -> Result<(), String> {
    let img = image::open(path)
        .map_err(|e| format!("WRITE_FAILED: Cannot open image: {}", e))?;

    // Re-save the image to strip all metadata
    img.save(path)
        .map_err(|e| format!("WRITE_FAILED: {}", e))?;

    Ok(())
}

/// Get editable field descriptors for a file type.
pub fn get_editable_fields(file_type: &str) -> Vec<FieldDescriptor> {
    match file_type {
        "audio" => vec![
            FieldDescriptor { key: "title".into(), label: "Title".into(), editable: true },
            FieldDescriptor { key: "artist".into(), label: "Artist".into(), editable: true },
            FieldDescriptor { key: "album".into(), label: "Album".into(), editable: true },
            FieldDescriptor { key: "year".into(), label: "Year".into(), editable: true },
            FieldDescriptor { key: "track".into(), label: "Track".into(), editable: true },
            FieldDescriptor { key: "genre".into(), label: "Genre".into(), editable: true },
        ],
        "image" => vec![
            FieldDescriptor { key: "exif".into(), label: "EXIF Data".into(), editable: false },
        ],
        _ => vec![],
    }
}

pub struct FieldDescriptor {
    pub key: String,
    pub label: String,
    pub editable: bool,
}
