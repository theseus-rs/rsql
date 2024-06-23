use crate::writers::writer::Writer;
use std::fmt::{Debug, Display};
use std::io::{Result, Write};

#[derive(Default)]
pub struct FanoutWriter {
    writers: Vec<Box<dyn Writer + Send + Sync>>,
}

impl FanoutWriter {
    #[must_use]
    pub fn new(writers: Vec<Box<dyn Writer + Send + Sync>>) -> Self {
        Self { writers }
    }
}

impl Write for FanoutWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for writer in &mut self.writers {
            let _ = writer.write(buf)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        for writer in &mut self.writers {
            writer.flush()?;
        }
        Ok(())
    }
}

impl Display for FanoutWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let writers = self
            .writers
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{writers}")
    }
}

impl Debug for FanoutWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("fanout")
            .field("writers", &self.writers)
            .finish()
    }
}

impl Writer for FanoutWriter {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writers::MemoryWriter;

    #[test]
    fn test_writer() -> anyhow::Result<()> {
        let data = "foo";
        let memory_writer1 = MemoryWriter::default();
        let memory_writer2 = MemoryWriter::default();
        let mut writer =
            FanoutWriter::new(vec![Box::new(memory_writer1), Box::new(memory_writer2)]);
        writer.write_all(data.as_bytes())?;
        writer.flush()?;
        let output = writer.to_string();
        assert_eq!(output, "foo,foo");
        Ok(())
    }
}
