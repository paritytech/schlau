#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod crypto {
    #[ink(storage)]
    pub struct Crypto {}

    impl Crypto {
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn sha3(&self, iterations: u32) -> u32 {
            use sha3::{Digest, Sha3_256};

            let mut hashes = ink::prelude::vec::Vec::new();
            for i in 0..iterations {
                let mut hasher = Sha3_256::new();
                hasher.update(i.to_le_bytes());
                let hash = hasher.finalize();
                hashes.push(hash);
            }
            hashes.len() as u32
        }
    }
}
