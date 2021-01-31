use crate::{
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
		assert_eq!(AccessModule::roles(WRITER_ROLE).len(), 0);
		assert_eq!(AccessModule::roles(ADMIN_ROLE).len(), 1);
		assert_eq!(AccessModule::roles(ADMIN_ROLE)[0], ALICE);
		// Set ADMIN, or ALICE, as administrator for WRITER ROLE
		AccessModule::set_role_admin(ADMIN_ROLE, WRITER_ROLE);
		// Check we have this set in storage
		assert_eq!(AccessModule::admin_roles(WRITER_ROLE), ADMIN_ROLE);
		// Grant BOB WRITER ROLE by ALICE
		assert_ok!(AccessModule::grant_role(ALICE, WRITER_ROLE, BOB));
		// Grant CHARLIE WRITE ROLE by BOB
		assert_noop!(AccessModule::grant_role(BOB, WRITER_ROLE, CHARLIE), Error::<Test>::AdminRequired);
	});
}