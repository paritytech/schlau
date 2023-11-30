use drink::{
    frame_support::traits::fungible::Inspect,
    pallet_balances, pallet_contracts,
    runtime::{AccountIdFor, Runtime as RuntimeT},
    BalanceOf, Sandbox, DEFAULT_GAS_LIMIT,
};
use ink_env::{
    call::{
        utils::{ReturnType, Set, Unset},
        CreateBuilder, ExecutionInput,
    },
    Environment,
};
use parity_scale_codec::Encode;
use std::marker::PhantomData;
use subxt_signer::sr25519::{dev, Keypair};

type ContractsBalanceOf<R> =
    <<R as pallet_contracts::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;

pub struct DrinkApi<AccountId, E: Environment, Runtime: RuntimeT> {
    sandbox: Sandbox<Runtime>,
    _phantom: PhantomData<(AccountId, E)>,
}

impl<AccountId, E, Runtime> DrinkApi<AccountId, E, Runtime>
where
    E: Environment<AccountId = AccountId, Balance = ContractsBalanceOf<Runtime>>,
    AccountId: Clone + Send + Sync + From<[u8; 32]> + AsRef<[u8; 32]>,
    E::Hash: Copy + From<[u8; 32]>,
    Runtime: RuntimeT + pallet_balances::Config + pallet_contracts::Config,
    AccountIdFor<Runtime>: From<[u8; 32]>,
    BalanceOf<Runtime>: From<u128>,
{
    pub fn new() -> Self {
        let mut sandbox = Sandbox::new().expect("Failed to initialize Drink! sandbox");
        Self::fund_accounts(&mut sandbox);
        DrinkApi {
            sandbox,
            _phantom: PhantomData,
        }
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

    pub fn deploy_contract<Contract, Args, R>(
        &mut self,
        code: Vec<u8>,
        value: ContractsBalanceOf<Runtime>,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R>,
        salt: Vec<u8>,
        caller: &Keypair,
        storage_deposit_limit: Option<ContractsBalanceOf<Runtime>>,
    ) -> anyhow::Result<()>
    where
        Contract: Clone,
        Args: Encode + Clone,
    {
        let data = constructor_exec_input(constructor.clone());
        let result = self.sandbox.deploy_contract(
            code,
            value,
            data,
            salt,
            keypair_to_account(caller),
            DEFAULT_GAS_LIMIT,
            storage_deposit_limit,
        );
        match result.result {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Failed to deploy contract: {:?}", e)),
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
pub fn constructor_exec_input<E, ContractRef, Args, R>(
    builder: CreateBuilderPartial<E, ContractRef, Args, R>,
) -> Vec<u8>
where
    E: Environment,
    Args: Encode,
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

fn keypair_to_account<AccountId: From<[u8; 32]>>(keypair: &Keypair) -> AccountId {
    AccountId::from(keypair.public_key().0)
}
