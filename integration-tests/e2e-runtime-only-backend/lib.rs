#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod flipper {
    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        /// Flips the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Returns the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }

        /// Returns the current balance of the Flipper.
        #[ink(message)]
        pub fn get_contract_balance(&self) -> Balance {
            self.env().balance()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::env::DefaultEnvironment;
        use ink_e2e::{
            subxt::dynamic::Value,
            ChainBackend,
            ContractsBackend,
            E2EBackend,
            InstantiationResult,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// Deploys the flipper contract with `initial_value` and returns the contract
        /// instantiation result.
        ///
        /// Uses `ink_e2e::alice()` as the caller.
        async fn deploy<Client: E2EBackend>(
            client: &mut Client,
            initial_value: bool,
        ) -> Result<
            InstantiationResult<
                DefaultEnvironment,
                <Client as ContractsBackend<DefaultEnvironment>>::EventLog,
            >,
            <Client as ContractsBackend<DefaultEnvironment>>::Error,
        > {
            let constructor = FlipperRef::new(initial_value);
            client
                .instantiate(
                    "e2e-runtime-only-backend",
                    &ink_e2e::alice(),
                    constructor,
                    0,
                    None,
                )
                .await
        }

        /// Tests standard flipper scenario:
        /// - deploy the flipper contract with initial value `false`
        /// - flip the flipper
        /// - get the flipper's value
        /// - assert that the value is `true`
        #[ink_e2e::test(backend = "runtime-only")]
        async fn it_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            const INITIAL_VALUE: bool = false;
            let contract = deploy(&mut client, INITIAL_VALUE)
                .await
                .expect("deploy failed");

            // when
            let mut call = contract.call::<Flipper>();
            let _flip_res = client
                .call(&ink_e2e::bob(), &call.flip(), 0, None)
                .await
                .expect("flip failed");

            // then
            let get_res = client
                .call(&ink_e2e::bob(), &call.get(), 0, None)
                .await
                .expect("get failed");
            assert_eq!(get_res.return_value(), !INITIAL_VALUE);

            Ok(())
        }

        /// Tests runtime call scenario:
        /// - deploy the flipper contract
        /// - get the contract's balance
        /// - transfer some funds to the contract using runtime call
        /// - get the contract's balance again
        /// - assert that the contract's balance increased by the transferred amount
        #[ink_e2e::test(backend = "runtime-only")]
        async fn runtime_call_works() -> E2EResult<()> {
            // given
            let contract = deploy(&mut client, false).await.expect("deploy failed");
            let call = contract.call::<Flipper>();

            let old_balance = client
                .call(&ink_e2e::alice(), &call.get_contract_balance(), 0, None)
                .await
                .expect("get_contract_balance failed")
                .return_value();

            const ENDOWMENT: u128 = 10;

            // when
            let call_data = vec![
                Value::from_bytes(&contract.account_id),
                Value::u128(ENDOWMENT),
            ];
            client
                .runtime_call(&ink_e2e::alice(), "Balances", "transfer", call_data)
                .await
                .expect("runtime call failed");

            // then
            let new_balance = client
                .call(&ink_e2e::alice(), &call.get_contract_balance(), 0, None)
                .await
                .expect("get_contract_balance failed")
                .return_value();

            assert_eq!(old_balance + ENDOWMENT, new_balance);
            Ok(())
        }

        /// Just instantiate a contract using non-default runtime.
        #[ink_e2e::test(backend = "runtime-only", runtime = ink_e2e::MinimalRuntime)]
        async fn custom_runtime<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            client
                .instantiate(
                    "e2e-runtime-only-backend",
                    &ink_e2e::alice(),
                    FlipperRef::new(false),
                    0,
                    None,
                )
                .await
                .expect("instantiate failed");

            Ok(())
        }
    }
}
