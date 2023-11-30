use contract_build::Target;
use criterion::{criterion_group, criterion_main, Criterion};
use drink::runtime::{MinimalRuntime};
use ink::env::{DefaultEnvironment};
use schlau::{
    drink_api::{CallArgs, DrinkApi},
    ink_build,
};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
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

        let message = contract.sha3(1000);

        let call_args = CallArgs::from_call_builder(&subxt_signer::sr25519::dev::alice(), &message);

        b.iter(|| drink_api.call(call_args.clone()).unwrap())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
