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
		/// Store a list of creditors for work done
		pub Creditors get(fn creditors): map hasher(blake2_128_concat) T::AccountId => Option<Value>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// An account has been registered with an hourly rate
		/// [account, value]
		AccountRegistered(AccountId, Option<Value>),
		AccountUpdated(AccountId, Option<Value>),
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
			Rates::<T>::mutate_exists(&account, |v| *v = value);
			// Emit an event.
			Self::deposit_event(RawEvent::AccountRegistered(account, value));
			// Return a successful DispatchResult
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn update_rate_for_account(origin, account: T::AccountId, value: Option<Value>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// TODO who here has to be a transaction signed by an administrator and the account holder
			Rates::<T>::mutate_exists(&account, |v| *v = value);
			// Emit an event.
			Self::deposit_event(RawEvent::AccountUpdated(account, value));
			// Return a successful DispatchResult
			Ok(())
		}
	}
}
