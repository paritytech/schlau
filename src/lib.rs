use contract_build::Target;

#[cfg(feature = "riscv")]
pub use drink_riscv as drink;
#[cfg(feature = "wasm")]
pub use drink_wasm as drink;
pub mod drink_api;
pub mod evm;
pub mod ink;
pub mod solang;
pub mod solc;

pub const fn target() -> Target {
    if cfg!(feature = "wasm") {
        Target::Wasm
    } else if cfg!(feature = "riscv") {
        Target::RiscV
    } else {
        panic!("No VM target feature enabled")
    }
}

pub const fn target_str() -> &'static str {
    match target() {
        Target::Wasm => "wasm",
        Target::RiscV => "riscv",
    }
}
