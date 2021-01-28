#![cfg_attr(not(feature = "std"), no_std)]

use debug::{debug, info};
use dispatch::DispatchResult;
use frame_support::sp_std::convert::TryInto;

use frame_support::{debug, decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::{Currency, ExistenceRequirement, Get, WithdrawReasons, WithdrawReason}};
use frame_support::weights::{DispatchClass, Pays};
use frame_system::{ensure_signed, ensure_root};
use sp_runtime::ModuleId;
use sp_runtime::traits::AccountIdConversion;
use pallet_timestamp as timestamp;
use orml_utilities::with_transaction_result;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type AccountIdOf<T> = <T as frame_system::Trait>::AccountId;
pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Access {
		pub Roles get(fn roles): map hasher(blake2_128_concat) T::AccountId => Option<u32>;
	}
}

decl_event!(
	pub enum Event<T>
	where AccountId = <T as frame_system::Trait>::AccountId {
		RoleGranted(AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;
	}
}

impl<T: Trait> Module<T> {
}

