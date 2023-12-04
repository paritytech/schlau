use contract_build::Target;
use criterion::{criterion_group, criterion_main, Criterion};
use drink::runtime::MinimalRuntime;
use ink::env::DefaultEnvironment;
use parity_scale_codec::Decode;
use schlau::{
    drink_api::{CallArgs, DrinkApi},
    ink_build_and_instantiate,
};

fn crypto_hash(c: &mut Criterion) {
    use crypto::crypto::{Crypto, CryptoRef};

    let mut drink_api = DrinkApi::new();
    let contract = ink_build_and_instantiate::<_, MinimalRuntime, DefaultEnvironment, Crypto, _, _>(
        "contracts/ink/crypto/Cargo.toml",
        Target::RiscV,
        &mut CryptoRef::new(),
        &mut drink_api,
    );

    let message = contract.sha3(1_000_000);
    let call_args = CallArgs::from_call_builder(&subxt_signer::sr25519::dev::alice(), &message);

    let mut group = c.benchmark_group("crypto_hash");
    group.sample_size(50);

    let instant = std::time::Instant::now();
    let result = drink_api.call(call_args.clone()).unwrap();
    // let hashes_len = u32::decode(&mut &result[..]).unwrap();
    println!("Time elapsed for {result:?}: {:?}", instant.elapsed());

    group.bench_function("sha3", |b| {
        b.iter(|| {
            // std::thread::sleep(std::time::Duration::from_millis(20));
            drink_api.call(call_args.clone()).unwrap()
        })
    });

    group.finish()
}

criterion_group!(benches, crypto_hash);
criterion_main!(benches);
