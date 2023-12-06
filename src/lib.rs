pub use drink_api::DrinkApi;

pub mod drink_api;

pub mod ink_build;

pub mod ink_drink;

#[cfg(feature = "riscv")]
pub use drink_riscv as drink;
#[cfg(feature = "wasm")]
pub use drink_wasm as drink;
