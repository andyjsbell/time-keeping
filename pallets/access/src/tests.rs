use crate::{
	Error,
	mock::*
};
use frame_support::{assert_ok, assert_noop};
use sp_core::H256;

const NONE_ROLE : H256 = H256::repeat_byte(0);
const ADMIN_ROLE : H256 = H256::repeat_byte(1);
const WRITER_ROLE : H256 = H256::repeat_byte(2);
const BOB : u64 = 101;
const ALICE : u64 = 100;
const CHARLIE : u64 = 102;

#[test]
fn it_creates_and_grants_roles() {
	new_test_ext().execute_with(|| {
		// Make sure we don't have these roles set
		assert_eq!(AccessModule::get_admin_of_role(ADMIN_ROLE), NONE_ROLE);		
		assert_eq!(AccessModule::get_admin_of_role(WRITER_ROLE), NONE_ROLE);
		// Add Alice to Admin group
		assert_ok!(AccessModule::add_account_to_role(ADMIN_ROLE, ALICE));
		// Check storage
		assert_eq!(AccessModule::roles(WRITER_ROLE).len(), 0);
		assert_eq!(AccessModule::roles(ADMIN_ROLE).len(), 1);
		assert_eq!(AccessModule::roles(ADMIN_ROLE)[0], ALICE);
		// Set ADMIN, or ALICE, as administrator for WRITER ROLE
		assert_ok!(AccessModule::set_admin_for_role(WRITER_ROLE, ADMIN_ROLE));
		// Try to set it again will fail
		assert_noop!(AccessModule::set_admin_for_role(WRITER_ROLE, ADMIN_ROLE), Error::<Test>::AdminRoleExists);
		// Check we have this set in storage, the admin role for writer role should be 
		assert_eq!(AccessModule::get_admin_of_role(WRITER_ROLE), ADMIN_ROLE);
		// Grant BOB WRITER role by ALICE
		assert_ok!(AccessModule::grant_role(Origin::signed(ALICE), WRITER_ROLE, BOB));
		// BOB should have WRITER role
		assert!(AccessModule::has_role(WRITER_ROLE, BOB));
		// ALICE should not have WRITER role
		assert!(!AccessModule::has_role(WRITER_ROLE, ALICE));
		// Grant CHARLIE WRITE ROLE by BOB, BOB should not be able to
		assert_noop!(AccessModule::grant_role(Origin::signed(BOB), WRITER_ROLE, CHARLIE), Error::<Test>::AdminRequired);
	});
}

#[test]
fn it_revokes_roles() {
	new_test_ext().execute_with(|| {
		// Add Alice to Admin group
		assert_ok!(AccessModule::add_account_to_role(ADMIN_ROLE, ALICE));
		assert_ok!(AccessModule::set_admin_for_role(WRITER_ROLE, ADMIN_ROLE));
		// Grant BOB WRITER role by ALICE
		assert_ok!(AccessModule::grant_role(Origin::signed(ALICE), WRITER_ROLE, BOB));
		// Revoke BOB WRITER role
		assert_ok!(AccessModule::revoke_role(Origin::signed(ALICE), WRITER_ROLE, BOB));
		// Check storage
		assert!(!AccessModule::has_role(WRITER_ROLE, BOB));
		// Grant BOB WRITER role by ALICE
		assert_ok!(AccessModule::grant_role(Origin::signed(ALICE), WRITER_ROLE, BOB));
	});
}

#[test]
fn it_renounces_roles() {
	new_test_ext().execute_with(|| {
		// Add Alice to Admin group
		assert_ok!(AccessModule::add_account_to_role(ADMIN_ROLE, ALICE));
		assert_ok!(AccessModule::set_admin_for_role(WRITER_ROLE, ADMIN_ROLE));
		// Grant BOB WRITER role by ALICE
		assert_ok!(AccessModule::grant_role(Origin::signed(ALICE), WRITER_ROLE, BOB));
		// Renounce BOB WRITER role
		// Should fail if the administrator role tries
		assert_noop!(AccessModule::renounce_role(Origin::signed(ALICE), WRITER_ROLE, BOB), Error::<Test>::RenounceSelf);
		// The account should be able to
		assert_ok!(AccessModule::renounce_role(Origin::signed(BOB), WRITER_ROLE, BOB));
		// Check storage
		assert!(!AccessModule::has_role(WRITER_ROLE, BOB));
	});
}