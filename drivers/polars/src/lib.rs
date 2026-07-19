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
mod value;

pub use driver::Connection;
pub use driver::get_table_name;
