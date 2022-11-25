// Mocking file structure of frame-support project
#![allow(panics)]
#![allow(clippy::result_unit_err)]

pub mod hash {
    pub struct Twox64Concat;
}

pub struct Identity;

pub mod storage {
    pub mod types {
        pub mod map {
            pub struct StorageMap<
                Hasher,
                Key,
                Value
            >(core::marker::PhantomData<(Hasher, Key, Value)>);
        }
    }
}
