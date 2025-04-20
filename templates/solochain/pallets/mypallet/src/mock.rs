use frame_support::{
    parameter_types,
    traits::{Currency, EnsureOrigin, OnInitialize},
    PalletId,
};
use frame_system::{self as system, pallet_prelude::*};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    testing::Header,
    Perbill,
};

pub use crate::Pallet;
use crate::{Event, Error};

#[frame_support::pallet]
pub struct Test;

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Nothing;
    type BlockWeights = frame_support::weights::BlockWeights;
    type BlockLength = frame_support::weights::BlockLength;
    type DbWeight = frame_support::weights::DbWeight;
    type Origin = Origin;
    type Call = Call;
    type Index = u32;
    type BlockNumber = u32;
    type Hash = sp_core::H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = Event;
    type Version = ();
    type PalletInfo = frame_support::PalletInfo;
    type SystemWeightInfo = ();
    type OnInitialize = ();
    type OnFinalization = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
    pub const MaxLocks: u32 = 50;
}

#[frame_support::construct_runtime]
pub struct Runtime;

impl pallet::Config for Test {
    type RuntimeEvent = Event;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
    sp_io::TestExternalities::new(t)
}
