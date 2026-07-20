#![cfg_attr(
    test,
    expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic when verification fails"
    )
)]

mod driver;
mod metadata;
mod results;

pub use driver::Driver;
pub use metadata::get_metadata;
