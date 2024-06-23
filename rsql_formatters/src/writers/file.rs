use crate::writers::writer::Writer;
use std::fmt::Display;
use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;
use std::str::FromStr;
use tempfile::NamedTempFile;

#[derive(Debug)]
pub struct FileWriter {
    file: File,
}

impl FileWriter {
    #[must_use]
    pub fn new(file: File) -> Self {
        Self { file }
    }

    #[must_use]
    pub fn file(&self) -> &File {
        &self.file
    }

    /// Create a new `FileWriter` from a path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            file: File::create(path)?,
        })
    }
}

impl Display for FileWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.file)
    }
}

impl FromStr for FileWriter {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self {
            file: File::create(s)?,
        })
    }
}

impl Write for FileWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.file.flush()
    }
}

impl Writer for FileWriter {}

impl Default for FileWriter {
    fn default() -> Self {
        let file = NamedTempFile::new().expect("Failed to create temporary file");
        FileWriter::new(file.into_file())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer() -> anyhow::Result<()> {
        let mut writer = FileWriter::default();
        writer.write_all(b"Hello, world!")?;
        writer.flush()?;

        let file = NamedTempFile::new().expect("Failed to create temporary file");
        let path = file.as_ref().to_string_lossy().to_string();
        let writer = FileWriter::from_str(path.as_str())?;
        assert!(writer.to_string().contains("File"));

        let writer = FileWriter::from_path(path)?;
        assert!(writer.file().metadata().is_ok());

        Ok(())
    }
}
