use contract_build::Target;
use criterion::{criterion_group, criterion_main, Criterion};
use drink::runtime::{MinimalRuntime};
use ink::env::{DefaultEnvironment};
use schlau::{
    drink_api::{CallArgs, DrinkApi},
    ink_build,
};

fn criterion_benchmark(c: &mut Criterion) {
    let contract = "contracts/ink/crypto/Cargo.toml";
    let build_result = ink_build::build_contract(contract, Target::RiscV).unwrap();
    let code = std::fs::read(build_result).unwrap();

    let mut drink_api = DrinkApi::<DefaultEnvironment, MinimalRuntime>::new();

    let value = 0;
    let salt = Vec::new();
    let storage_deposit_limit = None;

    use crypto::crypto::{Crypto, CryptoRef};

    let mut constructor = CryptoRef::new();

    let contract = drink_api
        .ink_instantiate_with_code::<Crypto, _, _>(
            code,
            value,
            &mut constructor,
            salt,
            &subxt_signer::sr25519::dev::alice(),
            storage_deposit_limit,
        )
        .unwrap();

    let message = contract.sha3(10);

    let call_args = CallArgs::from_call_builder(&subxt_signer::sr25519::dev::alice(), &message);

    let mut group = c.benchmark_group("sample-size-example");
    group.sample_size(10);

    group.bench_function("sha3", |b| {
        b.iter(|| drink_api.call(call_args.clone()).unwrap())
    });

    group.finish()
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
