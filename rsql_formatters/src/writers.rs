#[cfg(not(target_family = "wasm"))]
mod clipboard;
mod fanout;
mod file;
mod memory;
mod stderr;
mod stdout;
mod writer;

#[cfg(not(target_family = "wasm"))]
pub use clipboard::ClipboardWriter;
pub use fanout::FanoutWriter;
pub use file::FileWriter;
pub use memory::MemoryWriter;
pub use stderr::StderrWriter;
pub use stdout::StdoutWriter;
pub use writer::{Output, Writer};
