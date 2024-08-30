use super::*;
use frame::prelude::*;
use frame::primitives::BlakeTwo256;
use frame::traits::Hash;

impl<T: Config> Pallet<T> {
	// Generates a unique dna by hashing data from 'frame_system::Pallet::<T>'.
	pub fn gen_dna() -> [u8; 32] {
		// Create a randomness payload.
		// Mulitple kitties can be generated in the same block retaining randomness.
		let unique_payload = (
			frame_system::Pallet::<T>::parent_hash(),
			frame_system::Pallet::<T>::block_number(),
			frame_system::Pallet::<T>::extrinsic_index(),
			CountForKitties::<T>::get(),
		);

		BlakeTwo256::hash_of(&unique_payload).into()
	}

	// Mint a new kitty.
	pub fn mint(owner: T::AccountId, dna: [u8; 32]) -> DispatchResult {
		// Create an instance of Kitty.
		let kitty = Kitty { dna, owner: owner.clone() };

		// Ensure whether a kitty is already present in our storage or not.
		ensure!(!Kitties::<T>::contains_key(dna), Error::<T>::DuplicateKitty);

		// Get the current count of kitties.
		let current_count = CountForKitties::<T>::get();

		// Create new count by adding one to the current count while using safe math.
		let new_count = current_count.checked_add(1).ok_or(Error::<T>::TooManyKitties)?;

		// Append kitty's DNA to the vector for the 'owner'
		KittiesOwned::<T>::try_append(&owner, dna).map_err(|_| Error::<T>::TooManyOwnedKitties)?;

		// Inserts a new kitty in 'Kitties' map whenever mint() is called.
		Kitties::<T>::insert(dna, kitty);

		// Set new count of kitties.
		CountForKitties::<T>::set(new_count);

		// Emit successful creation event.
		Self::deposit_event(Event::<T>::Created { owner });
		Ok(())
	}

	// Transfer a kitty.
	pub fn do_transfer(
		from: T::AccountId, 
		to: T::AccountId, 
		kitty_id: [u8; 32]
	) -> DispatchResult {
		// Emit successful transfer event.
		Self::deposit_event(Event::<T>::Transferred { from, to, kitty_id });
		Ok(())
	}
}
