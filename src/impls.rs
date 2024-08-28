use super::*;
use frame::prelude::*;

impl<T: Config> Pallet<T> {
	pub fn mint(owner: T::AccountId) -> DispatchResult {
		// Get the current count of kitties.
		let current_count = CountForKitties::<T>::get().unwrap_or(0) ;

		// Create new count by adding one to the current count while using safe math.
		let new_count = current_count.checked_add(1).ok_or(Error::<T>::TooManyKitties) ?;

		// Set new count of kitties.
		CountForKitties::<T>::set(Some(new_count)) ;

		// Emit event.
		Self::deposit_event(Event::<T>::Created { owner });
		Ok(())
	}
}
