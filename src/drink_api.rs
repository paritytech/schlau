use drink::{
    pallet_balances,
    pallet_contracts,
    runtime::Runtime as RuntimeT,
    Sandbox,
    DEFAULT_GAS_LIMIT,
};
use ink_env::{
    call::{
        utils::{
            ReturnType,
            Set,
            Unset,
        },
        CreateBuilder,
        ExecutionInput,
    },
    Environment,
};
use std::marker::PhantomData;
use subxt_signer::sr25519::dev;

pub struct DrinkApi<AccountId, Hash, Runtime: RuntimeT> {
    sandbox: Sandbox<Runtime>,
    _phantom: PhantomData<(AccountId, Hash)>,
}

impl<AccountId, Hash, Runtime> DrinkApi<AccountId, Hash, Runtime>
where
    AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
    Hash: Copy + From<[u8; 32]>,
    Runtime: RuntimeT + pallet_balances::Config + pallet_contracts::Config,
{
    pub fn new() -> Self {
        let mut sandbox = Sandbox::new().expect("Failed to initialize Drink! sandbox");
        Self::fund_accounts(&mut sandbox);
        DrinkApi { sandbox, _phantom: PhantomData }
    }

    fn fund_accounts(sandbox: &mut Sandbox<Runtime>) {
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

    pub fn deploy_contract(&mut self, code: Vec<u8>, value: u128, constructor: &mut CreateBuilderPartial<E, Contract, Args, R>, salt: Vec<u8>, caller: [u8; 32], gas_limit: u128, storage_deposit_limit: Option<u128>) -> Result<(), String> {
        let result = self.sandbox.deploy_contract(
            code,
            value,
            data,
            salt,
            caller.into(),
            DEFAULT_GAS_LIMIT,
            storage_deposit_limit,
        );
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// The type returned from `ContractRef` constructors, partially initialized with the
/// execution input arguments.
pub type CreateBuilderPartial<E, ContractRef, Args, R> = CreateBuilder<
    E,
    ContractRef,
    Unset<<E as Environment>::Hash>,
    Unset<u64>,
    Unset<<E as Environment>::Balance>,
    Set<ExecutionInput<Args>>,
    Unset<ink_env::call::state::Salt>,
    Set<ReturnType<R>>,
>;

/// Get the encoded constructor arguments from the partially initialized `CreateBuilder`
pub fn constructor_exec_input<E, ContractRef, Args: Encode, R>(
    builder: CreateBuilderPartial<E, ContractRef, Args, R>,
) -> Vec<u8>
    where
        E: Environment,
{
    // set all the other properties to default values, we only require the `exec_input`.
    builder
        .endowment(0u32.into())
        .code_hash(ink_primitives::Clear::CLEAR_HASH)
        .salt_bytes(Vec::new())
        .params()
        .exec_input()
        .encode()
}