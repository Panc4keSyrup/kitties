#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::pallet_prelude::DispatchResult;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;
use scale_info::TypeInfo;
pub type Id = u32;
use sp_runtime::ArithmeticError;
use frame_support::dispatch::fmt;
use frame_support::traits::UnixTime;

#[frame_support::pallet]
pub mod pallet {

	pub use super::*;
	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: Vec<u8>,
		pub price: u64,
		pub gender: Gender,
		pub owner: T::AccountId,
		pub created_date: u64,
	}
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Gender {
		Male,
		Female,
	}

	impl<T:Config> fmt::Debug for Kitty<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Kitty")
			 .field("dna", &self.dna)
			 .field("price", &self.price)
			 .field("gender", &self.gender)
			 .field("owner", &self.owner)
			 .field("created_date", &self.created_date)
			 .finish()
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TimeProvider: UnixTime;
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;
	}



	#[pallet::storage]
	#[pallet::getter(fn kitty_id)]
	pub type KittyId<T> = StorageValue<_, Id, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_kitty)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<Vec<u8>, T::MaxKittyOwned>, ValueQuery>;







	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new kitty was successfully created.
		Created { kitty: Vec<u8>, owner: T::AccountId, timestamp: u64 },
		Transferred { from: T::AccountId, to: T::AccountId, kitty:Vec<u8> },

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		DuplicateKitty,
		TooManyOwned,
		NoKitty,
		NotOwner,
		TransferToSelf,
		ExceedMaxKittyOwned,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8>) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let owner = ensure_signed(origin)?;

			let created_date = T::TimeProvider::now().as_secs();
			
			let gender = Self::gen_gender(&dna)?;
			let kitty = Kitty::<T> { dna: dna.clone(), price: 0, gender, owner: owner.clone(), created_date, };
			
			ensure!(!Kitties::<T>::contains_key(&kitty.dna), Error::<T>::DuplicateKitty);

			let current_id = KittyId::<T>::get();
			let next_id = current_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			<KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| {
				kitty_vec.try_push(dna.clone())
			  }).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			Kitties::<T>::insert(dna.clone(), kitty.clone());
			KittyId::<T>::put(next_id);
			log::info!("New kitty:{:?}", kitty);
			Self::deposit_event(Event::Created { kitty: dna, owner: owner.clone(), timestamp: created_date.clone()});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			dna: Vec<u8>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let from = ensure_signed(origin)?;
			let kitty = Kitties::<T>::get(&dna).ok_or(Error::<T>::NoKitty)?;
			
			ensure!(kitty.owner == from, Error::<T>::NotOwner);
			ensure!(from != to, Error::<T>::TransferToSelf);

			let prev_owner = kitty.owner.clone();

			<KittiesOwned<T>>::try_mutate(&prev_owner, |owned| {
			if let Some(ind) = owned.iter().position(|ids| *ids == dna) {
				owned.swap_remove(ind);
				return Ok(());
			}Err(())
			}).map_err(|_| Error::<T>::NoKitty);

			<Kitties::<T>>::insert(dna.clone(), kitty);

			<KittiesOwned<T>>::try_mutate(&to, |kitty_vec| 
			{kitty_vec.try_push(dna.clone())}).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;
			log::info!("Transfered kitty to: {:?}", to.clone());
			Self::deposit_event(Event::Transferred { from, to, kitty: dna });

			Ok(())
		}

	}
}

impl<T> Pallet<T> {
	fn gen_gender(dna: &Vec<u8>) -> Result<Gender,Error<T>>{
		let mut res = Gender::Female;
		if dna.len() % 2 ==0 {	
			res = Gender::Male;
		}
		Ok(res)
	}

}