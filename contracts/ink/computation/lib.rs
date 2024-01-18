#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod computation {
    #[ink(storage)]
    pub struct Computation {}

    impl Computation {
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn triangle_number(&self, n: i64) -> i64 {
            let mut sum: i64 = 0;
            for x in 1..=n {
                sum = sum.wrapping_add(x);
            }
            sum
        }

        #[ink(message)]
        #[allow(clippy::arithmetic_side_effects)]
        pub fn odd_product(&self, n: i32) -> i64 {
            let mut prod: i64 = 1;
            for x in 1..=n {
                prod = prod.wrapping_mul(2 * x as i64 - 1);
            }
            prod
        }
    }
}
