use super::*;
use frame::prelude::*;
use frame::primitives::BlakeTwo256;
use frame::traits::tokens::Preservation;
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
		let kitty = Kitty { dna, owner: owner.clone(), price: None };

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
		// Transfer should not be happening to the same owner.
		ensure!(from != to, Error::<T>::TransferToSelf);

		// Get the kitty from the storage (if it exists).
		let mut kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::KittyNotFound)?;

		// Kitty should be owned by 'from' to initiate a transfer.
		ensure!(kitty.owner == from, Error::<T>::NotOwner);

		// Update the owner of kitty to AccountId 'to'.
		kitty.owner = to.clone();

		// Update KittiesOwned for 'from' and 'to'.
		let mut to_owned = KittiesOwned::<T>::get(&to);

		// Try push the kitty id for the account 'to'. If we are not able to push the kitty, i.e.,
		// if the boundedvec has reached its limit, we return an error.
		to_owned.try_push(kitty_id).map_err(|_| Error::<T>::TooManyOwnedKitties)?;

		let mut from_owned = KittiesOwned::<T>::get(&from);

		// Try to remove the kitty id for the account 'from'. If it is not present, we return an
		// error.
		if let Some(index) = from_owned.iter().position(|&id| id == kitty_id) {
			// 'swap_remove' will remove that given index from the vector, panics if 'index' is out
			// of bounds.
			from_owned.swap_remove(index);
		} else {
			return Err(Error::<T>::KittyNotFound.into());
		}

		// Update the 'Kitties' storage.
		Kitties::<T>::insert(kitty_id, kitty);

		// Update the 'KittiesOwned' storage for 'to'.
		KittiesOwned::<T>::insert(&to, to_owned);

		// Update the 'KittiesOwned' storage for 'from'.
		KittiesOwned::<T>::insert(&from, from_owned);

		// Emit successful transfer event.
		Self::deposit_event(Event::<T>::Transferred { from, to, kitty_id });
		Ok(())
	}

	// Set price for a kitty.
	pub fn do_set_price(
		caller: T::AccountId,
		kitty_id: [u8; 32],
		new_price: Option<BalanceOf<T>>,
	) -> DispatchResult {
		// Get the kitty from the storage (if it exists).
		let mut kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::KittyNotFound)?;

		// Ensure whether the caller is the owner of that kitty or not.
		ensure!(caller == kitty.owner, Error::<T>::NotOwner);

		// Set new price of kitty.
		kitty.price = new_price;

		// Update kitties storage.
		Kitties::<T>::insert(kitty_id, kitty);

		// Emit successful price set event.
		Self::deposit_event(Event::<T>::PriceSet { owner: caller, kitty_id, new_price });
		Ok(())
	}

	// Buy a kitty.
	pub fn do_buy_kitty(
		buyer: T::AccountId,
		kitty_id: [u8; 32],
		price: BalanceOf<T> 
	) -> DispatchResult {
		// Get the kitty from the storage (if it exists).
		let kitty = Kitties::<T>::get(kitty_id).ok_or(Error::<T>::KittyNotFound)?;

		// Get the real price of kitty (if it is for sale).
		let real_price = kitty.price.ok_or(Error::<T>::NotForSale)?;

		// Ensure whether the buying price is greater than the real price of kitty.
		ensure!(price >= real_price, Error::<T>::MaxPriceTooLow);

		// Transfer amount from buyer to seller.
		T::NativeBalance::transfer(&buyer, &kitty.owner, real_price, Preservation::Preserve)?;

		// Transfer kitty from seller to buyer.
		Self::do_transfer(kitty.owner, buyer.clone(), kitty_id)?;

		// Emit successful kitty sold event.
		Self::deposit_event(Event::<T>::Sold { buyer, kitty_id, price: real_price });
		Ok(())
	}
}
