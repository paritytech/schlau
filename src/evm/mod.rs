mod runtime;

use fp_evm::{CreateInfo, ExitReason};
use frame_support::sp_runtime;
use frame_support::traits::fungible::Mutate;
use frame_system::GenesisConfig;
use pallet_evm::Runner;
use sp_core::{H160, H256, U256};
use sp_io::TestExternalities;
use sp_runtime::BuildStorage;

pub use runtime::EvmRuntime;

pub type AccountIdFor<R> = <R as frame_system::Config>::AccountId;
pub type BalanceOf<R> = <R as pallet_balances::Config>::Balance;

pub const DEFAULT_ACCOUNT: H160 = H160::repeat_byte(1);

pub struct EvmSandbox<R = EvmRuntime> {
    externalities: TestExternalities,
    phantom: std::marker::PhantomData<R>,
}

impl<R> EvmSandbox<R>
where
    R: pallet_evm::Config + pallet_balances::Config,
    AccountIdFor<R>: From<H160> + Into<H160>,
    BalanceOf<R>: From<u64>,
{
    pub fn new() -> Self {
        let mut storage = GenesisConfig::<R>::default()
            .build_storage()
            .expect("error building storage");

        // initialize the balance of the default account
        pallet_balances::GenesisConfig::<R> {
            balances: vec![(
                AccountIdFor::<R>::from(DEFAULT_ACCOUNT),
                BalanceOf::<R>::from(u64::MAX),
            )],
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        let sandbox = Self {
            externalities: TestExternalities::new(storage),
            phantom: Default::default(),
        };

        // sandbox
        //     .externalities
        //     // We start the chain from the 1st block, so that events are collected (they are not
        //     // recorded for the genesis block...).
        //     .execute_with(|| R::initialize_block(BlockNumberFor::<R>::one(), Default::default()))
        //     .expect("Error initializing block");

        sandbox
    }

    pub fn execute_with<T>(&mut self, execute: impl FnOnce() -> T) -> T {
        self.externalities.execute_with(execute)
    }

    pub fn create(&mut self, create_args: CreateArgs) -> anyhow::Result<H160> {
        let CreateArgs {
            source,
            init,
            value,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
        } = create_args;
        self.execute_with(|| {
            let is_transactional = true;
            let validate = true;
            let CreateInfo {
                exit_reason,
                value: create_address,
                ..
            } = R::Runner::create(
                source,
                init,
                value,
                gas_limit,
                Some(max_fee_per_gas),
                max_priority_fee_per_gas,
                nonce,
                access_list,
                is_transactional,
                validate,
                None,
                None,
                R::config(),
            )
            .map_err(|err| {
                let err: sp_runtime::DispatchError = err.error.into();
                let ser_err = serde_json::to_string_pretty(&err).unwrap();
                anyhow::anyhow!("error invoking create: {}", ser_err)
            })?;

            if let ExitReason::Succeed(_) = exit_reason {
                Ok(create_address)
            } else {
                Err(anyhow::anyhow!("create failed: {:?}", exit_reason))
            }
        })
    }

    pub fn call(&mut self, call_args: CallArgs) -> anyhow::Result<()> {
        let CallArgs {
            source,
            target,
            input,
            value,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
        } = call_args;
        self.execute_with(|| {
            let is_transactional = true;
            let validate = true;
            let info = R::Runner::call(
                source,
                target,
                input,
                value,
                gas_limit,
                Some(max_fee_per_gas),
                max_priority_fee_per_gas,
                nonce,
                access_list,
                is_transactional,
                validate,
                None,
                None,
                R::config(),
            )
            .map_err(|err| {
                let err: sp_runtime::DispatchError = err.error.into();
                let ser_err = serde_json::to_string_pretty(&err).unwrap();
                anyhow::anyhow!("error invoking call: {}", ser_err)
            })?;
            if let ExitReason::Succeed(_) = info.exit_reason {
                Ok(())
            } else {
                Err(anyhow::anyhow!("call failed: {:?}", info.exit_reason))
            }
        })
    }

    pub fn mint_into(
        &mut self,
        address: H160,
        amount: BalanceOf<R>,
    ) -> anyhow::Result<BalanceOf<R>> {
        let address = AccountIdFor::<R>::from(address);
        self.execute_with(|| pallet_balances::Pallet::<R>::mint_into(&address, amount))
            .map_err(|_err| anyhow::anyhow!("error minting into account"))
    }

    /// Return the free balance of an account.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the account to query.
    pub fn free_balance(&mut self, address: H160) -> BalanceOf<R> {
        let address = AccountIdFor::<R>::from(address);
        self.execute_with(|| pallet_balances::Pallet::<R>::free_balance(&address))
    }
}

#[derive(Default)]
pub struct CreateArgs {
    pub source: H160,
    pub init: Vec<u8>,
    pub value: U256,
    pub gas_limit: u64,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: Option<U256>,
    pub nonce: Option<U256>,
    pub access_list: Vec<(H160, Vec<H256>)>,
}

#[derive(Default)]

pub struct CallArgs {
    source: H160,
    target: H160,
    input: Vec<u8>,
    value: U256,
    gas_limit: u64,
    max_fee_per_gas: U256,
    max_priority_fee_per_gas: Option<U256>,
    nonce: Option<U256>,
    access_list: Vec<(H160, Vec<H256>)>,
}
