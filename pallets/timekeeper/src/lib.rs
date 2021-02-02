#![cfg_attr(not(feature = "std"), no_std)]

use dispatch::DispatchResult;
use frame_support::sp_std::convert::TryInto;

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::{Currency, ExistenceRequirement, Get}};
use frame_support::weights::{DispatchClass, Pays};
use frame_system::{ensure_signed, ensure_root};
use sp_runtime::ModuleId;
use sp_runtime::traits::{AccountIdConversion};
use pallet_timestamp as timestamp;
use pallet_access as access;
use orml_utilities::with_transaction_result;
use sp_runtime::traits::Hash;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const PALLET_ID: ModuleId = ModuleId(*b"timekeep");
type AccountIdOf<T> = <T as frame_system::Trait>::AccountId;
type BalanceOf<T> = <<T as Trait>::Currency as Currency<AccountIdOf<T>>>::Balance;

pub trait Trait: timestamp::Trait + access::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Timekeeper {
		pub AdminRole get(fn admin_role): T::Hash;
		pub RegistrarRole get(fn registrar_role): T::Hash;
		/// Store the rate for an account
		pub Rates get(fn rates): map hasher(blake2_128_concat) T::AccountId => Option<BalanceOf<T>>;
		/// Store a list of creditors for work done
		pub Creditors get(fn creditors): map hasher(blake2_128_concat) T::AccountId => Option<BalanceOf<T>>;
		/// Map whether account is in or out
		pub Entered get(fn entered): map hasher(blake2_128_concat) T::AccountId => Option<T::Moment>;
	}
	add_extra_genesis {
		build(|_config| {
			let admin = T::Hashing::hash("timekeeper-administrator".as_bytes());
			let registrar = T::Hashing::hash("timekeeper-registrar".as_bytes());
			// Create the role "administrator" and "registrar", setting the role "admin" as admin of role "registrar"
			// We would need to set an account to the "administrator" role which we would do with a sudo call
			// These roles then can be assigned users with calls on the access pallet
			AdminRole::<T>::put(admin);
			RegistrarRole::<T>::put(registrar);
		})
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
		AdminSetup(AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		FailedToEnter,
		FailedToExit,
		FailedToWithdraw,
		FailedCredit,
		FailedInsufficientCredit,
		ErrorRegistrarRoleRequired,
		ErrorAdminRoleRequired,
		ErrorAlreadyRegistered,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn setup(origin, account: T::AccountId) -> dispatch::DispatchResult {
			let _ = ensure_root(origin)?;
			<access::Module<T>>::set_admin_for_role(RegistrarRole::<T>::get(), AdminRole::<T>::get()).expect("failed creating roles, make sure these roles aren't in use");
			<access::Module<T>>::add_account_to_role(AdminRole::<T>::get(), account.clone())?;
			Self::deposit_event(RawEvent::AdminSetup(account));
			Ok(())
		}
	
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn register_account(origin, account: T::AccountId, rate: Option<BalanceOf<T>>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Rates::<T>::contains_key(&account), Error::<T>::ErrorAlreadyRegistered);
			ensure!(<access::Module<T>>::has_role(RegistrarRole::<T>::get(), who), Error::<T>::ErrorRegistrarRoleRequired);
			Rates::<T>::mutate_exists(&account, |r| *r = rate);
			Self::deposit_event(RawEvent::AccountRegistered(account, rate));
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

