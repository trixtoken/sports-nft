#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch,
    traits::{Currency, Get, LockIdentifier, LockableCurrency, Randomness, Time, WithdrawReason},
};
use frame_system::ensure_signed;
use sp_core::RuntimeDebug;
use sp_std::vec::Vec;

use pallet_commodities::nft::UniqueAssets;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const MODULE_ID: LockIdentifier = *b"sportnft";

/// Attributes that uniquely identify a kitty
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Default, RuntimeDebug)]
pub struct SportsInfo<Hash, Moment> {
    dob: Moment,
    dna: Hash,
}

/// Attributes that do not uniquely identify a kitty
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Default, RuntimeDebug)]
pub struct SportsMetadata {
    name: Vec<u8>,
}

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type SportsInfoOf<T> =
    SportsInfo<<T as frame_system::Trait>::Hash, <<T as Trait>::Time as Time>::Moment>;

pub trait Trait: frame_system::Trait {
    type Sports: pallet_commodities::nft::UniqueAssets<
        Self::AccountId,
        AssetId = Self::Hash,
        AssetInfo = SportsInfoOf<Self>,
    >;
    type Time: frame_support::traits::Time;
    type Randomness: frame_support::traits::Randomness<Self::Hash>;
    type Currency: frame_support::traits::LockableCurrency<Self::AccountId>;
    type BasePrice: Get<BalanceOf<Self>>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as SportsNft {
        MetadataForSports get(fn metadata_for_sports_nft): map hasher(identity) T::Hash => SportsMetadata;
    }
}

decl_event!(
    pub enum Event<T>
    where
        SportsId = <T as frame_system::Trait>::Hash,
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        Conjured(SportsId, AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        SportsConjureFailure,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        /// The dispatch origin for this call must be Signed.
        #[weight = 10_000]
        pub fn conjure(origin, name: Vec<u8>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            T::Currency::set_lock(MODULE_ID, &who, T::BasePrice::get(), WithdrawReason::Fee | WithdrawReason::Reserve);
            match T::Sports::mint(&who, SportsInfo{dob: T::Time::now(), dna: T::Randomness::random(&MODULE_ID)}) {
                Ok(id) => {
                    MetadataForSports::<T>::insert(id, SportsMetadata{name: name});
                    Self::deposit_event(RawEvent::Conjured(id, who));
                },
                Err(err) => Err(err)?
            }

            Ok(())
        }
    }
}
