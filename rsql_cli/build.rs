#[cfg(windows)]
extern crate winres;
use anyhow::Result;

fn main() -> Result<()> {
    #[cfg(windows)]
    {
        static_vcruntime::metabuild();

        let mut resource = winres::WindowsResource::new();
        resource.set_icon("resources/rsql.ico");
        resource.compile()?;
    }

    Ok(())
}
