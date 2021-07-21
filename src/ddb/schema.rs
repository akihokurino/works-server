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

table! {
    invoices (id) {
        id -> Varchar,
        supplier_id -> Varchar,
        issue_ymd -> Varchar,
        issue_at -> Date,
        payment_due_on_ymd -> Varchar,
        payment_due_on_at -> Date,
        invoice_number -> Varchar,
        payment_status -> Integer,
        invoice_status -> Integer,
        recipient_name -> Varchar,
        subject -> Varchar,
        total_amount -> Integer,
        tax -> Integer,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}
joinable!(invoices -> suppliers (supplier_id));

allow_tables_to_appear_in_same_query!(users, suppliers, invoices,);
