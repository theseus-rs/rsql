use crate::Error::IoError;
use crate::Result;
use std::path::{MAIN_SEPARATOR_STR, PathBuf};
use url::Url;

pub trait UrlExtension {
    /// Get a file path from the URL
    ///
    /// # Errors
    /// if the file does not exist
    fn to_file(&self) -> Result<PathBuf>;
}

impl UrlExtension for Url {
    fn to_file(&self) -> Result<PathBuf> {
        let url = self.as_str();
        let scheme = self.scheme();
        let start_index = scheme.len() + 3;
        let end_index = url.find('?').unwrap_or(url.len());
        let path = &url[start_index..end_index];

        #[cfg(target_os = "windows")]
        let path = if path.contains(':') {
            // Strip preceding '/' character for Windows absolute path (e.g. /C:/foo)
            path.strip_prefix('/').unwrap_or(path)
        } else {
            path
        };

        let path = path.replace('/', MAIN_SEPARATOR_STR);
        let file_path = PathBuf::from(path);
        let path = file_path.to_string_lossy().to_string();
        if path.is_empty() || path == MAIN_SEPARATOR_STR {
            return Err(IoError("No file provided".to_string()));
        }
        Ok(file_path)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver_test_utils::dataset_url;

    #[test]
    fn test_file() -> Result<()> {
        let dataset_file = dataset_url("file", "users.csv");
        let url = Url::parse(dataset_file.as_str())?;
        let path = url.to_file()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn test_file_path() -> Result<()> {
        let url = Url::parse("file:///foo")?;
        let path = url.to_file()?.to_string_lossy().to_string();
        assert!(path.ends_with("foo"));
        Ok(())
    }

    #[test]
    fn test_file_error() -> Result<()> {
        let url = Url::parse("file://")?;
        assert!(url.to_file().is_err());
        Ok(())
    }
}
