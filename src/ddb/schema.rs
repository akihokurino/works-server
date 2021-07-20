table! {
    users (id) {
        id -> Varchar,
        misoca_refresh_token -> Varchar,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
    suppliers (id) {
        id -> Varchar,
        user_id -> Varchar,
        name -> Varchar,
        billing_amount -> Integer,
        billing_type -> Integer,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

joinable!(suppliers -> users (user_id));

allow_tables_to_appear_in_same_query!(users, suppliers,);
