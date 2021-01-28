use crate::{Module, Trait};
use sp_core::H256;
use frame_support::{
	impl_outer_origin, 
	impl_outer_event, 
	parameter_types, 
	weights::Weight,
	traits::{OnInitialize, OnFinalize},
};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

mod access {
	pub use super::super::*;
}

impl_outer_event! {
	pub enum Event for Test {
		frame_system<T>,
		access<T>,
	}
}
// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl frame_system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl Trait for Test {
	type Event = Event;
}

pub type AccessModule = Module<Test>;
pub type System = frame_system::Module<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

    frame_system::GenesisConfig::default().assimilate_storage::<Test>(&mut t).unwrap();
    
	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
    t.execute_with(|| System::set_block_number(1) );
    t
}

pub fn last_event() -> Event {
    System::events().last().unwrap().event.clone()
}


pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        // AccessModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        // AccessModule::on_initialize(System::block_number());
	}
	
	Timestamp::set_timestamp(n);
}