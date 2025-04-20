use crate::{mock::*, Error, Event, Vaults, Strategies};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use frame_system::{self as system, pallet_prelude::*};

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        system::set_block_number(1);
        
        assert_ok!(Pallet::<Test>::deposit_funds(RuntimeOrigin::signed(1), 100));
        assert_eq!(Vaults::<Test>::get(1), 100);
        
        System::assert_last_event(Event::Deposited { who: 1, amount: 100 }.into());
        
        assert_ok!(Pallet::<Test>::withdraw_funds(RuntimeOrigin::signed(1), 50));
        assert_eq!(Vaults::<Test>::get(1), 50);
        
        System::assert_last_event(Event::Withdrawn { who: 1, amount: 50 }.into());
    });
}

#[test]
fn it_subscribes_to_a_strategy() {
    new_test_ext().execute_with(|| {
        system::set_block_number(1);
        
        // Insert an empty strategy using BoundedVec
        Strategies::<Test>::insert(1, BoundedVec::default());
        
        assert_ok!(Pallet::<Test>::subscribe_to_strategy(RuntimeOrigin::signed(1), 1));
        
        let subscribers = Strategies::<Test>::get(1);
        assert!(subscribers.contains(&1));
        
        System::assert_last_event(Event::SubscribedToStrategy { who: 1, strategy_id: 1 }.into());
    });
}

#[test]
fn it_fails_if_strategy_does_not_exist() {
    new_test_ext().execute_with(|| {
        system::set_block_number(1);
        
        // No strategy inserted at id 99
        assert_noop!(
            Pallet::<Test>::subscribe_to_strategy(RuntimeOrigin::signed(1), 99),
            Error::<Test>::InvalidStrategy
        );
    });
}

#[test]
fn it_prevents_duplicate_subscription() {
    new_test_ext().execute_with(|| {
        system::set_block_number(1);
        
        Strategies::<Test>::insert(1, BoundedVec::default());
        
        assert_ok!(Pallet::<Test>::subscribe_to_strategy(RuntimeOrigin::signed(1), 1));
        
        assert_noop!(
            Pallet::<Test>::subscribe_to_strategy(RuntimeOrigin::signed(1), 1),
            Error::<Test>::AlreadySubscribed
        );
    });
}

#[test]
fn it_executes_strategy() {
    new_test_ext().execute_with(|| {
        system::set_block_number(1);
        
        Strategies::<Test>::insert(1, BoundedVec::default());
        
        assert_ok!(Pallet::<Test>::execute_strategy(RuntimeOrigin::signed(1), 1, 100));
        
        System::assert_last_event(Event::StrategyExecuted { strategy_id: 1, amount: 100 }.into());
    });
}

#[test]
fn it_fails_if_insufficient_funds_on_withdrawal() {
    new_test_ext().execute_with(|| {
        system::set_block_number(1);
        
        assert_noop!(
            Pallet::<Test>::withdraw_funds(RuntimeOrigin::signed(1), 100),
            Error::<Test>::InsufficientFunds
        );
    });
}

#[test]
fn it_fails_if_storage_overflow_occur() {
    new_test_ext().execute_with(|| {
        system::set_block_number(1);

        // Pre-fund account 1 to max - 1, then try to deposit 2 to force overflow
        Vaults::<Test>::insert(1, u32::MAX - 1);
        
        assert_noop!(
            Pallet::<Test>::deposit_funds(RuntimeOrigin::signed(1), 2),
            Error::<Test>::StorageOverflow
        );
    });
}
