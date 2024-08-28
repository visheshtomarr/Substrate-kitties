use super::*;
use frame::prelude::*;

impl<T: Config> Pallet<T> {
	pub fn mint(owner: T::AccountId, default_kitty: [u8; 32]) -> DispatchResult {
		// Ensure whether a kitty is already present in our storage or not.
		ensure!(!Kitties::<T>::contains_key(default_kitty), Error::<T>::DuplicateKitty);

		// Get the current count of kitties.
		let current_count = CountForKitties::<T>::get();

		// Create new count by adding one to the current count while using safe math.
		let new_count = current_count.checked_add(1).ok_or(Error::<T>::TooManyKitties)?;

		// Inserts a new kitty in 'Kitties' map whenever mint() is called.
		Kitties::<T>::insert(default_kitty, ()) ;

		// Set new count of kitties.
		CountForKitties::<T>::set(new_count);

		// Emit event.
		Self::deposit_event(Event::<T>::Created { owner });
		Ok(())
	}
}
