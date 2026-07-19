#[cfg(windows)]
extern crate winres;
#[cfg(windows)]
use anyhow::Result;

#[cfg(windows)]
fn main() -> Result<()> {
    static_vcruntime::metabuild();

    let mut resource = winres::WindowsResource::new();
    resource.set_icon("resources/rsql.ico");
    resource.compile()?;

    Ok(())
}

#[cfg(not(windows))]
fn main() {}
