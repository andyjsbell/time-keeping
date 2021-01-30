#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{decl_error, decl_event, decl_module, dispatch, decl_storage, ensure, traits::{Get}};
use frame_system::{ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
type HashOf<T> = <T as frame_system::Trait>::Hash;
type AccountIdOf<T> = <T as frame_system::Trait>::AccountId;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Access {
		pub AdminRoles get(fn admin_roles):
			map hasher(blake2_128_concat) HashOf<T> => HashOf<T>;
		pub Roles get(fn roles): 
			map hasher(blake2_128_concat) HashOf<T> => Vec<AccountIdOf<T>>;
	}
}

decl_event!(
	pub enum Event<T>
	where 
	AccountId = <T as frame_system::Trait>::AccountId,
	<T as frame_system::Trait>::Hash {
		/// Account is granted role \[role, previous role, new role\]
		RoleAdminChanged(Hash, Hash, Hash),
		/// Account is granted role \[role, account granted, calling account\]
		RoleGranted(Hash, AccountId, AccountId),
		/// Account is revoked role \[role, account granted, calling account\]
		RoleRevoked(Hash, AccountId, AccountId),	
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		AlreadyMember,
		NotMember,
		AdminRequired,
		RenounceSelf,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn grant_role(origin, role: T::Hash, account: T::AccountId) {
			let who = ensure_signed(origin)?;
			ensure!(Self::has_role(Self::admin_roles(role), who.clone()), Error::<T>::AdminRequired);
			Self::setup_role(role, account.clone())?;
			Self::deposit_event(RawEvent::RoleGranted(role, account, who));
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn revoke_role(origin, role: T::Hash, account: T::AccountId) {
			let who = ensure_signed(origin)?;
			ensure!(Self::has_role(Self::admin_roles(role), who.clone()), Error::<T>::AdminRequired);
			Self::remove_member(role, account.clone())?;
			Self::deposit_event(RawEvent::RoleRevoked(role, account, who));
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn renounce_role(origin, role: T::Hash, account: T::AccountId) {
			let who = ensure_signed(origin)?;
			ensure!(account == who.clone(), Error::<T>::RenounceSelf);
			Self::remove_member(role, account.clone())?;
			Self::deposit_event(RawEvent::RoleRevoked(role, account, who));
		}
	}
}

impl<T: Trait> Module<T> {

	pub fn has_role(role: HashOf<T>, account: AccountIdOf<T>) -> bool {
		Self::roles(role).contains(&account)
	}
	
	pub fn get_role_member_count(role: HashOf<T>) -> usize {
        Self::roles(role).len()
    }

	pub fn get_role_admin(role: HashOf<T>) -> HashOf<T> {
        Self::admin_roles(role)
	}
	
	fn set_role_admin(role: HashOf<T>, admin_role: HashOf<T>) {
		<AdminRoles<T>>::insert(role, admin_role);
	}

	fn setup_role(role: HashOf<T>, account: AccountIdOf<T>) -> dispatch::DispatchResult {
		Self::add_member(role, account.clone())?;
		Ok(())
	}
	
	fn add_member(role: HashOf<T>, account: AccountIdOf<T>) -> dispatch::DispatchResult {
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

	fn remove_member(role: HashOf<T>, account: AccountIdOf<T>) -> dispatch::DispatchResult {
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
