#[cfg(windows)]
extern crate winres;
use anyhow::Result;

#[cfg(not(windows))]
fn main() -> Result<()> {
    Ok(())
}

#[cfg(windows)]
fn main() -> Result<()> {
    static_vcruntime::metabuild();

    let mut resource = winres::WindowsResource::new();
    resource.set_icon("resources/rsql.ico");
    resource.compile()?;

    Ok(())
}
