use crate::{
	RawEvent, 
	mock::*
};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_registering_a_user() {
	new_test_ext().execute_with(|| {
		assert_ok!(TimeKeeperModule::register_account(Origin::signed(100), 101, Some(10)));
		assert_eq!(TimeKeeperModule::rates(&101), Some(10));
		assert_eq!(last_event(), Event::timekeeper(RawEvent::AccountRegistered(101, Some(10))));
	});
}
