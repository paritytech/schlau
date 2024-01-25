#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod nop {
    #[ink(storage)]
    pub struct Nop {}

    impl Nop {
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn baseline(&self, _param: u32) {}
    }
}
