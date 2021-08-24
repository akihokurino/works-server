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
        contact_id -> Varchar,
        contact_group_id -> Varchar,
        name -> Varchar,
        billing_amount -> Integer,
        billing_type -> Integer,
        payment_due_on_ym -> Varchar,
        subject -> Varchar,
        subject_template -> Varchar,
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
        issue_at -> Nullable<Datetime>,
        payment_due_on_ymd -> Varchar,
        payment_due_on_at -> Nullable<Datetime>,
        invoice_number -> Varchar,
        payment_status -> Integer,
        invoice_status -> Integer,
        recipient_name -> Varchar,
        subject -> Varchar,
        total_amount -> Integer,
        tax -> Integer,
        pdf_path -> Nullable<Varchar>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}
joinable!(invoices -> suppliers (supplier_id));

table! {
    banks (id) {
        id -> Varchar,
        user_id -> Varchar,
        name -> Varchar,
        code -> Varchar,
        account_type -> Integer,
        account_number -> Varchar,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}
joinable!(banks -> users (user_id));

table! {
    senders (id) {
        id -> Varchar,
        user_id -> Varchar,
        name -> Varchar,
        email -> Varchar,
        tel -> Varchar,
        postal_code -> Varchar,
        address -> Varchar,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}
joinable!(senders -> users (user_id));

allow_tables_to_appear_in_same_query!(users, suppliers, invoices, banks, senders);
