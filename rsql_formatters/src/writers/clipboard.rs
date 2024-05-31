use crate::writers::writer::Writer;
use arboard::Clipboard;
use std::fmt::{Debug, Display};
use std::io;
use std::io::{Result, Write};
use std::string::FromUtf8Error;

pub struct ClipboardWriter {
    clipboard: Clipboard,
    buffer: Vec<u8>,
}

impl ClipboardWriter {
    pub fn new(clipboard: Clipboard, buffer: Vec<u8>) -> Self {
        Self { clipboard, buffer }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    pub fn as_utf8(&self) -> std::result::Result<String, FromUtf8Error> {
        String::from_utf8(self.buffer.clone())
    }
}

impl Default for ClipboardWriter {
    fn default() -> Self {
        Self::new(
            Clipboard::new().expect("Failed to create clipboard"),
            Vec::new(),
        )
    }
}

impl Debug for ClipboardWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClipboardWriter: {}",
            self.as_utf8().expect("Invalid UTF-8")
        )
    }
}

impl Display for ClipboardWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_utf8().expect("Invalid UTF-8"))
    }
}

impl Write for ClipboardWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.buffer.flush()?;
        let data = String::from_utf8(self.buffer.to_vec()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Failed to convert to UTF-8")
        })?;
        self.clipboard
            .set_text(data)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to set clipboard text"))?;
        Ok(())
    }
}

impl Writer for ClipboardWriter {}

#[cfg(not(target_os = "linux"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer() -> anyhow::Result<()> {
        let data = "Hello, world!";
        let mut writer = ClipboardWriter::default();
        writer.write_all(data.as_bytes())?;
        writer.flush()?;

        let mut clipboard = Clipboard::new()?;
        let clipboard_data = clipboard.get_text()?;
        assert_eq!(data, clipboard_data);
        assert_eq!(writer.as_slice(), data.as_bytes());
        assert_eq!(writer.as_utf8()?, data);
        Ok(())
    }
}
