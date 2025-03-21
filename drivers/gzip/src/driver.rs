use async_trait::async_trait;
use file_type::FileType;
use flate2::read::GzDecoder;
use rsql_driver::Error::{ConversionError, IoError};
use rsql_driver::{Connection, DriverManager, Result, UrlExtension};
use std::fs::{File, create_dir_all};
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tracing::debug;
use url::Url;

/// Driver for Gzip compressed files.  The driver decompresses the original file and then delegates
/// to the appropriate driver based on the decompressed file type.
#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "gzip"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn Connection>> {
        let temp_dir = TempDir::new()?;
        let parsed_url = Url::parse(url)?;
        let file = parsed_url.to_file()?;
        let file_path = Self::decompress_file(&file, temp_dir.path())?
            .to_string_lossy()
            .to_string();
        let file_type =
            FileType::try_from_file(&file_path).map_err(|error| IoError(error.to_string()))?;

        debug!("temp_dir: {temp_dir:?}; file_path: {file_path}");
        let driver = DriverManager::get_by_file_type(file_type)?;
        match driver {
            Some(driver) => {
                let (_url, parameters) = url.split_once('?').unwrap_or((url, ""));
                let url = format!("{}://{file_path}?{parameters}", driver.identifier());
                let connection = driver.connect(url.as_str()).await?;
                Ok(connection)
            }
            None => Err(IoError(format!(
                "{file_path:?}: {:?}",
                file_type.media_types()
            ))),
        }
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type.extensions().contains(&"gz")
    }
}

impl Driver {
    fn decompress_file(file: &PathBuf, temp_dir: &Path) -> Result<PathBuf> {
        let Some(file_name) = file.file_name() else {
            return Err(ConversionError(format!(
                "File name is not a valid path: {file:?}"
            )));
        };

        let mut file_name = PathBuf::from(file_name);
        if let Some(extension) = file_name.extension() {
            if extension.to_string_lossy().to_string().to_lowercase() == "gz" {
                file_name.set_extension("");
            }
        }
        create_dir_all(temp_dir)?;
        file_name = temp_dir.join(file_name);

        let file =
            File::open(file).map_err(|error| IoError(format!("Failed to open file: {error}")))?;

        let mut decoder = GzDecoder::new(file);
        let mut output_file = File::create(&file_name)
            .map_err(|error| IoError(format!("Failed to create output file: {error}")))?;

        io::copy(&mut decoder, &mut output_file)
            .map_err(|error| IoError(format!("Failed to decompress data: {error}")))?;

        Ok(file_name)
    }
}
