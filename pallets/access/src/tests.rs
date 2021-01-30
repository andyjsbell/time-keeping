use crate::{
	RawEvent, 
	Error,
	mock::*
};
use frame_support::{assert_ok, assert_noop};
use sp_core::H256;

const ADMIN_ROLE : H256 = H256::repeat_byte(0);
const WRITER_ROLE : H256 = H256::repeat_byte(1);
const BOB : u64 = 101;
const ALICE : u64 = 100;
const CHARLIE : u64 = 102;

#[test]
fn it_sets_roles() {
	new_test_ext().execute_with(|| {
		// Set ALICE as ADMIN_ROLE
		assert_ok!(AccessModule::setup_role(ADMIN_ROLE, ALICE));
		// Set ADMIN, or ALICE, as administrator for WRITER ROLE
		AccessModule::set_role_admin(ADMIN_ROLE, WRITER_ROLE);
		// Check we have this set in storage
		assert_eq!(AccessModule::admin_roles(WRITER_ROLE), ADMIN_ROLE);
		// Grant BOB WRITER ROLE by ALICE
		assert_ok!(AccessModule::grant_role(Origin::signed(ALICE), WRITER_ROLE, BOB));
		// Event emitted
		assert_eq!(last_event(), Event::access(RawEvent::RoleGranted(WRITER_ROLE, BOB, ALICE)));
		// Grant CHARLIE WRITE ROLE by BOB
		assert_noop!(AccessModule::grant_role(Origin::signed(BOB), WRITER_ROLE, CHARLIE), Error::<Test>::AdminRequired);
	});
}