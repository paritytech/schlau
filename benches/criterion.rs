use criterion::{criterion_group, criterion_main, Criterion};
use ink::env::DefaultEnvironment;

use schlau::drink::runtime::MinimalRuntime;
use schlau::drink_api::{CallArgs, DrinkApi};

fn crypto(c: &mut Criterion) {
    use crypto::crypto::{Crypto, CryptoRef};

    let mut drink_api = DrinkApi::<DefaultEnvironment, MinimalRuntime>::new();
    let contract = drink_api.build_and_instantiate::<_, Crypto, _, _>(
        "contracts/ink/crypto/Cargo.toml",
        &mut CryptoRef::new(),
    );

    let message = contract.sha3(100);
    let call_args = CallArgs::from_call_builder(&subxt_signer::sr25519::dev::alice(), &message);

    let mut group = c.benchmark_group("crypto");
    group.sample_size(50);

    group.bench_function("sha3", |b| {
        b.iter(|| drink_api.call(call_args.clone()).unwrap())
    });
    group.finish()
}

fn computation(c: &mut Criterion) {
    use computation::computation::{Computation, ComputationRef};

    let mut drink_api = DrinkApi::<DefaultEnvironment, MinimalRuntime>::new();
    let contract = drink_api.build_and_instantiate::<_, Computation, _, _>(
        "contracts/ink/computation/Cargo.toml",
        &mut ComputationRef::new(),
    );

    let message = contract.odd_product(100_000);
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
