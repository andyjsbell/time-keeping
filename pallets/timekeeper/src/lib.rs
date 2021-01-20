#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get};
use frame_system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type Value = u32;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TimeKeeperModule {
		/// Store the rate for an account
		pub Rates get(fn rates): map hasher(blake2_128_concat) T::AccountId => Option<Value>;
		/// Store a whitelist of administrators
		pub Administrators get(fn adminstrators): map hasher(blake2_128_concat) T::AccountId => Option<bool>;		
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// An account has been registered with an hourly rate
		/// [account, value]
		AccountRegistered(AccountId, Option<Value>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn register_account(origin, account: T::AccountId, value: Option<Value>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// TODO check if the origin is in the whitelist
			Self::set_rate(account, value)
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn update_rate_for_account(origin, account: T::AccountId, value: Option<Value>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// TODO who here has to be a transaction signed by an administrator and the account holder
			Self::set_rate(account, value)
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			// match Something::get() {
			// 	// Return an error if the value has not been set.
			// 	None => Err(Error::<T>::NoneValue)?,
			// 	Some(old) => {
			// 		// Increment the value read from storage; will error in the event of overflow.
			// 		let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
			// 		// Update the value in storage with the incremented result.
			// 		Something::put(new);
			// 		Ok(())
			// 	},
			// }

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	fn set_rate(account: T::AccountId, value:Option<Value>) -> dispatch::DispatchResult {
		Rates::<T>::mutate_exists(&account, |v| *v = value);
		// Emit an event.
		Self::deposit_event(RawEvent::AccountRegistered(account, value));
		// Return a successful DispatchResult
		Ok(())
	}
}
