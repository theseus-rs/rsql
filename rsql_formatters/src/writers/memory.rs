use crate::writers::writer::Writer;
use std::fmt::Display;
use std::io::{Result, Write};
use std::string::FromUtf8Error;

#[derive(Debug, Default)]
pub struct MemoryWriter {
    buffer: Vec<u8>,
}

impl MemoryWriter {
    #[must_use]
    pub fn new(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    /// Convert the buffer to a UTF-8 string
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is not valid UTF-8
    pub fn as_utf8(&self) -> std::result::Result<String, FromUtf8Error> {
        String::from_utf8(self.buffer.clone())
    }
}

impl Display for MemoryWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_utf8().expect("Invalid UTF-8"))
    }
}

impl Write for MemoryWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.buffer.flush()
    }
}

impl Writer for MemoryWriter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer() -> anyhow::Result<()> {
        let data = "Hello, world!";
        let mut writer = MemoryWriter::default();
        writer.write_all(data.as_bytes())?;
        writer.flush()?;
        assert_eq!(writer.as_slice(), data.as_bytes());
        assert_eq!(writer.as_utf8()?, data);

        let writer = MemoryWriter::new(data.as_bytes().to_vec());
        assert_eq!(writer.to_string(), data);
        Ok(())
    }
}
