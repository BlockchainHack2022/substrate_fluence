#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    //use ed25519_dalek::PublicKey;
    //use ed25519_dalek::Signature;
    //use ed25519_dalek::Verifier;

    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::vec::Vec;

    use frame_support::{
        sp_runtime::traits::Hash,
        traits::{ Randomness, Currency, tokens::ExistenceRequirement },
        transactional
    };
    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type Address = Vec<u8>;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The Currency handler for the Kitties pallet.
        type Currency: Currency<Self::AccountId>;

    }

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        SignatureNotValid,
        NotEnoughBalance,
    }

    // Events.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Deposit(Vec<u8>, BalanceOf<T>),
    }

    #[pallet::storage]
    #[pallet::getter(fn deposits)]
    pub(super) type Deposits<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, BalanceOf<T>>;


    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight(10_000)]
        pub fn make_deposit(
            origin: OriginFor<T>,
            client_id: Vec<u8>,
            amount: BalanceOf<T>
            ) -> DispatchResult {

            let account = ensure_signed(origin)?;
            ensure!(T::Currency::free_balance(&account) >= amount, <Error<T>>::NotEnoughBalance); 

            //t::currency::transfer(&account, &node_owner, amount, existencerequirement::keepalive)?;

            <Deposits<T>>::insert(&client_id, amount);
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn claim_reward(origin: OriginFor<T>) -> DispatchResult {
            ensure!(Self::validate_signature().is_ok(), <Error<T>>::SignatureNotValid);
            Ok(())
        }
    }



    impl<T: Config> Pallet<T> {
        pub fn validate_signature() -> Result<(), Error<T>> {
            let message: &[u8] = &[1, 2, 3, 4, 5];
            let public_key_bytes: [u8; 32] = [
                64, 62, 214, 61, 58, 61, 30, 176,
                50, 167, 89, 40, 14, 74, 55, 162,
                180, 239, 145, 111, 62, 251, 55, 20,
                244, 147, 168, 212, 182, 184, 143, 123
            ];
            //let public_key: PublicKey = PublicKey::from_bytes(&public_key_bytes)?;

            let signature_bytes: [u8; 64] = [
                176, 149, 248, 112, 26, 7, 30, 203,
                255, 227, 148, 63, 19, 230, 136, 205,
                93, 114, 0, 103, 101, 67, 84, 120,
                3, 52, 141, 23, 252, 198, 208, 169,
                209, 134, 164, 172, 214, 131, 120, 63,
                105, 92, 76, 45, 155, 50, 196, 227,
                151, 241, 74, 77, 131, 104, 115, 22,
                220, 133, 41, 227, 4, 249, 66, 13
            ];
            Ok(())
            //let signature: Signature = Signature::from_bytes(&signature_bytes)?;

            //public_key.verify(message, &signature)
        }
    }
}
