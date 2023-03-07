#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod staking {
    use erc20::Erc20Ref;
    use ink_env;
    use ink_env::call::FromAccountId;
    use ink_prelude::{
        // string::ToString,
        vec,
        vec::Vec,
    };
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        traits::{
            // SpreadAllocate,
            PackedLayout,
            SpreadLayout,
        },
    };

    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    // #[derive(SpreadAllocate)]
    #[ink(storage)]
    pub struct Staking {
        staked: StorageHashMap<AccountId, Vec<Stake>>,
        unstaked: StorageHashMap<AccountId, Vec<Balance>>,
        token: Erc20Ref,
        sig_status: u128, //////////////////////////////
    }

    /// Staking data per wallet
    ///
    /// # Note
    /// This struct is based on std module so that
    /// we can embed into `staked` data described above.
    /// Otherwise, might raise ERR.
    #[derive(
        Copy,
        Clone,
        Debug,
        Ord,
        PartialOrd,
        Eq,
        PartialEq,
        Default,
        PackedLayout,
        SpreadLayout,
        scale::Encode,
        scale::Decode,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Stake {
        amount: Balance,
        timestamp: Balance,
    }

    impl Staking {
        /// @dev    Default Initialization.
        /// @param  address of pre-deployed ERC20 contract.
        ///         this is available only after the deployment of ERC20 contract.
        /// @note   Initialize the contract with pre-deployed erc20 instance address

        #[ink(constructor)]
        pub fn new(_erc20_account_id: AccountId) -> Self {
            //
            // let address : AccountId = AccountId::decode(&mut ref_account32).unwrap_or_default();
            let erc20_instance = Erc20Ref::from_account_id(_erc20_account_id);
            Self {
                staked: StorageHashMap::new(),
                unstaked: StorageHashMap::new(),
                token: erc20_instance,
                sig_status: 0, ////////////////////////////
            }
        }

        /// @dev     Method #1 (WRITE)
        /// @param   _amount:Balance
        /// @note    register/update caller's staking data, and stake ERC20 token.
        #[ink(message)]
        pub fn stake(&mut self, _amount: Balance) {
            let caller = self.env().caller();
            let me = self.env().account_id();
            let current_block_timestamp: Balance = self.env().block_timestamp().into();
            if self.token.balance_of(caller) < _amount {
                ink_env::debug_println!("{}", "Insufficient funds");
                return;
            }
            // Rigister/update caller's staking data.
            if self.staked.contains_key(&caller) {
                let mut _staked = self.staked.get_mut(&caller).unwrap();
                _staked.push(Stake {
                    timestamp: current_block_timestamp,
                    amount: _amount,
                });
            } else {
                self.staked.insert(
                    caller,
                    vec![Stake {
                        timestamp: current_block_timestamp,
                        amount: _amount,
                    }],
                );
            }
            // Register/update caller's unstaking data.
            if self.unstaked.contains_key(&caller) {
                self.unstaked.get_mut(&caller).unwrap().push(0);
            } else {
                self.unstaked.insert(caller, vec![0]);
            }
            // Transfer ERC20 token to this contract.

            self.transfer_with_signature(caller, me, _amount);
        }

        /// @dev       Method #2 (READ)
        /// @param     
        /// @note      Stake up to 5 days. Each day within 5 has 10% increament than the day before.
        #[ink(message)]
        pub fn get_unstakable(&self, _start: Balance) -> Balance {
            if u128::from(self.env().block_timestamp()) < _start {
                return 0;
            }
            let times: Balance = u128::from(self.env().block_timestamp()) - _start;
            let days: Balance = times / 86400_000;
            match days {
                0 => 0,
                1 => 5,
                2 => 6,
                3 => 7,
                4 => 8,
                5 => 9,
                _ => 10,
            }
        }

        /// @dev     Method #3 (READ)
        /// @param   addr: AccountId
        /// @return  Total balance of _addr's ERC20 token.
        #[ink(message)]
        pub fn get_balance(&self, _addr: AccountId) -> Balance {
            let mut balance: Balance = 0;
            let length = self.staked.get(&_addr).unwrap().len();
            (0..length).for_each(|i| {
                let staked_time: Balance = self.staked.get(&_addr).unwrap()[i].timestamp;
                let staked_amount: Balance = self.staked.get(&_addr).unwrap()[i].amount;
                balance = balance + self.get_unstakable(staked_time) * staked_amount / 10
                    - self.unstaked.get(&_addr).unwrap()[i];
            });
            return balance;
        }

        /// @dev     Method #3 (READ)
        /// @param   addr: AccountId
        /// @return  Total balance of _addr's ERC20 token.
        #[ink(message)]
        pub fn get_erc20_totalsupply(&self) -> Balance {
            return self.token.total_supply();
        }

        /// @dev     Method #3 (READ)
        /// @param   addr: AccountId
        /// @return  Total balance of _addr's ERC20 token.
        #[ink(message)]
        pub fn get_erc20_balance(&self, _addr: AccountId) -> Balance {
            return self.token.balance_of(_addr);
        }

        /// @dev     Method #3 (READ)
        /// @param   addr: AccountId
        /// @return  Total balance of _addr's ERC20 token.
        #[ink(message)]
        pub fn get_staked_timestamp(&self, _addr: AccountId, _index: Balance) -> Balance {
            return self.staked.get(&_addr).unwrap()
                [TryInto::<usize>::try_into(_index).ok().unwrap()]
            .timestamp;
        }

        /// @dev     Method #3 (READ)
        /// @param   addr: AccountId
        /// @return  Total balance of _addr's ERC20 token.
        #[ink(message)]
        pub fn get_staked_amount(&self, _addr: AccountId, _index: Balance) -> Balance {
            return self.staked.get(&_addr).unwrap()
                [TryInto::<usize>::try_into(_index).ok().unwrap()]
            .amount;
        }

        /// @dev     Method #3 (READ)
        /// @param   addr: AccountId
        /// @return  Total balance of _addr's ERC20 token.
        #[ink(message)]
        pub fn get_sig_status(&self) -> u128 {
            self.sig_status
        }

        /// @dev     Method #4 (WRITE)
        /// @param   _amount: Balance
        /// @note    TL;DR : "Inline comment will help you."
        #[ink(message)]
        pub fn claim(&mut self, _amount: Balance) {
            let caller = self.env().caller();
            let me = self.env().account_id();
            if self.get_balance(caller) < _amount {
                ink_env::debug_println!("{}", "Exceeds current unstakable");
                return;
            }
            let mut unstakable: Balance;
            let mut length = self.staked.get(&caller).unwrap().len();
            let _claim_amount = _amount;
            let mut amount = _amount.clone();
            let mut i = 0;

            // Looping through storage, sum up unstakable balance and update storage.
            // Finally transfer ERC20 token to caller.
            loop {
                if !(i < length && amount > 0) {
                    break;
                }
                unstakable = (self
                    .get_unstakable(self.staked.get(&caller).unwrap()[i].timestamp)
                    * self.staked.get(&caller).unwrap()[i].amount)
                    / 10
                    - self.unstaked.get(&caller).unwrap()[i];
                if unstakable > amount {
                    self.unstaked.get_mut(&caller).unwrap()[i] += amount;
                    amount = 0;
                } else {
                    self.unstaked.get_mut(&caller).unwrap()[i] += unstakable;
                    if self.staked.get(&caller).unwrap()[i].amount
                        == self.unstaked.get(&caller).unwrap()[i]
                    {
                        length -= 1;
                        self.staked.get_mut(&caller).unwrap().remove(i);
                        self.unstaked.get_mut(&caller).unwrap().remove(i);
                    } else {
                        i += 1;
                    }
                    amount -= unstakable;
                }
            }
            self.transfer_with_signature(me, caller, _claim_amount);
        }

        /// @dev     Method #5 (WRITE)
        /// @note    unstake all tokens.
        ///          This method is similar to claim()
        #[ink(message)]
        pub fn claim_all(&mut self) {
            let caller = self.env().caller();
            let me = self.env().account_id();
            let balance: Balance = self.get_balance(caller);
            if balance <= 0 {
                ink_env::debug_println!("{}", "No token to be staked");
                return;
            }
            let mut i = 0;
            let mut _length = self.staked.get(&caller).unwrap().len();
            let mut unstakable: Balance;
            loop {
                if i >= _length {
                    break;
                }
                unstakable = (self
                    .get_unstakable(self.staked.get(&caller).unwrap()[i].timestamp)
                    * self.staked.get(&caller).unwrap()[i].amount)
                    / 10
                    - self.unstaked.get(&caller).unwrap()[i];
                self.unstaked.get_mut(&caller).unwrap()[i] += unstakable;
                if self.staked.get(&caller).unwrap()[i].amount
                    == self.unstaked.get(&caller).unwrap()[i]
                {
                    _length -= 1;
                    self.staked.get_mut(&caller).unwrap().remove(i);
                    self.unstaked.get_mut(&caller).unwrap().remove(i);
                } else {
                    i += 1;
                }
            }
            self.transfer_with_signature(me, caller, balance);
        }

        // EIP-2612: Digital Signature Algorithm
        // This makes Transfer with signature of owner.
        fn transfer_with_signature(
            &mut self,
            from: AccountId,
            to: AccountId,
            balance: Balance,
        ) {
            // Make hash
            let deadline = self.env().block_timestamp() + 86400000;
            let nonce = self.token.nonce();
            let encodable = (from, to, balance, deadline, nonce); // Implements `scale::Encode`
            use ink_env::hash::{HashOutput, Keccak256};
            let mut hash_out = <Keccak256 as HashOutput>::Type::default(); // 256-bit buffer
            ink_env::hash_encoded::<Keccak256, _>(&encodable, &mut hash_out);
            let mut sig = [0u8; 65];
            self.sig_status += 1; ///////////////////////////////////////////
                                  // Make signature from ownerId
            #[cfg(all(feature = "std", feature = "rand-std"))]
            {
                let secp = Secp256k1::new();
                let secret_key = SecretKey::from_slice(&[0xcd; 32])
                    .expect("32 bytes, within curve order");
                let public_key = PublicKey::from_secret_key(&secp, &secret_key);
                // This is unsafe unless the supplied byte slice is the output of a cryptographic hash function.
                // See the above example for how to use this library together with `bitcoin_hashes`.
                let message = Message::from_slice(&hash_out).expect("32 bytes");

                sig = secp.sign_ecdsa(&message, &secret_key);
                assert!(secp.verify_ecdsa(&message, &sig, &public_key).is_ok());
                // self.sig_status += 1; // This code doesn't work cozOf block. So don use self.????
            }
            self.token.transfer_with_signature(
                from, to, balance, deadline, nonce, sig, hash_out,
            );
        }
    }

    // Odded out Unit Test.
    // module and test functions are marked with a `#[test]` attribute.
    // The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        use crate::staking::Staking;
        use ink_env::DefaultEnvironment as Environment;
        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

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
            use ink_env::hash::{HashOutput, Sha2x256};
            let input: &[u8] = &[13, 14, 15];
            let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
            let hash = ink_env::hash_bytes::<Sha2x256>(input, &mut output);
            // acNMiQfPMbgTyM1ABJiXFiE7WnB2pitAV7TpJScmPAVB5Sr
            const erc20_hash: [u8; 32] = [
                0x7c, 0x96, 0x32, 0x97, 0xc5, 0xb0, 0x80, 0x7c, 0xe4, 0x13, 0xa5, 0xeb,
                0x8d, 0x87, 0x77, 0x9b, 0x10, 0x5a, 0xd2, 0x50, 0xeb, 0xd1, 0x8e, 0x21,
                0x65, 0xe8, 0xb5, 0x6c, 0xbd, 0x5f, 0x67, 0xbf,
            ];
            ink_env::debug_println!("{:?}", erc20_hash);
            let staking = Staking::new_init(erc20_hash.into());
        }
    }
}
