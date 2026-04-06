use bevy::asset::io::Reader;
use bevy::asset::{AssetPath, LoadContext};
use bevy::platform::collections::HashMap;

/// Parses a properties file (from a dynamic Reader instance) into a HashMap.
/// The properties file should have the format:
/// ```text
/// # This is a comment
/// key1: value1
/// key2: value2
///
/// # Blank lines are ignored
/// key3: value3
/// ```
pub async fn parse_properties(
    reader: &mut dyn Reader,
) -> Result<HashMap<Box<str>, Box<str>>, PropertyParserError> {
    let mut meta_data_bytes = vec![];
    reader
        .read_to_end(&mut meta_data_bytes)
        .await
        .map_err(|e| PropertyParserError::ReaderError(e.to_string()))?;

    let contents =
        String::from_utf8(meta_data_bytes).map_err(|e| PropertyParserError::NotUtf8(e))?;

    let mut properties = HashMap::default();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((mut key, mut value)) = line.split_once(':') else {
            return Err(PropertyParserError::InvalidLine(format!(
                "Invalid property line: {:?}",
                line
            )));
        };

        key = key.trim();
        value = value.trim();

        if key.is_empty() {
            return Err(PropertyParserError::InvalidLine(format!(
                "Property key cannot be empty: {line}"
            )));
        }

        if value.is_empty() {
            return Err(PropertyParserError::InvalidLine(format!(
                "Property value cannot be empty: {line}"
            )));
        }

        properties.insert(key.into(), value.into());
    }

    Ok(properties)
}

pub trait ContextRelativePathEtx {
    /// Gets the relative AssetPath of based on the current context path.
    fn get_relative_path(
        &self,
        relative_path: &str,
    ) -> Result<AssetPath<'static>, RelativePathError>;
}

impl ContextRelativePathEtx for LoadContext<'_> {
    fn get_relative_path(
        &self,
        relative_path: &str,
    ) -> Result<AssetPath<'static>, RelativePathError> {
        let rel_path = "../".to_string() + relative_path;

        let path = self
            .path()
            .resolve(&rel_path)
            .map_err(|e| RelativePathError::PathError(e.to_string()))?;

        Ok(path)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PropertyParserError {
    /// Failed to read from the reader.
    #[error("Failed to read file: {0}")]
    ReaderError(String),

    /// The file is not valid UTF-8.
    #[error("File is not valid UTF-8: {0}")]
    NotUtf8(#[from] std::string::FromUtf8Error),

    /// A line in the file is not a valid property.
    #[error("Invalid property line: {0}")]
    InvalidLine(String),
}

/// An error that can occur when attempting to access a relative path from a
/// context.
#[derive(Debug, thiserror::Error)]
pub enum RelativePathError {
    /// Failed to get the parent directory of the context path.
    #[error("Failed to get parent directory of context path: {0}")]
    PathError(String),
}
