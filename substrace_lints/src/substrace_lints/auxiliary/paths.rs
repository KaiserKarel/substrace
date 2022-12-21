use substrace_utils::match_def_path;
use rustc_hir as hir;
use rustc_hir::def_id::DefId;
use rustc_lint::LateContext;

pub fn is_insecure_hash_function<'hir>(cx: &LateContext<'hir>, generics: &hir::GenericArgs<'_>) -> bool {
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

pub const STORAGE_MAP: [&str; 5] = ["frame_support", "storage", "types", "map", "StorageMap"];
pub const STORAGE_DOUBLE_MAP: [&str; 5] = ["frame_support", "storage", "types", "double_map", "StorageDoubleMap"];
pub const STORAGE_N_MAP: [&str; 5] = ["frame_support", "storage", "types", "nmap", "StorageNMap"];

pub const TWOX64CONCTAT: [&str; 3] = ["frame_support", "hash", "Twox64Concat"];
pub const TWOX128: [&str; 3] = ["frame_support", "hash", "Twox128"];
pub const TWOX256: [&str; 3] = ["frame_support", "hash", "Twox256"];

pub const IDENTITY: [&str; 3] = ["frame_support", "hash", "Identity"];

pub const WITH_TRANSACTION: [&str; 4] = ["frame_support", "storage", "transactional", "with_transaction"];