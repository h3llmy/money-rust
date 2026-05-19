// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;

    categories (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        #[max_length = 100]
        name -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        #[max_length = 100]
        username -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    pockets (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 50]
        pocket_type -> Varchar,
        #[max_length = 3]
        currency -> Varchar,
        balance -> Numeric,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    transactions (id) {
        id -> Uuid,
        user_id -> Uuid,
        pocket_id -> Uuid,
        category_id -> Nullable<Uuid>,
        amount -> Numeric,
        #[sql_name = "type"]
        type_ -> Varchar,
        #[max_length = 255]
        title -> Varchar,
        transaction_time -> Timestamptz,
        destination_pocket_id -> Nullable<Uuid>,
        description -> Nullable<Text>,
        status -> Varchar,
    }
}


diesel::joinable!(transactions -> categories (category_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    pockets,
    transactions,
    users,
);
