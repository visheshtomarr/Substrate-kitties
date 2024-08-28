#![cfg_attr(not(feature = "std"), no_std)]

mod impls;

use frame::prelude::*;
pub use pallet::*;

#[frame::pallet(dev_mode)]
pub mod pallet {
	use core::default;

use frame_system::Config;
	use frame_system::Key;

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(core::marker::PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	/// Storage to store count of kitties.
	#[pallet::storage]
	pub(super) type CountForKitties<T: Config> = StorageValue<Value = u32, QueryKind = ValueQuery>;

	/// Storage to store different kitties.
	#[pallet::storage]
	pub(super) type Kitties<T: Config> = StorageMap<Key = [u8; 32], Value = ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created { owner: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Overflow occurs while storing too many kitties.
		TooManyKitties,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// "origin" is the first parameter to every callable function.
		// It describes where the call is calling from, and allows us to perform simple access
		// control logic based on that information.
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Create a default id to insert kitty into the map when we call mint().
			let default_id = [0u8; 32] ;
			Self::mint(who, default_id)?;
			Ok(())
		}
	}
}
