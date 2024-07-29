// @generated automatically by Diesel CLI.

diesel::table! {
    cartas (id) {
        id -> Int4,
        parent -> Nullable<Int4>,
        user_id -> Nullable<Int4>,
    }
}
