use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ed25519_verifier::ed25519_verifier::{Ed25519Verifier, Ed25519VerifierRef};
use ink::env::DefaultEnvironment;
use parity_scale_codec::decode_from_bytes;
use subxt_signer::sr25519::dev;

use schlau::{drink::runtime::MinimalRuntime, drink_api::CallArgs, ink::InkDrink};

macro_rules! ink_contract_bench {
    ( $name:ident, $contract:ident, $contract_ref:ident, $message:ident, $args:tt) => {
        fn $message(c: &mut Criterion) {
            use $name::$name::{$contract, $contract_ref};

            let contract_name = stringify!($name);

            let mut group = c.benchmark_group(stringify!($message));
            group.sample_size(30);

            for args in $args {
                let mut ink_drink = InkDrink::<DefaultEnvironment, MinimalRuntime>::new();
                let contract = ink_drink.build_and_instantiate::<_, $contract, _, _>(
                    &format!("contracts/ink/{}/Cargo.toml", contract_name),
                    &mut $contract_ref::new(),
                );

                let message = contract.$message(args);
                let call_args =
                    CallArgs::from_call_builder(dev::alice(), &message).with_max_gas_limit();

                let parameter = args.to_string();
                let id = BenchmarkId::new(&format!("ink({})", schlau::target_str()), parameter);

                group.bench_function(id, |b| {
                    b.iter(|| ink_drink.drink.call(call_args.clone()).unwrap())
                });
            }

            group.finish()
        }
    };
}

fn ed25519_verify(c: &mut Criterion) {
    use ed25519_dalek::{Signature, Signer, SigningKey};
    let signing_key: SigningKey = SigningKey::from_bytes(&[
        157, 97, 177, 157, 239, 253, 90, 96, 186, 132, 74, 244, 146, 236, 44, 196, 68, 73, 197,
        105, 123, 50, 105, 25, 112, 59, 172, 3, 28, 174, 127, 96,
    ]);
    let message: &[u8] = b"This is the contracts runtime benchmark. We compare the runtime performance of various Wasm, RISC-V and EVM smart contracts.";
    let signature: Signature = signing_key.sign(message);
    let verifying_key = signing_key.verifying_key().to_bytes();

    let contract_name = "ed25519_verifier";

    let mut group = c.benchmark_group("verify");
    group.sample_size(30);

    let mut ink_drink = InkDrink::<DefaultEnvironment, MinimalRuntime>::new();
    let contract = ink_drink.build_and_instantiate::<_, Ed25519Verifier, _, _>(
        &format!("contracts/ink/{}/Cargo.toml", contract_name),
        &mut Ed25519VerifierRef::new(),
    );

    let message = contract.verify(verifying_key, signature.to_bytes(), message.to_vec());
    let call_args = CallArgs::from_call_builder(dev::alice(), &message).with_max_gas_limit();

    let result: Result<bool, ()> =
        decode_from_bytes(ink_drink.drink.call(call_args.clone()).unwrap().into()).unwrap();
    assert!(result.expect("signature verification should succeed"));

    let id = BenchmarkId::new(&format!("ink({})", schlau::target_str()), "ed25516_verify");

    group.bench_function(id, |b| {
        b.iter(|| ink_drink.drink.call(call_args.clone()).unwrap())
    });

    group.finish()
}

ink_contract_bench!(crypto, Crypto, CryptoRef, sha3, [2000, 4000, 8000]);
ink_contract_bench!(
    computation,
    Computation,
    ComputationRef,
    odd_product,
    [2_000_000, 4_000_000, 8_000_000]
);
ink_contract_bench!(
    computation,
    Computation,
    ComputationRef,
    triangle_number,
    [3_000_000, 6_000_000, 12_000_000]
);

ink_contract_bench!(nop, Nop, NopRef, baseline, [0]);

criterion_group!(
    benches,
    baseline,
    sha3,
    odd_product,
    triangle_number,
    ed25519_verify
);
criterion_main!(benches);
