use substrace_utils::match_def_path;
use rustc_hir as hir;
use rustc_hir::def_id::DefId;
use rustc_lint::LateContext;

pub fn is_insecure_hash_function<'hir>(cx: &LateContext<'hir>, generics: &hir::GenericArgs) -> bool {
    for typ in generics.args {
        if let hir::GenericArg::Type(ty) = typ {
            if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = &ty.kind {
                if let hir::def::Res::Def(hir::def::DefKind::Struct, id) = path.res {
                    if match_def_path(cx, id, &TWOX64CONCTAT)
                        || match_def_path(cx, id, &IDENTITY)
                        || match_def_path(cx, id, &TWOX128)
                        || match_def_path(cx, id, &TWOX256) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn is_like_storage_map(cx: &LateContext<'_>, fn_def_id: DefId) -> bool {
    match_def_path(cx, fn_def_id, &STORAGE_MAP)
        || match_def_path(cx, fn_def_id, &STORAGE_DOUBLE_MAP)
        || match_def_path(cx, fn_def_id, &STORAGE_N_MAP)
}

pub fn is_storage_map(cx: &LateContext<'_>, fn_def_id: DefId) -> bool {
    match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_MAP_ITER)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_MAP_DRAIN)
}

pub fn is_storage_double_map(cx: &LateContext<'_>, fn_def_id: DefId) -> bool {
    match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_ITER_PREFIX)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_DRAIN_PREFIX)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_ITER)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_DRAIN)
}

pub fn modifies_storage_double_map(cx: &LateContext<'_>, fn_def_id: DefId) -> bool {
    match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_SWAP)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_TAKE)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_INSERT)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_REMOVE)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_REMOVE_PREFIX)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_TRY_MUTATE)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_MUTATE)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_TRY_MUTATE_EXISTS)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_DOUBLE_MAP_APPEND)
}

pub fn modifies_storage_map(cx: &LateContext<'_>, fn_def_id: DefId) -> bool {
    match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_MAP_SWAP)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_MAP_REMOVE)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_MAP_TAKE)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_MAP_APPEND)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_MAP_INSERT)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_MAP_MIGRATE_KEY)
        || match_def_path(cx, fn_def_id, &ITERABLE_STORAGE_MAP_MIGRATE_KEY_FROM_BLAKE)
}

pub const STORAGE_MAP: [&str; 5] = ["frame_support", "storage", "types", "map", "StorageMap"];
pub const STORAGE_DOUBLE_MAP: [&str; 5] = ["frame_support", "storage", "types", "double_map", "StorageDoubleMap"];
pub const STORAGE_N_MAP: [&str; 5] = ["frame_support", "storage", "types", "nmap", "StorageNMap"];

pub const TWOX64CONCTAT: [&str; 3] = ["frame_support", "hash", "Twox64Concat"];
pub const TWOX128: [&str; 3] = ["frame_support", "hash", "Twox128"];
pub const TWOX256: [&str; 3] = ["frame_support", "hash", "Twox256"];

pub const IDENTITY: [&str; 3] = ["frame_support", "hash", "Identity"];

pub const ITERABLE_STORAGE_MAP_ITER: [&str; 4] = ["frame_support", "storage", "IterableStorageMap", "iter"];
pub const ITERABLE_STORAGE_MAP_DRAIN: [&str; 4] = ["frame_support", "storage", "IterableStorageMap", "drain"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_ITER_PREFIX: [&str; 4] =
    ["frame_support", "storage", "IterableStorageDoubleMap", "iter_prefix"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_DRAIN_PREFIX: [&str; 4] =
    ["frame_support", "storage", "IterableStorageDoubleMap", "drain_prefix"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_ITER: [&str; 4] =
    ["frame_support", "storage", "IterableStorageDoubleMap", "iter"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_DRAIN: [&str; 4] =
    ["frame_support", "storage", "IterableStorageDoubleMap", "drain"];
pub const ITERABLE_STORAGE_MAP_INSERT: [&str; 4] = ["frame_support", "storage", "StorageMap", "insert"];
pub const ITERABLE_STORAGE_MAP_SWAP: [&str; 4] = ["frame_support", "storage", "StorageMap", "swap"];
pub const ITERABLE_STORAGE_MAP_REMOVE: [&str; 4] = ["frame_support", "storage", "StorageMap", "remove"];
pub const ITERABLE_STORAGE_MAP_TAKE: [&str; 4] = ["frame_support", "storage", "StorageMap", "take"];
pub const ITERABLE_STORAGE_MAP_APPEND: [&str; 4] = ["frame_support", "storage", "StorageMap", "append"];
pub const ITERABLE_STORAGE_MAP_MIGRATE_KEY: [&str; 4] = ["frame_support", "storage", "StorageMap", "migrate_key"];
pub const ITERABLE_STORAGE_MAP_MIGRATE_KEY_FROM_BLAKE: [&str; 4] =
    ["frame_support", "storage", "StorageMap", "migrate_key_from_blake"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_SWAP: [&str; 4] = ["frame_support", "storage", "StorageDoubleMap", "swap"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_TAKE: [&str; 4] = ["frame_support", "storage", "StorageDoubleMap", "take"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_INSERT: [&str; 4] = ["frame_support", "storage", "StorageDoubleMap", "insert"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_REMOVE: [&str; 4] = ["frame_support", "storage", "StorageDoubleMap", "remove"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_REMOVE_PREFIX: [&str; 4] =
    ["frame_support", "storage", "StorageDoubleMap", "remove_prefix"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_MUTATE: [&str; 4] = ["frame_support", "storage", "StorageDoubleMap", "mutate"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_APPEND: [&str; 4] = ["frame_support", "storage", "StorageDoubleMap", "append"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_TRY_MUTATE: [&str; 4] =
    ["frame_support", "storage", "StorageDoubleMap", "try_mutate"];
pub const ITERABLE_STORAGE_DOUBLE_MAP_TRY_MUTATE_EXISTS: [&str; 4] =
    ["frame_support", "storage", "StorageDoubleMap", "try_mutate_exists"];
