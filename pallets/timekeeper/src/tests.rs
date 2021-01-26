use crate::{
	RawEvent, 
	mock::*
};
use frame_support::{assert_ok, assert_noop};
use frame_support::{debug};
const CHECKER : u64 = 101;
const ADMIN : u64 = 100;
const AMOUNT : u64 = 10;

#[test]
fn it_works_registering_a_user() {
	new_test_ext().execute_with(|| {
		assert_ok!(TimeKeeperModule::register_account(Origin::signed(ADMIN), CHECKER, Some(AMOUNT)));
		assert_eq!(TimeKeeperModule::rates(&CHECKER), Some(AMOUNT));
		assert_eq!(last_event(), Event::timekeeper(RawEvent::AccountRegistered(CHECKER, Some(AMOUNT))));
	});
}

#[test]
fn it_checks_in_and_out() {
	new_test_ext().execute_with(|| {
		assert_ok!(TimeKeeperModule::register_account(Origin::signed(ADMIN), CHECKER, Some(AMOUNT)));
		assert_eq!(last_event(), Event::timekeeper(RawEvent::AccountRegistered(CHECKER, Some(AMOUNT))));
		assert_ok!(TimeKeeperModule::enter_account(Origin::signed(CHECKER)));
		assert_eq!(last_event(), Event::timekeeper(RawEvent::AccountEntered(CHECKER)));
		assert_ok!(TimeKeeperModule::exit_account(Origin::signed(CHECKER)));
		assert_eq!(last_event(), Event::timekeeper(RawEvent::AccountExited(CHECKER)));
		assert_eq!(TimeKeeperModule::creditors(&CHECKER), Some(AMOUNT));
	});
}
