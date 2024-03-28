use crate::writers::MemoryWriter;
use std::fmt::{Debug, Display};
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct Output {
    writer: Box<dyn Writer + Send + Sync>,
}

impl Output {
    pub fn new(writer: Box<dyn Writer + Send + Sync>) -> Self {
        Self { writer }
    }

    pub fn get(&self) -> &dyn Writer {
        self.writer.as_ref()
    }

    pub fn set(&mut self, writer: Box<dyn Writer + Send + Sync>) {
        self.writer = writer;
    }

    pub fn to_string(&self) -> String {
        self.writer.to_string()
    }

    pub fn write_fmt(&mut self, fmt: std::fmt::Arguments) -> io::Result<()> {
        self.writer.write_fmt(fmt)
    }
}

impl Default for Output {
    fn default() -> Self {
        Self {
            writer: Box::new(MemoryWriter::default()),
        }
    }
}

// impl Display for Output {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.writer.to_string())
//     }
// }

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
