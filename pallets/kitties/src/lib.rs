#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use frame_support::dispatch::fmt;
use frame_support::inherent::Vec;
use scale_info::TypeInfo;
use frame_support::sp_runtime::ArithmeticError;
use frame_support::traits::Currency;
use frame_support::traits::UnixTime;
use frame_support::traits::Randomness;
use frame_support::sp_runtime::traits::Hash;
use frame_support::dispatch::fmt::Debug;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
#[frame_support::pallet]
pub mod pallet {
	use codec::EncodeLike;
	use frame_support::traits::{DenyAll, UnixTime};

	pub use super::*;

	/// Struct for Kitty
	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: T::Hash,
		owner: T::AccountId,
		price: BalanceOf<T>,
		gender: Gender,
		created_date: <T as pallet_timestamp::Config>::Moment,
	}


	/// enum Gender of Kitty
	#[derive(TypeInfo, Encode, Decode, Debug, Clone, Copy, PartialEq)]
	pub enum Gender {
		Male,
		Female,
	}

	/// Implement function generate Gender of Kitty
	impl Default for Gender {
		fn default() -> Self {
			Gender::Male
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		// create associate type "MaxKitty" for pallet-kitties. (left: Name --- Right: Trait name)
		type MaxKitty: Get<u32>;

		// create KittyRandomness
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn kitty_number)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type KittyNumber<T> = StorageValue<_, u32, ValueQuery>;

	// StorageMap for Kitty
	//key : dna
	//value : struct Kitty
	#[pallet::storage]
	#[pallet::getter(fn Kitties)]
	pub(super) type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T>, OptionQuery>;

	// StorageMap for owner has how many kitty
	//key : T:: AccountId
	//value : vec<dna>
	#[pallet::storage]
	#[pallet::getter(fn KittyOwner)]
	pub(super) type KittyOwner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::Hash, T::MaxKitty>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		/// Event for kitty stored
		KittyCreated(T::Hash, T::AccountId),

		/// Event for kitty transferred
		KittyTransferred(T::AccountId, T::AccountId, T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Errors duplicate
		DuplicateKitty,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Errors when transfer to yourself
		TransferToSelf,
		/// Errors when Kitty not exist
		KittyNotExist,
		/// Errors when Kitty not exceed number
		ExceedNumberKitty,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Function create kitty by user.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			
			let dna = Self::gen_dna();

			// let dna_for_gender = dna.clone().to_vec();
			// let temp_dna = dna.clone();

			// Generate Gender for kitty
			let gender = Self::gen_gender(dna.clone());

			// Create date use tightly coupling
			let now = pallet_timestamp::Pallet::<T>::now();

			// Create kitty
			let kitty = Kitty { 
				dna: dna.clone(), 
				price: 0u32.into(), 
				gender: gender, 
				owner: who.clone() ,
				created_date: now,
			};

			// Get max kitty
			let max = T::MaxKitty::get();

			// Check duplicate kitty
			ensure!(!Kitties::<T>::contains_key(&kitty.dna), Error::<T>::DuplicateKitty);

			// Insert kitty into Kitties (StorageMap)
			<Kitties<T>>::insert(&dna, kitty);

			// Insert kitty into KittyOwner (StorageMap)
			let get_kitty = <KittyOwner<T>>::get(&who).unwrap_or_default();

			ensure!((get_kitty.len() as u32) < max, Error::<T>::ExceedNumberKitty);

			KittyOwner::<T>::try_append(&who, dna.clone()).map_err(|_| <Error<T>>::KittyNotExist)?;
			// let mut kitty_vec_check = match kitty_vec {
			// 	None => Vec::new(),
			// 	_ => <KittyOwner<T>>::get(&who).unwrap(),
			// };

			// get_kitty.push(dna.clone());
			// <KittyOwner<T>>::insert(who.clone(), get_kitty);

			// Update number of kitty into KittyNumber (StorageValue)
			let mut number = <KittyNumber<T>>::get();
			number += 1;
			<KittyNumber<T>>::put(number);

			// Emit an event.
			Self::deposit_event(Event::KittyStored(dna, who));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// Function transfer owner of kitty
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_owner_kitty(
			origin: OriginFor<T>,
			dna: T::Hash,
			newOwner: T::AccountId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Verify the kitty is not transferring back to its owner.
			ensure!(who != newOwner, <Error<T>>::TransferToSelf);

			let prev_owner = who.clone();

			// Get and check kitty is exist
			let kitty_by_dna = <Kitties<T>>::get(&dna);
			ensure!(kitty_by_dna.is_none(), Error::<T>::KittyNotExist);
			
			// Get and check max kitty in newOwner
			let max = T::MaxKitty::get();
			let get_kitty = <KittyOwner<T>>::get(&newOwner).unwrap_or_default();
			ensure!((get_kitty.len() as u32) < max, Error::<T>::ExceedNumberKitty);

			// Check and Update new owner of kitty into Kitties
			if let Some(mut kitty) = kitty_by_dna {
				kitty.owner = newOwner.clone();
				<Kitties<T>>::insert(&dna, kitty);
			}

			// Remove `kitty_dna` from the KittyOwner vector of `prev_owner'
			<KittyOwner<T>>::mutate(&prev_owner, |dna_list| {
				if let Some(dna_list) = dna_list {
					dna_list.retain(|dna_list| dna_list != &dna);
				}
			});

			// Update new owner of kitty into KittyOwner
			<KittyOwner<T>>::mutate(&newOwner, |dna_list| {
				if let Some(dna_list) = dna_list {
					dna_list.try_push(dna.clone());
				}
			});

			// Emit an event.
			Self::deposit_event(Event::KittyTransferred(prev_owner, newOwner.clone(), dna.clone()));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

// helper function: generate gender according to dna

impl<T: Config> Pallet<T> {
	// fn gen_gender(dna: Vec<u8>) -> Result<Gender, Error<T>> {
	// 	let mut res = Gender::Male;
	// 	if dna.len() % 2 == 1 {
	// 		res = Gender::Female;
	// 	}
	// 	Ok(res)
	// }

	fn gen_gender(dna: T::Hash) -> Gender {
		match dna.as_ref()[0] % 2 {
			0 => Gender::Male,
			_ => Gender::Female,
		}
	}

	fn gen_dna() -> T::Hash {
		let random_gen_dna = (
			T::KittyRandomness::random(&b"dna"[..]).0,
			<frame_system::Pallet<T>>::block_number(),
		);
		let kitty_dna_random = T::Hashing::hash_of(&random_gen_dna);
		kitty_dna_random
	}
}
