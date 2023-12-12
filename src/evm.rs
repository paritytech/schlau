use fp_evm::{CreateInfo, ExitReason};
use pallet_evm::Runner;
use sp_core::{H160, H256, U256};
use sp_io::TestExternalities;

pub struct EvmSandbox<R> {
    externalities: TestExternalities,
    phantom: std::marker::PhantomData<R>,
}

impl<R: pallet_evm::Config> EvmSandbox<R> {
    pub fn execute_with<T>(&mut self, execute: impl FnOnce() -> T) -> T {
        self.externalities.execute_with(execute)
    }

    pub fn create(
        &mut self,
        source: H160,
        init: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>
    ) -> anyhow::Result<H160> {
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
            ).map_err(|err| anyhow::anyhow!("error invoking create"))?;

            if let ExitReason::Succeed(_) = exit_reason {
                Ok(create_address)
            } else {
                Err(anyhow::anyhow!("create failed: {:?}", exit_reason))
            }
        });
    }
}