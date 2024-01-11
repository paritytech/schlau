use criterion::{criterion_group, criterion_main, Criterion};
use ink::env::DefaultEnvironment;
use subxt_signer::sr25519::dev;

use schlau::{drink::runtime::MinimalRuntime, drink_api::CallArgs, ink::InkDrink};

macro_rules! ink_contract_bench {
    ( $name:ident, $contract:ident, $contract_ref:ident, $message:ident, $args:tt) => {
        fn $message(c: &mut Criterion) {
            use $name::$name::{$contract, $contract_ref};

            let contract_name = stringify!($name);

            let mut ink_drink = InkDrink::<DefaultEnvironment, MinimalRuntime>::new();
            let contract = ink_drink.build_and_instantiate::<_, $contract, _, _>(
                &format!("contracts/ink/{}/Cargo.toml", contract_name),
                &mut $contract_ref::new(),
            );

            let message = contract.$message($args);
            let call_args = CallArgs::from_call_builder(dev::alice(), &message);

            let group_name = format!("{}_{}", stringify!($message), stringify!($args));

            let mut group = c.benchmark_group(group_name);
            group.sample_size(30);
            group.measurement_time(std::time::Duration::from_secs(20));

            let target = schlau::target_str();
            let bench_name = format!("ink_{}", target);

            group.bench_function(bench_name, |b| {
                b.iter(|| ink_drink.drink.call(call_args.clone()).unwrap())
            });

            group.finish()
        }
    };
}

ink_contract_bench!(crypto, Crypto, CryptoRef, sha3, 1000);
ink_contract_bench!(
    computation,
    Computation,
    ComputationRef,
    odd_product,
    1000000
);
ink_contract_bench!(
    computation,
    Computation,
    ComputationRef,
    triangle_number,
    1000000
);

criterion_group!(benches, sha3, odd_product, triangle_number);
criterion_main!(benches);
