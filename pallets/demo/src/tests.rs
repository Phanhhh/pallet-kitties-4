use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use super::*;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {

		let origin = Origin::signed(1);
		let name = b"conbo".to_vec();
		let age = 22;

		/// StorageValue StudentId when not created student yet
		assert_eq!(StudentId::<Test>::get(), 0);

		// Dispatch a signed extrinsic.
		assert_ok!(Demo::create_student(Origin::signed(1), name, age));
		
		/// StorageValue StudentId when created student yet
		assert_eq!(StudentId::<Test>::get(), 1);
	});
}

#[test]
fn correct_error_for_to_young() {
	new_test_ext().execute_with(|| {
		let name = b"conbo".to_vec();
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(Demo::create_student(Origin::signed(1), name, 16), Error::<Test>::TooYoung);
	});
}
