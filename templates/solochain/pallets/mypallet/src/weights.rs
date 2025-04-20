use frame_support::weights::{Weight};
use sp_runtime::traits::Saturating;

pub trait WeightInfo {
    fn deposit_funds() -> Weight;
    fn withdraw_funds() -> Weight;
    fn subscribe_to_strategy() -> Weight;
    fn execute_strategy() -> Weight;
}

pub struct WeightInfoImpl;

impl WeightInfo for WeightInfoImpl {
    fn deposit_funds() -> Weight {
        10_000.into() // Ensuring conversion to Weight
    }

    fn withdraw_funds() -> Weight {
        10_000.into()
    }

    fn subscribe_to_strategy() -> Weight {
        15_000.into()
    }

    fn execute_strategy() -> Weight {
        20_000.into()
    }
}
