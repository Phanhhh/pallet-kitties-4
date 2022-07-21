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

use frame_support::dispatch::fmt;
use frame_support::inherent::Vec;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::EncodeLike;
	use frame_support::traits::DenyAll;

	pub use super::*;

	/// Struct for Kitty
	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: Vec<u8>,
		owner: T::AccountId,
		price: u32,
		gender: Gender,
	}


	/// enum Gender of Kitty
	#[derive(TypeInfo, Encode, Decode, Debug)]
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
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
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
		StorageMap<_, Blake2_128Concat, Vec<u8>, Kitty<T>, OptionQuery>;

	// StorageMap for owner has how many kitty
	//key : T:: AccountId
	//value : vec<dna>
	#[pallet::storage]
	#[pallet::getter(fn KittyOwner)]
	pub(super) type KittyOwner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Vec<u8>>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		/// Event for kitty stored
		KittyStored(Vec<u8>, T::AccountId),

		/// Event for kitty transferred
		KittyTransferred(T::AccountId, T::AccountId, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Errors when transfer to yourself
		TransferToSelf,
		/// Errors when Kitty not exist
		KittyNotExist,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Function create kitty by user.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8>, price: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			let temp_dna = dna.clone();

			// Generate Gender for kitty
			let gender = Self::gen_gender(temp_dna)?;

			// Create kitty
			let kitty = Kitty { 
				dna: dna.clone(), 
				price: price, 
				gender: gender, 
				owner: who.clone() 
			};

			// Insert kitty into Kitties (StorageMap)
			<Kitties<T>>::insert(&dna, kitty);

			// Insert kitty into KittyOwner (StorageMap)

			let kitty_vec = <KittyOwner<T>>::get(&who);

			let mut kitty_vec_check = match kitty_vec {
				None => Vec::new(),
				_ => <KittyOwner<T>>::get(&who).unwrap(),
			};

			kitty_vec_check.push(dna.clone());
			<KittyOwner<T>>::insert(who.clone(), kitty_vec_check);

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
			dna: Vec<u8>,
			newOwner: T::AccountId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Verify the kitty is not transferring back to its owner.
			ensure!(who != newOwner, <Error<T>>::TransferToSelf);

			let prev_owner = who.clone();

			// Check and Update new owner of kitty into Kitties
			let kitty_by_dna = <Kitties<T>>::get(&dna);

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
					dna_list.push(dna.clone());
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

impl<T> Pallet<T> {
	fn gen_gender(dna: Vec<u8>) -> Result<Gender, Error<T>> {
		let mut res = Gender::Male;
		if dna.len() % 2 == 1 {
			res = Gender::Female;
		}
		Ok(res)
	}
}
