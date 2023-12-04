#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod crypto {
    #[ink(storage)]
    pub struct Crypto {}

    impl Crypto {
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new() -> Self {
            ink::env::debug_println!("check debug");
            Self {}
        }

        #[ink(message)]
        pub fn sha3(&self, iterations: u32) -> u32 {
            use sha3::{Digest, Sha3_256};

            ink::env::debug_println!("iterations: {:?}", iterations);

            let mut hashes = ink::prelude::vec::Vec::new();
            for i in 0..iterations {
                let mut hasher = Sha3_256::new();
                hasher.update(i.to_le_bytes());
                let hash = hasher.finalize();
                hashes.push(hash);
            }
            hashes.len() as u32
        }

        #[ink(message)]
        pub fn triangle_number(&self, n: i64) {
            let _res: i64 = (1..=n as i64).fold(0, |sum, x| sum.wrapping_add(x));
        }

        #[ink(message)]
        pub fn odd_product(&self, n: i32) -> i64 {
            (1..=n as i64).fold(1, |prod, x| prod.wrapping_mul(2i64.wrapping_mul(x.wrapping_sub(1i64))))
        }
    }
}
