use crate::writers::MemoryWriter;
use std::fmt::{Debug, Display};
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct Output {
    writer: Box<dyn Writer + Send + Sync>,
}

impl Output {
    #[must_use]
    pub fn new(writer: Box<dyn Writer + Send + Sync>) -> Self {
        Self { writer }
    }

    pub fn set(&mut self, writer: Box<dyn Writer + Send + Sync>) {
        self.writer = writer;
    }

    /// Write a formatted string to the output
    ///
    /// # Errors
    ///
    /// Returns an error if the write operation fails
    pub fn write_fmt(&mut self, fmt: std::fmt::Arguments) -> io::Result<()> {
        self.writer.write_fmt(fmt)
    }
}

impl Default for Output {
    fn default() -> Self {
        Output::new(Box::<MemoryWriter>::default())
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.writer)
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

pub trait Writer: Debug + Display + Write {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output() -> anyhow::Result<()> {
        let mut output = Output::default();
        let writer = MemoryWriter::default();
        output.set(Box::new(writer));
        output.write_all(b"Hello, world!")?;
        output.flush()?;
        assert_eq!(output.to_string(), "Hello, world!");
        Ok(())
    }
}
