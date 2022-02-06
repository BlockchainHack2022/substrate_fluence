#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use ed25519_dalek::{PublicKey, Signature, Verifier};

    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    //use frame_support::inherent::Vec;
    use frame_support::{
        inherent::Vec,
        sp_runtime::traits::Hash,
        traits::{ Randomness, Currency, tokens::ExistenceRequirement },
        transactional
    };
    use sp_io::hashing::blake2_128;
    use scale_info::TypeInfo;

    #[cfg(feature = "std")]
    use frame_support::serde::{Deserialize, Serialize};

    type AccountOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // Struct for holding Kitty information.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Kitty<T: Config> {
        pub dna: [u8; 16],   // Using 16 bytes to represent a kitty DNA
        pub price: Option<BalanceOf<T>>,
        pub gender: Gender,
        pub owner: AccountOf<T>,
    }

    // Set Gender type in Kitty struct.
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum Gender {
        Male,
        Female,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The Currency handler for the Kitties pallet.
        type Currency: Currency<Self::AccountId>;

        /// The maximum amount of Kitties a single account can own.
        #[pallet::constant]
        type MaxKittyOwned: Get<u32>;

        /// The type of Randomness we want to specify for this pallet.
        type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
    }

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        /// Handles arithemtic overflow when incrementing the Kitty counter.
        KittyCntOverflow,
        /// An account cannot own more Kitties than `MaxKittyCount`.
        ExceedMaxKittyOwned,
        /// Buyer cannot be the owner.
        BuyerIsKittyOwner,
        /// Cannot transfer a kitty to its owner.
        TransferToSelf,
        /// Handles checking whether the Kitty exists.
        KittyNotExist,
        /// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.
        NotKittyOwner,
        /// Ensures the Kitty is for sale.
        KittyNotForSale,
        /// Ensures that the buying price is greater than the asking price.
        KittyBidPriceTooLow,
        /// Ensures that an account has enough funds to make a deposit.
        NotEnoughBalance,
        SignatureNotValid,
    }

    // Events.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Deposited(T::AccountId, Vec<u8>, BalanceOf<T>),
        /// A new Kitty was successfully created. \[sender, kitty_id\]
        Created(T::AccountId, T::Hash),
        /// Kitty price was successfully set. \[sender, kitty_id, new_price\]
        PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),
        /// A Kitty was successfully transferred. \[from, to, kitty_id\]
        Transferred(T::AccountId, T::AccountId, T::Hash),
        /// A Kitty was successfully bought. \[buyer, seller, kitty_id, bid_price\]
        Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
    }

    // Storage items.

    #[pallet::storage]
    #[pallet::getter(fn deposits)]
    /// Stores a Kitty's unique traits, owner and price.
    pub(super) type Deposits<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, BalanceOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn kitty_cnt)]
    /// Keeps track of the number of Kitties in existence.
    pub(super) type KittyCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    /// Stores a Kitty's unique traits, owner and price.
    pub(super) type Kitties<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Kitty<T>>;

    #[pallet::storage]
    #[pallet::getter(fn kitties_owned)]
    /// Keeps track of what accounts own what Kitty.
    pub(super) type KittiesOwned<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<T::Hash, T::MaxKittyOwned>, ValueQuery>;

    // ACTION #11: Our pallet's genesis configuration.

    // Our pallet's genesis configuration.
    #[pallet::genesis_config]
    pub struct GenesisConfig {}

    // Required to implement default for GenesisConfig.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {}


    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn make_deposit(
            origin: originfor<t>,
            client_id: vec<u8>,
            amount: balanceof<t>
            ) -> dispatchresult {

            let account = ensure_signed(origin)?;
            ensure!(t::currency::free_balance(&account) >= amount, <error<t>>::notenoughbalance); 

            //t::currency::transfer(&account, &node_owner, amount, existencerequirement::keepalive)?;

            <deposits<t>>::insert(&client_id, amount);
            ok(())
        }

        #[pallet::weight(10_000)]
        pub fn claim_reward(origin: originfor<t>) -> dispatchresult {
            ensure!(self::validate_signature().is_ok(), <error<t>>::signaturenotvalid);
            ok(())
        }
    }

    

    //** Our helper functions.**//

    impl<T: Config> Pallet<T> {

        fn validate_signature() -> Result<(), ed25519::Error> {
            let message: &[u8] = &[1, 2, 3, 4, 5];
            let public_key_bytes: [u8; 32] = [
                64, 62, 214, 61, 58, 61, 30, 176,
                50, 167, 89, 40, 14, 74, 55, 162,
                180, 239, 145, 111, 62, 251, 55, 20,
                244, 147, 168, 212, 182, 184, 143, 123
            ];
            let public_key: PublicKey = PublicKey::from_bytes(&public_key_bytes)?;

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
            let signature: Signature = Signature::from_bytes(&signature_bytes)?;

            public_key.verify(message, &signature)
        }
    }
}
