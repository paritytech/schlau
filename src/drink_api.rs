use super::drink::{
    frame_support::traits::fungible::Inspect,
    pallet_balances, pallet_contracts,
    runtime::{AccountIdFor, Runtime as RuntimeT},
    BalanceOf, Sandbox, Weight, DEFAULT_GAS_LIMIT,
};
use subxt_signer::sr25519::{dev, Keypair};

pub type ContractsBalanceOf<R> =
    <<R as pallet_contracts::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;

pub struct DrinkApi<Runtime: RuntimeT> {
    sandbox: Sandbox<Runtime>,
}

impl<Runtime> DrinkApi<Runtime>
where
    Runtime: RuntimeT + pallet_balances::Config + pallet_contracts::Config,
    AccountIdFor<Runtime>: From<[u8; 32]> + AsRef<[u8; 32]>,
    BalanceOf<Runtime>: From<u128>,
{
    pub fn new() -> Self {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .with_test_writer()
            .try_init()
            .ok();

        let mut sandbox = Sandbox::new().expect("Failed to initialize Drink! sandbox");
        Self::fund_accounts(&mut sandbox);
        DrinkApi { sandbox }
    }

    pub fn fund_accounts(sandbox: &mut Sandbox<Runtime>) {
        const TOKENS: u128 = 1_000_000_000_000_000;

        let accounts = [
            dev::alice(),
            dev::bob(),
            dev::charlie(),
            dev::dave(),
            dev::eve(),
            dev::ferdie(),
            dev::one(),
            dev::two(),
        ]
        .map(|kp| kp.public_key().0)
        .map(From::from);
        for account in accounts.into_iter() {
            sandbox
                .mint_into(account, TOKENS.into())
                .unwrap_or_else(|_| panic!("Failed to mint {} tokens", TOKENS));
        }
    }

    pub fn instantiate_with_code(
        &mut self,
        create_args: CreateArgs<Runtime>,
    ) -> anyhow::Result<AccountIdFor<Runtime>> {
        let CreateArgs {
            code,
            value,
            data,
            salt,
            caller,
            storage_deposit_limit,
        } = create_args;
        let result = self.sandbox.deploy_contract(
            code,
            value,
            data,
            salt,
            caller,
            DEFAULT_GAS_LIMIT,
            storage_deposit_limit,
        );
        if result.debug_message.len() > 0 {
            tracing::debug!(
                "debug message {}",
                String::from_utf8_lossy(&result.debug_message)
            )
        }
        result
            .result
            .map(|r| r.account_id)
            .map_err(|e| anyhow::anyhow!("Failed to instantiate contract: {:?}", e))
    }

    pub fn call(&mut self, call_args: CallArgs<Runtime>) -> anyhow::Result<Vec<u8>> {
        let CallArgs {
            contract_account,
            caller,
            exec_input,
            value,
            gas_limit,
            storage_deposit_limit,
        } = call_args;
        let gas_limit = gas_limit.unwrap_or(DEFAULT_GAS_LIMIT);
        let result = self.sandbox.call_contract(
            contract_account,
            value,
            exec_input,
            caller,
            gas_limit,
            storage_deposit_limit,
            pallet_contracts::Determinism::Enforced,
        );
        if result.debug_message.len() > 0 {
            tracing::debug!(
                "debug message: {}",
                String::from_utf8_lossy(&result.debug_message)
            )
        }
        match result.result {
            Ok(result) => {
                if result.did_revert() {
                    tracing::error!("contract reverted with {:?}", result);
                    return Err(anyhow::anyhow!("Contract execution reverted"));
                }
                Ok(result.data)
            }
            Err(e) => Err(anyhow::anyhow!("Failed to call contract: {:?}", e)),
        }
    }
}

#[derive(Clone)]
pub struct CreateArgs<Runtime: RuntimeT + pallet_contracts::Config> {
    pub code: Vec<u8>,
    pub value: ContractsBalanceOf<Runtime>,
    pub data: Vec<u8>,
    pub salt: Vec<u8>,
    pub caller: AccountIdFor<Runtime>,
    pub storage_deposit_limit: Option<ContractsBalanceOf<Runtime>>,
}

impl<Runtime: RuntimeT + pallet_contracts::Config> CreateArgs<Runtime>
where
    AccountIdFor<Runtime>: From<[u8; 32]>,
    ContractsBalanceOf<Runtime>: From<u128>,
{
    pub fn new(code: Vec<u8>, caller: Keypair) -> Self {
        Self {
            code,
            value: ContractsBalanceOf::<Runtime>::from(0u128),
            data: Vec::new(),
            salt: Vec::new(),
            caller: keypair_to_account(&caller),
            storage_deposit_limit: None,
        }
    }
}

impl<Runtime: RuntimeT + pallet_contracts::Config> CreateArgs<Runtime> {
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }
}

#[derive(Clone)]
pub struct CallArgs<Runtime: RuntimeT + pallet_contracts::Config> {
    pub contract_account: AccountIdFor<Runtime>,
    pub caller: AccountIdFor<Runtime>,
    pub exec_input: Vec<u8>,
    pub value: ContractsBalanceOf<Runtime>,
    pub gas_limit: Option<Weight>,
    pub storage_deposit_limit: Option<ContractsBalanceOf<Runtime>>,
}

impl<Runtime: RuntimeT + pallet_contracts::Config> CallArgs<Runtime>
where
    AccountIdFor<Runtime>: From<[u8; 32]>,
{
    pub fn new(
        contract_account: AccountIdFor<Runtime>,
        caller: Keypair,
        exec_input: Vec<u8>,
    ) -> Self {
        Self {
            contract_account,
            caller: keypair_to_account(&caller),
            exec_input,
            value: Default::default(),
            storage_deposit_limit: None,
            gas_limit: None,
        }
    }

    pub fn with_value(mut self, value: ContractsBalanceOf<Runtime>) -> Self {
        self.value = value;
        self
    }

    pub fn with_storage_deposit_limit(
        mut self,
        storage_deposit_limit: ContractsBalanceOf<Runtime>,
    ) -> Self {
        self.storage_deposit_limit = Some(storage_deposit_limit);
        self
    }

    pub fn with_gas_limit(mut self, gas_limit: Weight) -> Self {
        self.gas_limit = Some(gas_limit);
        self
    }

    pub fn with_max_gas_limit(self) -> Self {
        self.with_gas_limit(Weight::from_parts(u64::MAX, u64::MAX))
    }
}

fn keypair_to_account<AccountId: From<[u8; 32]>>(keypair: &Keypair) -> AccountId {
    AccountId::from(keypair.public_key().0)
}
