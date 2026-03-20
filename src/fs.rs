use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::error::TmError;

pub fn ensure_dir(path: &Path) -> Result<bool, TmError> {
    if path.exists() {
        return Ok(false);
    }
    fs::create_dir_all(path)?;
    Ok(true)
}

pub fn write_file_if_missing(path: &Path, content: &str) -> Result<bool, TmError> {
    if path.exists() {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(true)
}

pub fn merge_json_file(path: &Path, content: &str) -> Result<bool, TmError> {
    let incoming: Value =
        serde_json::from_str(content).map_err(|err| TmError::InvalidJsonMerge {
            path: path.to_path_buf(),
            message: format!("manifest content is not valid json: {err}"),
        })?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(&incoming)?)?;
        return Ok(true);
    }

    let existing_content = fs::read_to_string(path)?;
    let mut existing: Value =
        serde_json::from_str(&existing_content).map_err(|err| TmError::InvalidJsonMerge {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;

    let changed = merge_values(&mut existing, incoming);
    if changed {
        fs::write(path, serde_json::to_string_pretty(&existing)?)?;
    }
    Ok(changed)
}

fn merge_values(existing: &mut Value, incoming: Value) -> bool {
    match (existing, incoming) {
        (Value::Object(existing_map), Value::Object(incoming_map)) => {
            let mut changed = false;
            for (key, incoming_value) in incoming_map {
                match existing_map.get_mut(&key) {
                    Some(existing_value) => {
                        if merge_values(existing_value, incoming_value) {
                            changed = true;
                        }
                    }
                    None => {
                        existing_map.insert(key, incoming_value);
                        changed = true;
                    }
                }
            }
            changed
        }
        (Value::Array(existing_array), Value::Array(incoming_array)) => {
            if existing_array.is_empty() && !incoming_array.is_empty() {
                *existing_array = incoming_array;
                true
            } else {
                false
            }
        }
        (existing_value, incoming_value) => {
            if existing_value.is_null() && !incoming_value.is_null() {
                *existing_value = incoming_value;
                true
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::merge_json_file;

    #[test]
    fn merges_only_missing_json_fields() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("agent.json");
        std::fs::write(&path, "{ \"mcpServers\": { \"existing\": {} } }").unwrap();

        let changed =
            merge_json_file(&path, "{ \"mcpServers\": {}, \"profile\": \"default\" }").unwrap();

        assert!(changed);
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("\"existing\""));
        assert!(content.contains("\"profile\""));
    }
}
