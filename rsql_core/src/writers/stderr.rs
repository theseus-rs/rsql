use crate::writers::writer::Writer;
use std::fmt::Display;
use std::io::{stderr, Result, Write};

#[derive(Debug, Default)]
pub struct StderrWriter;

impl Write for StderrWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        stderr().write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        stderr().flush()
    }
}

impl Display for StderrWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stderr")
    }
}

impl Writer for StderrWriter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer() -> anyhow::Result<()> {
        let mut writer = StderrWriter::default();
        writer.write_all(b"Hello, world!")?;
        writer.flush()?;
        Ok(())
    }
}
