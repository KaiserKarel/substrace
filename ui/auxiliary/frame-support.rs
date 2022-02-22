#![allow(panics)]

pub mod storage {
    pub trait IterableStorageMap {
        type Iter;
        
        fn iter() -> Self::Iter;
    }


    pub trait StorageMap<K, V> {
        /// The type that get/take return.
        type Query;

        /// Get the storage key used to fetch a value corresponding to a specific key.
        fn hashed_key_for<KeyArg>(key: KeyArg) -> Vec<u8>;

        /// Does the value (explicitly) exist in storage?
        fn contains_key<KeyArg>(key: KeyArg) -> bool;

        /// Load the value associated with the given key from the map.
        fn get<KeyArg>(key: KeyArg) -> Self::Query;

        /// Try to get the value for the given key from the map.
        ///
        /// Returns `Ok` if it exists, `Err` if not.
        fn try_get<KeyArg>(key: KeyArg) -> Result<V, ()>;

        /// Swap the values of two keys.
        fn swap<KeyArg1, KeyArg2>(key1: KeyArg1, key2: KeyArg2);

        /// Store a value to be associated with the given key from the map.
        fn insert<KeyArg, ValArg>(key: KeyArg, val: ValArg);

        /// Remove the value under a key.
        fn remove<KeyArg>(key: KeyArg);

        /// Mutate the value under a key.
        fn mutate<KeyArg, R, F: FnOnce(&mut Self::Query) -> R>(key: KeyArg, f: F) -> R;

        /// Mutate the item, only if an `Ok` value is returned.
        fn try_mutate<KeyArg, R, E, F: FnOnce(&mut Self::Query) -> Result<R, E>>(
            key: KeyArg,
            f: F,
        ) -> Result<R, E>;

        /// Mutate the value under a key.
        ///
        /// Deletes the item if mutated to a `None`.
        fn mutate_exists<KeyArg, R, F: FnOnce(&mut Option<V>) -> R>(key: KeyArg, f: F) -> R;

        /// Mutate the item, only if an `Ok` value is returned. Deletes the item if mutated to a `None`.
        fn try_mutate_exists<KeyArg, R, E, F: FnOnce(&mut Option<V>) -> Result<R, E>>(
            key: KeyArg,
            f: F,
        ) -> Result<R, E>;

        /// Take the value under a key.
        fn take<KeyArg>(key: KeyArg) -> Self::Query;

        /// Append the given items to the value in the storage.
        ///
        /// `V` is required to implement `codec::EncodeAppend`.
        ///
        /// # Warning
        ///
        /// If the storage item is not encoded properly, the storage will be overwritten
        /// and set to `[item]`. Any default value set for the storage item will be ignored
        /// on overwrite.
        fn append<Item, EncodeLikeItem, EncodeLikeKey>(key: EncodeLikeKey, item: EncodeLikeItem);

        /// Migrate an item with the given `key` from a defunct `OldHasher` to the current hasher.
        ///
        /// If the key doesn't exist, then it's a no-op. If it does, then it returns its value.
        fn migrate_key<OldHasher, KeyArg>(key: KeyArg) -> Option<V>;

        /// Migrate an item with the given `key` from a `blake2_256` hasher to the current hasher.
        ///
        /// If the key doesn't exist, then it's a no-op. If it does, then it returns its value.
        fn migrate_key_from_blake<KeyArg>(key: KeyArg) -> Option<V>;
    }

    pub trait StorageDoubleMap<K1, K2, V> {
        /// The type that get/take returns.
        type Query;

        /// Get the storage key used to fetch a value corresponding to a specific key.
        fn hashed_key_for<KArg1, KArg2>(k1: KArg1, k2: KArg2) -> Vec<u8>;
    
        /// Does the value (explicitly) exist in storage?
        fn contains_key<KArg1, KArg2>(k1: KArg1, k2: KArg2) -> bool;

        /// Load the value associated with the given key from the double map.
        fn get<KArg1, KArg2>(k1: KArg1, k2: KArg2) -> Self::Query;

        /// Try to get the value for the given key from the double map.
        ///
        /// Returns `Ok` if it exists, `Err` if not.
        fn try_get<KArg1, KArg2>(k1: KArg1, k2: KArg2) -> Result<V, ()>;

        /// Take a value from storage, removing it afterwards.
        fn take<KArg1, KArg2>(k1: KArg1, k2: KArg2) -> Self::Query;

        /// Swap the values of two key-pairs.
        fn swap<XKArg1, XKArg2, YKArg1, YKArg2>(x_k1: XKArg1, x_k2: XKArg2, y_k1: YKArg1, y_k2: YKArg2);

        /// Store a value to be associated with the given keys from the double map.
        fn insert<KArg1, KArg2, VArg>(k1: KArg1, k2: KArg2, val: VArg);

        /// Remove the value under the given keys.
        fn remove<KArg1, KArg2>(k1: KArg1, k2: KArg2);

        /// Remove all values under the first key.
        fn remove_prefix<KArg1>(k1: KArg1);
        
        /// Mutate the value under the given keys.
        fn mutate<KArg1, KArg2, R, F>(k1: KArg1, k2: KArg2, f: F) -> R
        where
            F: FnOnce(&mut Self::Query) -> R;

        /// Mutate the value under the given keys when the closure returns `Ok`.
        fn try_mutate<KArg1, KArg2, R, E, F>(k1: KArg1, k2: KArg2, f: F) -> Result<R, E>
        where
            F: FnOnce(&mut Self::Query) -> Result<R, E>;

        /// Mutate the value under the given keys. Deletes the item if mutated to a `None`.
        fn mutate_exists<KArg1, KArg2, R, F>(k1: KArg1, k2: KArg2, f: F) -> R
        where
            F: FnOnce(&mut Option<V>) -> R;

        /// Mutate the item, only if an `Ok` value is returned. Deletes the item if mutated to a `None`.
        fn try_mutate_exists<KArg1, KArg2, R, E, F>(k1: KArg1, k2: KArg2, f: F) -> Result<R, E>
        where
            F: FnOnce(&mut Option<V>) -> Result<R, E>;

        /// Append the given item to the value in the storage.
        ///
        /// `V` is required to implement [`StorageAppend`].
        ///
        /// # Warning
        ///
        /// If the storage item is not encoded properly, the storage will be overwritten
        /// and set to `[item]`. Any default value set for the storage item will be ignored
        /// on overwrite.
        fn append<Item, EncodeLikeItem, KArg1, KArg2>(
            k1: KArg1,
            k2: KArg2,
            item: EncodeLikeItem,
        );


        /// Read the length of the storage value without decoding the entire value under the
        /// given `key1` and `key2`.
        ///
        /// `V` is required to implement [`StorageDecodeLength`].
        ///
        /// If the value does not exists or it fails to decode the length, `None` is returned.
        /// Otherwise `Some(len)` is returned.
        ///
        /// # Warning
        ///
        /// `None` does not mean that `get()` does not return a value. The default value is completly
        /// ignored by this function.
        fn decode_len<KArg1, KArg2>(key1: KArg1, key2: KArg2) -> Option<usize>;

        /// Migrate an item with the given `key1` and `key2` from defunct `OldHasher1` and
        /// `OldHasher2` to the current hashers.
        ///
        /// If the key doesn't exist, then it's a no-op. If it does, then it returns its value.
        fn migrate_keys<
            OldHasher1,
            OldHasher2,
            KeyArg1,
            KeyArg2,
        >(key1: KeyArg1, key2: KeyArg2) -> Option<V>;
    }

    pub trait IterableStorageDoubleMap<
        K1,
        K2,
        V
    >: StorageDoubleMap<K1, K2, V> {
        /// The type that iterates over all `(key2, value)`.
        type PrefixIterator: Iterator<Item = (K2, V)>;

        /// The type that iterates over all `(key1, key2, value)`.
        type Iterator: Iterator<Item = (K1, K2, V)>;

        /// Enumerate all elements in the map with first key `k1` in no particular order. If you add or
        /// remove values whose first key is `k1` to the map while doing this, you'll get undefined
        /// results.
        fn iter_prefix(k1: K1) -> Self::PrefixIterator;

        /// Remove all elements from the map with first key `k1` and iterate through them in no
        /// particular order. If you add elements with first key `k1` to the map while doing this,
        /// you'll get undefined results.
        fn drain_prefix(k1: K1) -> Self::PrefixIterator;

        /// Enumerate all elements in the map in no particular order. If you add or remove values to
        /// the map while doing this, you'll get undefined results.
        fn iter() -> Self::Iterator;

        /// Remove all elements from the map and iterate through them in no particular order. If you
        /// add elements to the map while doing this, you'll get undefined results.
        fn drain() -> Self::Iterator;

        /// Translate the values of all elements by a function `f`, in the map in no particular order.
        /// By returning `None` from `f` for an element, you'll remove it from the map.
        ///
        /// NOTE: If a value fail to decode because storage is corrupted then it is skipped.
        fn translate<O, F: FnMut(K1, K2, O) -> Option<V>>(f: F);
    }
}
