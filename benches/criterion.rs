use contract_build::Target;
use criterion::{criterion_group, criterion_main, Criterion};
use drink::runtime::MinimalRuntime;
use ink::env::DefaultEnvironment;
use schlau::{
    drink_api::{CallArgs, DrinkApi},
    ink_build_and_instantiate,
};

fn crypto(c: &mut Criterion) {
    use crypto::crypto::{Crypto, CryptoRef};

    let mut drink_api = DrinkApi::new();
    let contract = ink_build_and_instantiate::<_, MinimalRuntime, DefaultEnvironment, Crypto, _, _>(
        "contracts/ink/crypto/Cargo.toml",
        Target::RiscV,
        &mut CryptoRef::new(),
        &mut drink_api,
    );

    let message = contract.sha3(u32::MAX);
    let call_args = CallArgs::from_call_builder(&subxt_signer::sr25519::dev::alice(), &message);

    let mut group = c.benchmark_group("crypto");
    group.sample_size(50);

    group.bench_function("sha3", |b| {
        b.iter(|| {
            // std::thread::sleep(std::time::Duration::from_millis(20));
            drink_api.call(call_args.clone()).unwrap()
        })
    });
    group.finish()
}

fn computation(c: &mut Criterion) {
    use computation::computation::{Computation, ComputationRef};

    let mut drink_api = DrinkApi::new();
    let contract =
        ink_build_and_instantiate::<_, MinimalRuntime, DefaultEnvironment, Computation, _, _>(
            "contracts/ink/computation/Cargo.toml",
            Target::RiscV,
            &mut ComputationRef::new(),
            &mut drink_api,
        );

    let message = contract.odd_product(i32::MAX);
    let call_args = CallArgs::from_call_builder(&subxt_signer::sr25519::dev::alice(), &message);

    let mut group = c.benchmark_group("computation");
    group.sample_size(50);

    group.bench_function("odd_product", |b| {
        b.iter(|| drink_api.call(call_args.clone()).unwrap())
    });
    group.finish()
}

criterion_group!(benches, crypto, computation);
criterion_main!(benches);
