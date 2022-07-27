//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Kitties;
use frame_benchmarking::vec;
use frame_benchmarking::{benchmarks, whitelisted_caller, account};
use frame_system::RawOrigin;
use frame_support::StorageMap;
use frame_support::dispatch::result::Result;

benchmarks! {
	// tên của benchmark
	create_kitty {
	// khởi tạo các tham số cho extrinsic benchmark
		// let dnas : Vec<u8> = b"uocgimotngaymaitoicoduoccongviecveblockchainvakiemduocnhieutienvasaudotoisegiupdogiadinhtoiroifsaudotoisegiupdonhieunguoikhacoquecuatoivanhieufnguoikhactrendatnuocvietnamnaykecatrenholannguoilontuoidoladieuuocmocuartoiniemaouoclonnhatvaduynhatcuatoivatoitintoiselamduocCamonmoinguoicamontatcanhungnguoidadenvagiupdotoicamonratnhieu".to_vec();

		let caller: T::AccountId = whitelisted_caller();
	}: create_kitty (RawOrigin::Signed(caller))

	// kiểm tra lại trạng thái storage khi thực hiện extrinsic xem đúng chưa
	verify {
		assert_eq!(KittyNumber::<T>::get(), 1);
	}

	transfer_owner_kitty {
	// khởi tạo các tham số cho extrinsic benchmark
		// let dnas : Vec<u8> = b"uocgimotngaymaitoicoduoccongviecveblockchainvakiemduocnhieutienvasaudotoisegiupdogiadinhtoiroifsaudotoisegiupdonhieunguoikhacoquecuatoivanhieufnguoikhactrendatnuocvietnamnaykecatrenholannguoilontuoidoladieuuocmocuartoiniemaouoclonnhatvaduynhatcuatoivatoitintoiselamduocCamonmoinguoicamontatcanhungnguoidadenvagiupdotoicamonratnhieu".to_vec();

		let caller: T::AccountId = whitelisted_caller();

		let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));

		Kitties::<T>::create_kitty(caller_origin);
		
		let receiver: T::AccountId = account("receiver", 0, 0);

		let kitty_dna = Kitties::<T>::kitties_owned(caller.clone()).unwrap()[0];

	}: transfer_owner_kitty (RawOrigin::Signed(caller.clone()), kitty_dna.clone(), receiver.clone())

	// kiểm tra lại trạng thái storage khi thực hiện extrinsic xem đúng chưa
	verify {
		assert_eq!(KittyNumber::<T>::get(), 1);
		assert_eq!(Kitties::<T>::kitties_owned(caller).unwrap().len(), 0);
		// assert_eq!(Kitties::<T>::kitties_owned(receiver).len(), 1);
	}

	// thực hiện benchmark với mock runtime, storage ban đầu.
	impl_benchmark_test_suite!(Kitty, crate::mock::new_test_ext(), crate::mock::Test);
}
