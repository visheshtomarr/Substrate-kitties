#![cfg_attr(not(feature = "std"), no_std)]

mod impls;

use frame::prelude::*;
use frame::traits::fungible::Inspect;
use frame::traits::fungible::Mutate;
pub use pallet::*;

#[frame::pallet(dev_mode)]
pub mod pallet {
	use core::default;

	use frame_system::Config as OtherConfig;
	use frame_system::Key;

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(core::marker::PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The fungible handler for the kitties pallet.
		type NativeBalance: Inspect<Self::AccountId> + Mutate<Self::AccountId>;
	}

	/// Balance type for the kitties pallet.
	pub type BalanceOf<T> =
		<<T as Config>::NativeBalance as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

	/// Struct to represent a kitty.
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		// Using 32 bytes to represent a kitty DNA.
		pub dna: [u8; 32],
		pub owner: T::AccountId,
		// The Option<BalanceOf<T>> type denotes that if the price is 'None', the kitty is not for
		// sale. And if it is Some(price), it is for sale for that 'price'.
		pub price: Option<BalanceOf<T>>,
	}

	/// Storage to store count of kitties.
	#[pallet::storage]
	pub(super) type CountForKitties<T: Config> = StorageValue<Value = u32, QueryKind = ValueQuery>;

	/// Storage to store different kitties.
	#[pallet::storage]
	pub(super) type Kitties<T: Config> = StorageMap<Key = [u8; 32], Value = Kitty<T>>;

	/// Storage to store which kitties belongs to which owner.
	#[pallet::storage]
	pub(super) type KittiesOwned<T: Config> = StorageMap<
		Key = T::AccountId,
		// We will be using a 'BoundedVec' instead of normal vec to avoid storing too many owned
		// kitties.
		Value = BoundedVec<[u8; 32], ConstU32<100>>,
		QueryKind = ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// When a new kitty is minted.
		Created { owner: T::AccountId },
		/// When a kitty is successfully tranferred.
		Transferred { from: T::AccountId, to: T::AccountId, kitty_id: [u8; 32] },
		/// When a price is set for a kitty.
		PriceSet { owner: T::AccountId, kitty_id: [u8; 32], new_price: Option<BalanceOf<T>> },
		/// When kitty is sold to a buyer.
		Sold { buyer: T::AccountId, kitty_id: [u8; 32], price: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Overflow occurs while storing too many kitties.
		TooManyKitties,
		/// If a kitty is already present in storage.
		DuplicateKitty,
		/// If the number of owned kitties excceed 100.
		TooManyOwnedKitties,
		/// When 'from' and 'to' of the transfer is same.
		TransferToSelf,
		/// When kitty, which is being transferred, does not exist.
		KittyNotFound,
		/// When transfer is initiated by a false owner.
		NotOwner,
		/// When kitty has a price set to 'None'.
		NotForSale,
		/// When the price offered by the buyer is too low.
		MaxPriceTooLow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// "origin" is the first parameter to every callable function.
		// It describes where the call is calling from, and allows us to perform simple access
		// control logic based on that information.
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Create a default id to insert kitty into the map when we call mint().
			let dna = Self::gen_dna();
			Self::mint(who, dna)?;
			Ok(())
		}

		/// Extrinsic to implement transfer of a kitty.
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Call the internal function 'do_transfer()' to execute the transfer logic.
			Self::do_transfer(who, to, kitty_id)?;
			Ok(())
		}

		/// Extrinsic to set a price for a kitty.
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: [u8; 32],
			price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Call the internal function 'do_set_price()' to set price for the kitty.
			Self::do_set_price(who, kitty_id, price)?;
			Ok(())
		}

		/// Extrinsic to buy a kitty.
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: [u8; 32],
			max_price: BalanceOf<T>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Call internal function 'do_buy_kitty' to buy the kitty.
			Self::do_buy_kitty(who, kitty_id, max_price)?;
			Ok(())
		}
	}
}
