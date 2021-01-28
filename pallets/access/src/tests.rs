use crate::{
	RawEvent, 
	Error,
	mock::*
};
use frame_support::{assert_ok, assert_noop, assert_err};

#[test]
fn it_runs() {
	new_test_ext().execute_with(|| {
		assert_ok!(true)
	});
}