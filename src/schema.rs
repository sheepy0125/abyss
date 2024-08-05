// @generated automatically by Diesel CLI.

diesel::table! {
    cartas (id) {
        id -> Int4,
        #[max_length = 36]
        uuid -> Bpchar,
        parent -> Nullable<Int4>,
        user_id -> Nullable<Int4>,
        #[max_length = 36]
        title -> Nullable<Bpchar>,
        #[max_length = 24]
        sender -> Nullable<Bpchar>,
        #[max_length = 2048]
        content -> Varchar,
        #[max_length = 6]
        modification_code -> Bpchar,
        creation -> Int4,
        modification -> Nullable<Int4>,
        #[max_length = 2]
        lang -> Bpchar,
        random_accessible -> Bool,
        reports -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        certificate_hash -> Bytea,
        #[max_length = 2]
        lang -> Bpchar,
        creation -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cartas,
    users,
);
