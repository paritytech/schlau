#[cfg(feature = "riscv")]
pub use drink_riscv as drink;
#[cfg(feature = "wasm")]
pub use drink_wasm as drink;
pub mod drink_api;
pub mod ink;
