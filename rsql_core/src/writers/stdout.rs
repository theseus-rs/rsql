use crate::writers::writer::Writer;
use std::fmt::Display;
use std::io::{stdout, Result, Write};

#[derive(Debug, Default)]
pub struct StdoutWriter;

impl Write for StdoutWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        stdout().write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        stdout().flush()
    }
}

impl Display for StdoutWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stdout")
    }
}

impl Writer for StdoutWriter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer() -> anyhow::Result<()> {
        let mut writer = StdoutWriter::default();
        writer.write_all(b"Hello, world!")?;
        writer.flush()?;
        Ok(())
    }
}
