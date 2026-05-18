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

    notification_inbox (id) {
        id -> Uuid,
        #[max_length = 150]
        app_package -> Varchar,
        raw_title -> Nullable<Text>,
        raw_body -> Text,
        received_at -> Timestamptz,
        #[max_length = 20]
        status -> Varchar,
        transaction_id -> Nullable<Uuid>,
        amount -> Nullable<Numeric>,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
        pocket_id -> Nullable<Uuid>,
        category_id -> Nullable<Uuid>,
        destination_pocket_id -> Nullable<Uuid>,
        #[max_length = 255]
        title -> Nullable<Varchar>,
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

diesel::joinable!(notification_inbox -> transactions (transaction_id));
diesel::joinable!(transactions -> categories (category_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    notification_inbox,
    pockets,
    transactions,
);
