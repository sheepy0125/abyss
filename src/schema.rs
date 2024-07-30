// @generated automatically by Diesel CLI.

diesel::table! {
    cartas (id) {
        id -> Int4,
        parent -> Nullable<Int4>,
        user_id -> Nullable<Int4>,
        #[max_length = 24]
        title -> Nullable<Bpchar>,
        #[max_length = 2048]
        content -> Varchar,
        #[max_length = 6]
        modification_code -> Bpchar,
        creation -> Int4,
        modification -> Nullable<Int4>,
        random_accessible -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        certificate_hash -> Bytea,
        creation -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cartas,
    users,
);
