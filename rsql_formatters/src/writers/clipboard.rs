use crate::writers::writer::Writer;
use arboard::Clipboard;
use std::fmt::{Debug, Display};
use std::io;
use std::io::{Result, Write};
use std::string::FromUtf8Error;

pub struct ClipboardWriter {
    clipboard: Option<Clipboard>,
    buffer: Vec<u8>,
}

impl ClipboardWriter {
    #[must_use]
    pub fn new(clipboard: Clipboard, buffer: Vec<u8>) -> Self {
        Self {
            clipboard: Some(clipboard),
            buffer,
        }
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

impl Default for ClipboardWriter {
    fn default() -> Self {
        Self {
            clipboard: Clipboard::new().ok(),
            buffer: Vec::new(),
        }
    }
}

impl Debug for ClipboardWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClipboardWriter: {}",
            String::from_utf8_lossy(&self.buffer)
        )
    }
}

impl Display for ClipboardWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.buffer))
    }
}

impl Write for ClipboardWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.buffer.flush()?;
        let data = String::from_utf8(self.buffer.clone()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Failed to convert to UTF-8")
        })?;
        self.clipboard
            .as_mut()
            .ok_or_else(|| io::Error::other("Clipboard is unavailable"))?
            .set_text(data)
            .map_err(|_| io::Error::other("Failed to set clipboard text"))?;
        Ok(())
    }
}

impl Writer for ClipboardWriter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unavailable_clipboard() {
        let mut writer = ClipboardWriter {
            clipboard: None,
            buffer: vec![0xff],
        };
        assert_eq!(format!("{writer:?}"), "ClipboardWriter: �");
        assert_eq!(writer.to_string(), "�");
        assert_eq!(
            writer.flush().map_err(|error| error.kind()),
            Err(io::ErrorKind::InvalidData)
        );

        let mut writer = ClipboardWriter {
            clipboard: None,
            buffer: b"text".to_vec(),
        };
        assert_eq!(
            writer.flush().map_err(|error| error.kind()),
            Err(io::ErrorKind::Other)
        );

        let default_writer = ClipboardWriter::default();
        assert!(default_writer.as_slice().is_empty());
    }

    #[cfg(not(target_os = "linux"))]
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
