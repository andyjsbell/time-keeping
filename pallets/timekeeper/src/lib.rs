#![cfg_attr(not(feature = "std"), no_std)]

use debug::{debug, info};
use dispatch::DispatchResult;
use frame_support::sp_std::convert::TryInto;

use frame_support::{debug, decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::{Currency, ExistenceRequirement, Get, WithdrawReasons, WithdrawReason}};
use frame_support::weights::{DispatchClass, Pays};
use frame_system::ensure_signed;
use sp_runtime::ModuleId;
use sp_runtime::traits::AccountIdConversion;
use pallet_timestamp as timestamp;
use orml_utilities::with_transaction_result;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const PALLET_ID: ModuleId = ModuleId(*b"timekeep");

type AccountIdOf<T> = <T as frame_system::Trait>::AccountId;
type BalanceOf<T> = <<T as Trait>::Currency as Currency<AccountIdOf<T>>>::Balance;

pub trait Trait: timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TimeKeeper {
		/// Store the rate per hour for an account
		/// 1.0 unit is 1+e12
		pub Rates get(fn rates): map hasher(blake2_128_concat) T::AccountId => Option<BalanceOf<T>>;
		/// Store a whitelist of administrators
		pub Administrators get(fn adminstrators): map hasher(blake2_128_concat) T::AccountId => Option<bool>;
		/// Store a list of creditors for work done
		pub Creditors get(fn creditors): map hasher(blake2_128_concat) T::AccountId => Option<BalanceOf<T>>;
		/// Map whether account is in or out
		pub Entered get(fn entered): map hasher(blake2_128_concat) T::AccountId => Option<T::Moment>;
	}
}

decl_event!(
	pub enum Event<T> 
	where AccountId = <T as frame_system::Trait>::AccountId,
	Balance = BalanceOf<T> {
		/// An account has been registered with an hourly rate
		/// [account, value]
		AccountRegistered(AccountId, Option<Balance>),
		AccountWithdrawl(AccountId, Balance),
		AccountUpdated(AccountId, Option<Balance>),
		AccountEntered(AccountId),
		AccountExited(AccountId),
		Deposit(Balance),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		FailedToEnter,
		FailedToExit,
		FailedToWithdraw,
		FailedCredit,
		FailedInsufficientCredit
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn register_account(origin, account: T::AccountId, rate: Option<BalanceOf<T>>) -> dispatch::DispatchResult {
			// let who = ensure_signed(origin)?;
			ensure!(!Rates::<T>::contains_key(&account), "trying to register an existing account");
			// TODO check if the origin is in the whitelist
			Rates::<T>::mutate_exists(&account, |r| *r = rate);
			// Emit an event.
			Self::deposit_event(RawEvent::AccountRegistered(account, rate));
			// Return a successful DispatchResult
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn update_rate_for_account(origin, account: T::AccountId, rate: Option<BalanceOf<T>>) -> dispatch::DispatchResult {
			// let who = ensure_signed(origin)?;
			// TODO who here has to be a transaction signed by an administrator and the account holder
			Rates::<T>::mutate_exists(&account, |r| *r = rate);
			// Emit an event.
			Self::deposit_event(RawEvent::AccountUpdated(account, rate));
			// Return a successful DispatchResult
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn deposit(origin, value: Option<BalanceOf<T>>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			T::Currency::transfer(
				&who,
				&Self::account_id(),
				value.unwrap(),
				ExistenceRequirement::AllowDeath
			)?;
			// Emit an event.
			Self::deposit_event(RawEvent::Deposit(value.unwrap()));
			// Return a successful DispatchResult
			Ok(())
		}

		#[weight = (10_000, DispatchClass::Normal, Pays::No)]
		pub fn withdraw(origin, amount: BalanceOf<T>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			
			Creditors::<T>::try_mutate_exists(who.clone(), |credit| -> dispatch::DispatchResult {
				let credit = credit.take().ok_or(Error::<T>::FailedCredit)?;
				ensure!(credit >= amount.into(), Error::<T>::FailedInsufficientCredit);
				
				with_transaction_result(|| {
					
					T::Currency::transfer(
						&Self::account_id(),
						&who,
						amount,
						ExistenceRequirement::AllowDeath
					)?;

					Self::deposit_event(RawEvent::AccountWithdrawl(who, amount));
				
					Ok(())
				})
			})?;

			Ok(())
		}

		#[weight = (10_000, DispatchClass::Normal, Pays::No)]
		pub fn enter_account(origin) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			Self::check_if_registered(&who)?;

			match Self::entered(&who) {
				Some(_) => {
					// Already entered
					Err(Error::<T>::FailedToEnter)?
				},
				None => {
					let now = <timestamp::Module<T>>::get();
					Entered::<T>::mutate_exists(&who, |v| *v = Some(now));
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
			Self::check_if_registered(&who)?;
			
			match Self::entered(&who) {
				Some(timestamp) => {
					let now = <timestamp::Module<T>>::get();
					let diff = now - timestamp;
					Entered::<T>::mutate_exists(&who, |v| *v = None);
					let rate = Self::rates(&who);
					match rate {
						Some(r) => {
							Creditors::<T>::mutate_exists(&who, |credit| {
								*credit = Some(credit.unwrap_or(0.into()) + Self::calculate_credit(diff, r));
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

impl<T: Trait> Module<T> {

	pub fn account_id() -> T::AccountId {
		PALLET_ID.into_account()
	}

	pub fn check_if_registered(account: &T::AccountId) -> DispatchResult {
		ensure!(Rates::<T>::contains_key(account), "account not registered");
		Ok(())
	}

	pub fn calculate_credit(time: T::Moment, rate: BalanceOf<T>) -> BalanceOf<T> {
		let t : u64 = TryInto::<u64>::try_into(time).unwrap_or(0);
		let r : u64 = TryInto::<u64>::try_into(rate).unwrap_or(0);
		let credit = (t * r) / (60 * 60 * 1000);
		TryInto::<BalanceOf<T>>::try_into(credit).unwrap_or(0.into())
	}
}

