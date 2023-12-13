#[cfg(feature = "riscv")]
pub use drink_riscv as drink;
#[cfg(feature = "wasm")]
pub use drink_wasm as drink;
pub mod drink_api;
#[cfg(feature = "evm")]
pub mod evm;
pub mod ink;
pub mod solang;
pub mod solc;
