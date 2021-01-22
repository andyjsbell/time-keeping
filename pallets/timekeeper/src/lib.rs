#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, 
	traits::{Get, Currency}
};
use frame_support::weights::{DispatchClass, Pays};
use frame_system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type AccountIdOf<T> = <T as frame_system::Trait>::AccountId;
type BalanceOf<T> = <<T as Trait>::Currency as Currency<AccountIdOf<T>>>::Balance;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TimeKeeperModule {
		/// Store the rate for an account
		pub Rates get(fn rates): map hasher(blake2_128_concat) T::AccountId => Option<BalanceOf<T>>;
		/// Store a whitelist of administrators
		pub Administrators get(fn adminstrators): map hasher(blake2_128_concat) T::AccountId => Option<bool>;
		/// Store a list of creditors for work done
		pub Creditors get(fn creditors): map hasher(blake2_128_concat) T::AccountId => Option<BalanceOf<T>>;
		/// Map whether account is in or out
		pub Entered get(fn entered): map hasher(blake2_128_concat) T::AccountId => Option<bool>;
	}
}

decl_event!(
	pub enum Event<T> 
	where AccountId = <T as frame_system::Trait>::AccountId,
	Balance = BalanceOf<T> {
		/// An account has been registered with an hourly rate
		/// [account, value]
		AccountRegistered(AccountId, Option<Balance>),
		AccountUpdated(AccountId, Option<Balance>),
		AccountEntered(AccountId),
		AccountExited(AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		FailedToEnter,
		FailedToExit,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn register_account(origin, account: T::AccountId, value: Option<BalanceOf<T>>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// TODO check if the origin is in the whitelist
			Rates::<T>::mutate_exists(&account, |v| *v = value);
			// Emit an event.
			Self::deposit_event(RawEvent::AccountRegistered(account, value));
			// Return a successful DispatchResult
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn update_rate_for_account(origin, account: T::AccountId, value: Option<BalanceOf<T>>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			// TODO who here has to be a transaction signed by an administrator and the account holder
			Rates::<T>::mutate_exists(&account, |v| *v = value);
			// Emit an event.
			Self::deposit_event(RawEvent::AccountUpdated(account, value));
			// Return a successful DispatchResult
			Ok(())
		}

		#[weight = (10_000, DispatchClass::Normal, Pays::No)]
		pub fn enter_account(origin) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			match Self::entered(&who) {
				Some(true) => {
					Err(Error::<T>::FailedToEnter)?
				},
				_ => {
					Entered::<T>::mutate_exists(&who, |v| *v = Some(true));
					// Emit an event.
					Self::deposit_event(RawEvent::AccountEntered(who));
					// Return a successful DispatchResult
					Ok(())
				}
			}
		}

		#[weight = (10_000, DispatchClass::Normal, Pays::No)]
		pub fn exit_account(origin) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			match Self::entered(&who) {
				Some(true) => {
					Entered::<T>::mutate_exists(&who, |v| *v = Some(false));
					let rate = Self::rates(&who);
					match rate {
						Some(r) => {
							Creditors::<T>::mutate_exists(&who, |credit| {
								*credit = Some(credit.unwrap_or(0.into()) + r)
							});
						},
						_ => ()
					}
					
					// Emit an event.
					Self::deposit_event(RawEvent::AccountExited(who));
					// Return a successful DispatchResult
					Ok(())
				},
				_ => {
					Err(Error::<T>::FailedToExit)?
				}
			}
		}
	}
}
