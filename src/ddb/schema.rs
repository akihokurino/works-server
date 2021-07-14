table! {
    users (id) {
        id -> Varchar,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

allow_tables_to_appear_in_same_query!(users,);
