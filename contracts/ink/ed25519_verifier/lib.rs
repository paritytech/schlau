#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod ed25519_verifier {
    pub static EXAMPLE_KEY: [u8; 32] = [
        157, 97, 177, 157, 239, 253, 90, 96, 186, 132, 74, 244, 146, 236, 44, 196, 68, 73, 197,
        105, 123, 50, 105, 25, 112, 59, 172, 3, 28, 174, 127, 96,
    ];
    use ed25519_dalek::{Signature, Verifier, VerifyingKey, PUBLIC_KEY_LENGTH};

    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct Ed25519Verifier {}

    impl Ed25519Verifier {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn verify(
            &self,
            pubkey: [u8; PUBLIC_KEY_LENGTH],
            signature: [u8; 64],
            message: Vec<u8>,
        ) -> bool {
            let verifying_key: VerifyingKey = VerifyingKey::from_bytes(&pubkey).unwrap();
            verifying_key
                .verify(&message, &Signature::from_bytes(&signature))
                .is_ok()
        }
    }
}
