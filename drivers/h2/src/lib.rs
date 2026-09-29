#![doc = include_str!("../README.md")]
#![cfg_attr(
    test,
    expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic when verification fails"
    )
)]

mod driver;

pub use driver::Driver;

/// H2 version downloaded by this driver.
pub const H2_VERSION: &str = "2.5.252";
