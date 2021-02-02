#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{decl_error, decl_event, decl_module, dispatch, decl_storage, ensure, traits::{Get}};
use frame_system::ensure_signed;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Access {
		pub AdminRoles get(fn admin_roles):
			map hasher(blake2_128_concat) T::Hash => T::Hash;
		pub Roles get(fn roles): 
			map hasher(blake2_128_concat) T::Hash => Vec<T::AccountId>;
	}
}

decl_event!(
	pub enum Event<T> 
	where 
	AccountId = <T as frame_system::Trait>::AccountId,
	Hash = <T as frame_system::Trait>::Hash,
	{
		/// An role has been granted to an account
		/// [caller, role, account]
		RoleGranted(AccountId, Hash, AccountId),
		/// An role has been revoked for an account
		/// [caller, role, account]
		RoleRevoked(AccountId, Hash, AccountId),
		/// An role has been renounced for an account
		/// [caller, role]
		RoleRenounced(AccountId, Hash),
	}
);


decl_error! {
	pub enum Error for Module<T: Trait> {
		AlreadyMember,
		NotMember,
		AdminRequired,
		RenounceSelf,
		AdminRoleExists,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn grant_role(origin, role: T::Hash, account: T::AccountId) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::has_role(Self::get_admin_of_role(role), who.clone()), Error::<T>::AdminRequired);
			Self::add_account_to_role(role, account.clone())?;
			Self::deposit_event(RawEvent::RoleGranted(who, role, account));
			Ok(())
		}
	
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn revoke_role(origin, role: T::Hash, account: T::AccountId) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::has_role(Self::get_admin_of_role(role), who.clone()), Error::<T>::AdminRequired);
			Self::remove_member(role, account.clone())?;
			Self::deposit_event(RawEvent::RoleRevoked(who, role, account));
			Ok(())
		}
	
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn renounce_role(origin, role: T::Hash, account: T::AccountId) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(account == who, Error::<T>::RenounceSelf);
			Self::remove_member(role, account.clone())?;
			Self::deposit_event(RawEvent::RoleRenounced(account, role));
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {

	pub fn has_role(role: T::Hash, account: T::AccountId) -> bool {
		Self::roles(role).contains(&account)
	}
	
	pub fn get_role_member_count(role: T::Hash) -> usize {
        Self::roles(role).len()
    }

	pub fn get_admin_of_role(role: T::Hash) -> T::Hash {
        Self::admin_roles(role)
	}
	
	pub fn set_admin_for_role(role: T::Hash, admin_role: T::Hash) -> dispatch::DispatchResult {
		ensure!(!<AdminRoles<T>>::contains_key(&role), Error::<T>::AdminRoleExists);
		<AdminRoles<T>>::insert(role, admin_role);
		Ok(())
	}

	pub fn add_account_to_role(role: T::Hash, account: T::AccountId) -> dispatch::DispatchResult {
		Self::add_member(role, account.clone())?;
		Ok(())
	}
	
	fn add_member(role: T::Hash, account: T::AccountId) -> dispatch::DispatchResult {
		let mut roles = Self::roles(role);
		match roles.binary_search(&account) {
			Ok(_) => Err(Error::<T>::AlreadyMember.into()),
			Err(index) => {
				roles.insert(index, account.clone());
				<Roles<T>>::insert(role, roles);
				Ok(())
			}
		}
	}

	fn remove_member(role: T::Hash, account: T::AccountId) -> dispatch::DispatchResult {
		let mut roles = Self::roles(role);
		match roles.binary_search(&account) {
			Ok(index) => {
				roles.remove(index);
				<Roles<T>>::insert(role, roles);
				Ok(())
			},
			Err(_) => Err(Error::<T>::NotMember.into()),
		}
	}
}
