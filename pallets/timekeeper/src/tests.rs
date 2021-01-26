use crate::{
	RawEvent, 
	mock::*
};
use frame_support::{assert_ok, assert_noop};

const BOB : u64 = 101;
const ALICE : u64 = 100;
const RATE : u64 = 10;
const BLOCK_JUMP : u64 = 10;

#[test]
fn it_works_registering_a_user() {
	new_test_ext().execute_with(|| {
		// Register user BOB at RATE
		assert_ok!(TimeKeeperModule::register_account(Origin::signed(ALICE), BOB, Some(RATE)));
		// Check we have set the RATE for BOB
		assert_eq!(TimeKeeperModule::rates(&BOB), Some(RATE));
		// Confirm we sent the event out for BOB at RATE set
		assert_eq!(last_event(), Event::timekeeper(RawEvent::AccountRegistered(BOB, Some(RATE))));
	});
}

#[test]
fn it_checks_in_and_out() {
	new_test_ext().execute_with(|| {
		// Register BOB at RATE
		assert_ok!(TimeKeeperModule::register_account(Origin::signed(ALICE), BOB, Some(RATE)));
		// Confirm we sent the event out for BOB at RATE
		assert_eq!(last_event(), Event::timekeeper(RawEvent::AccountRegistered(BOB, Some(RATE))));
		// BOB enters
		assert_ok!(TimeKeeperModule::enter_account(Origin::signed(BOB)));
		// Confirm we sent the event for BOB entered
		assert_eq!(last_event(), Event::timekeeper(RawEvent::AccountEntered(BOB)));
		// Move to block
		run_to_block(BLOCK_JUMP);
		// BOB exits
		assert_ok!(TimeKeeperModule::exit_account(Origin::signed(BOB)));
		// Confirm we sent the event for BOB exited
		assert_eq!(last_event(), Event::timekeeper(RawEvent::AccountExited(BOB)));
		// Check that BOB is now a creditor and should have credit of RATE * BLOCK_JUMP
		assert_eq!(TimeKeeperModule::creditors(&BOB), Some(RATE * BLOCK_JUMP));
	});
}
