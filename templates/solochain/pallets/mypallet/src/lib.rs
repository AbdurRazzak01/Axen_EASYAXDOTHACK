#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn vaults)]
    pub(super) type Vaults<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        <T as frame_system::Config>::AccountId, 
        u32, 
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn strategies)]
    pub(super) type Strategies<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        u32, 
        BoundedVec<<T as frame_system::Config>::AccountId, ConstU32<100>>, 
        ValueQuery
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Deposited { who: <T as frame_system::Config>::AccountId, amount: u32 },
        SubscribedToStrategy { who: <T as frame_system::Config>::AccountId, strategy_id: u32 },
        Withdrawn { who: <T as frame_system::Config>::AccountId, amount: u32 },
        StrategyExecuted { strategy_id: u32, amount: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        InsufficientFunds,
        AlreadySubscribed,
        InvalidStrategy,
        StorageOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::deposit_funds())]
        pub fn deposit_funds(origin: OriginFor<T>, amount: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let current_balance = Vaults::<T>::get(&who);
            let new_balance = current_balance
                .checked_add(amount)
                .ok_or(Error::<T>::StorageOverflow)?;
            Vaults::<T>::insert(&who, new_balance);
            Self::deposit_event(Event::Deposited { who, amount });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::withdraw_funds())]
        pub fn withdraw_funds(origin: OriginFor<T>, amount: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let current_balance = Vaults::<T>::get(&who);
            if current_balance < amount {
                return Err(Error::<T>::InsufficientFunds.into());
            }
            let new_balance = current_balance
                .checked_sub(amount)
                .ok_or(Error::<T>::StorageOverflow)?;
            Vaults::<T>::insert(&who, new_balance);
            Self::deposit_event(Event::Withdrawn { who, amount });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::subscribe_to_strategy())]
        pub fn subscribe_to_strategy(origin: OriginFor<T>, strategy_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut subscribers = Strategies::<T>::get(strategy_id);

            if subscribers.contains(&who) {
                return Err(Error::<T>::AlreadySubscribed.into());
            }

            subscribers
                .try_push(who.clone())
                .map_err(|_| Error::<T>::StorageOverflow)?;
            Strategies::<T>::insert(strategy_id, subscribers);

            Self::deposit_event(Event::SubscribedToStrategy { who, strategy_id });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::execute_strategy())]
        pub fn execute_strategy(origin: OriginFor<T>, strategy_id: u32, amount: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            Self::deposit_event(Event::StrategyExecuted { strategy_id, amount });
            Ok(())
        }
    }
}
