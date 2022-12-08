#![allow(panics)]

pub struct Pallet<T>{
    a: T,
}

pub trait Config {
    type SomeType;
}

pub struct DispatchError;
type DispatchResult = Result<(), DispatchError>;

impl<T: Config> Pallet<T> {
    pub fn external_propose_majority(
        a: u32,
        b: u32,
    ) -> DispatchResult {
        Ok(())
    }

    pub fn external_propose_default(
        a: u32,
        b: u32,
    ) -> DispatchResult {
        Ok(())
    }
}

fn main() {}