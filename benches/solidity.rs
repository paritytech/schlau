use criterion::{criterion_group, criterion_main, Criterion};

use alloy_dyn_abi::{DynSolValue, JsonAbiExt};
use alloy_json_abi::JsonAbi;
use alloy_primitives::I256;
use schlau::{
    evm::{
        CallArgs as EvmCallArgs, CreateArgs as EvmCreateArgs, EvmRuntime, EvmSandbox,
        DEFAULT_ACCOUNT,
    },
    solang::SolangContract,
};
use sp_core::{H160, U256};

pub struct EvmContract {
    address: H160,
    abi: JsonAbi,
    sandbox: EvmSandbox<EvmRuntime>,
}

impl EvmContract {
    fn init(contract: &str) -> Self {
        let result =
            schlau::solc::build_contract(&format!("contracts/solidity/{}.sol", contract)).unwrap();
        let mut sandbox = EvmSandbox::<EvmRuntime>::new();

        let create_args = EvmCreateArgs {
            source: DEFAULT_ACCOUNT,
            init: result.code,
            gas_limit: 1_000_000_000,
            max_fee_per_gas: U256::from(1_000_000_000),
            ..Default::default()
        };
        let address = sandbox.create(create_args).unwrap();
        EvmContract {
            address,
            abi: result.abi,
            sandbox,
        }
    }

    pub fn call_args(&self, func: &str, args: &[DynSolValue]) -> EvmCallArgs {
        let func = &self.abi.function(func).unwrap()[0];
        let data = func.abi_encode_input(args).unwrap();

        EvmCallArgs {
            source: DEFAULT_ACCOUNT,
            target: self.address.clone(),
            input: data,
            gas_limit: 1_000_000_000,
            max_fee_per_gas: U256::from(1_000_000_000),
            ..Default::default()
        }
    }
}

macro_rules! bench_solang {
    ( $group:ident, $contract:ident, $message:ident, $args:tt) => {
        let mut solang_contract = SolangContract::init(stringify!($contract));

        $group .bench_function("solang", |b| {
            let args = solang_contract.call_args(stringify!($message), $args);

            b.iter(|| {
                solang_contract
                    .drink_api
                    .call(args.clone())
                    .unwrap();
            })
        });
    };
}

macro_rules! bench_evm {
    ( $group:ident, $contract:ident, $message:ident, $args:tt) => {
        let mut evm_contract = EvmContract::init(stringify!($contract));

        $group .bench_function("evm", |b| {
            let args = evm_contract.call_args(stringify!($message), &$args);

            b.iter(|| {
                evm_contract
                    .sandbox
                    .call(args.clone())
                    .unwrap();
            })
        });
    };
}

fn triangle_number(c: &mut Criterion) {
    let n = 100_000i64;

    let mut group = c.benchmark_group(format!("triangle_number_{}", n));

    bench_solang!(group, Computation, triangle_number, n);
    bench_evm!(group, Computation, triangle_number, [DynSolValue::Int(I256::try_from(n).unwrap(), 64)]);

    group.finish()
}

fn odd_product(c: &mut Criterion) {
    let n = 100_000i32;

    let mut group = c.benchmark_group(format!("odd_product_{}", n));
    group.sample_size(30);

    bench_solang!(group, Computation, odd_product, n);
    bench_evm!(group, Computation, odd_product, [DynSolValue::Int(I256::try_from(n).unwrap(), 32)]);

    group.finish()
}

criterion_group!(benches, odd_product, triangle_number);
criterion_main!(benches);
