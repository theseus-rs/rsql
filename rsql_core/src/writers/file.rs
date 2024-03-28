use crate::writers::writer::Writer;
use std::fmt::Display;
use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub struct FileWriter {
    file: File,
}

impl FileWriter {
    pub fn new(file: File) -> Self {
        Self { file }
    }

    pub fn file(&self) -> &File {
        &self.file
    }

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
        Self {
            file: tempfile::tempfile().expect("Failed to create temporary file"),
        }
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
        Ok(())
    }
}
