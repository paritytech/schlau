use super::drink::{runtime::AccountIdFor, BalanceOf};
use fp_evm::{CreateInfo, ExitReason};
use frame_support::traits::fungible::Mutate;
use pallet_evm::Runner;
use sp_core::{H160, H256, U256};
use sp_io::TestExternalities;

pub struct EvmSandbox<R> {
    externalities: TestExternalities,
    phantom: std::marker::PhantomData<R>,
}

impl<R> EvmSandbox<R>
where
    R: pallet_evm::Config + pallet_balances::Config,
    AccountIdFor<R>: From<H160> + Into<H160>,
{
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
            .map_err(|_err| anyhow::anyhow!("error invoking create"))?;

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
            .map_err(|_err| anyhow::anyhow!("error invoking call"))?;
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
}

pub struct CreateArgs {
    source: H160,
    init: Vec<u8>,
    value: U256,
    gas_limit: u64,
    max_fee_per_gas: U256,
    max_priority_fee_per_gas: Option<U256>,
    nonce: Option<U256>,
    access_list: Vec<(H160, Vec<H256>)>,
}

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
