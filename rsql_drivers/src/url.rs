use crate::Error::IoError;
use crate::Result;
use anyhow::anyhow;
use std::path::{PathBuf, MAIN_SEPARATOR_STR};
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
        let path = if path.contains(':') && path.starts_with('/') {
            // Strip preceding '/' Windows absolute path (e.g. /C:/foo)
            path.strip_prefix('/').unwrap()
        } else {
            path
        };

        let path = path.replace('/', MAIN_SEPARATOR_STR);
        let file_path = PathBuf::from(path);
        if !file_path.exists() && !file_path.is_file() {
            let file_path = file_path.to_string_lossy();
            return Err(IoError(anyhow!("File not found: {file_path}")));
        }
        Ok(file_path)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::dataset_url;

    #[test]
    fn test_file() -> Result<()> {
        let dataset_file = dataset_url("file", "users.csv");
        let url = Url::parse(dataset_file.as_str())?;
        let path = url.to_file()?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn test_file_error() -> Result<()> {
        let url = Url::parse("file:///foo")?;
        assert!(url.to_file().is_err());
        Ok(())
    }
}
