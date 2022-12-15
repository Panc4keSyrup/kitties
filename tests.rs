use crate::mock::*;
use frame_support::{assert_ok};

#[test]
fn is_the_right_kitty_created() {
	new_test_ext().execute_with(|| {
		let a =  1u8;
		let b = a.to_vec();
		assert_ok!(Kitties::create_kitty(RuntimeOrigin::signed(1), b));
		assert_eq!(Kitties::kitty_owned(1).to_vec(), b);
	});
}
#[test]
fn is_kitty_transfered() {
	new_test_ext().execute_with(|| {
		let a =  [1u8];
		let b = a.to_vec();
		assert_ok!(Kitties::create_kitty(RuntimeOrigin::signed(1), b));
		assert_ok!(Kitties::transfer(RuntimeOrigin::signed(1), 2, b));
		assert_eq!(Kitties::kitty_owned(2).to_vec(), b);
	});	
}