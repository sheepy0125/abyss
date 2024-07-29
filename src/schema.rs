// @generated automatically by Diesel CLI.

diesel::table! {
    cartas (id) {
        id -> Int4,
        parent -> Nullable<Int4>,
        user_id -> Nullable<Int4>,
        #[max_length = 2048]
        content -> Varchar,
        #[max_length = 6]
        modification_code -> Bpchar,
        creation -> Int4,
        modification -> Nullable<Int4>,
    }
}
