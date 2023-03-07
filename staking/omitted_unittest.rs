    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;
        use crate::staking;
        use ink_env::DefaultEnvironment as Environment;
        
        
        fn set_sender(sender: AccountId) {
            ink_env::test::set_caller::<Environment>(sender);
        }

        fn default_accounts() -> ink_env::test::DefaultAccounts<Environment> {
            ink_env::test::default_accounts::<Environment>()
        }

        fn contract_id() -> AccountId {
            ink_env::test::callee::<Environment>()
        }

        fn alice() -> AccountId {
            default_accounts().alice
        }

        fn bob() -> AccountId {
            default_accounts().bob
        }

        fn charlie() -> AccountId {
            default_accounts().charlie
        }
        
        fn set_balance(account_id: AccountId, balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                account_id, balance,
            )
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }
        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            
            let mut s;
            let mut ps1 = Person{
                id:1,
                name: "kch".to_string(),
                phone: 34,
            };
            // ps1.hash(&mut s);
            let Staking = Staking::new(Hasher("0x87e8c3e6c107a1c554ed8cf8b599b12aac428d5d83bad916b26eb0107487c2a9"));
            assert_eq!(Staking.staking_time, 86400 * 5);
            assert_eq!(Staking.block_time, 5);
        }

        // /// We test if the default constructor does its job.
        // #[ink::test]
        // fn stake_works() {
        //     let mut contract = Staking::new(FAUCET_HASH);
        //     let owner = alice();
        //     let operator = bob();
        //     set_sender(operator);
        //     set_balance(operator, 100);
        //     contract.stake(50);
        //     assert_eq!(get_balance(operator), 50);
        // }
    }
