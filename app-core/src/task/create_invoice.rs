use crate::ddb;
use crate::domain;
use crate::misoca;
use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};
use std::sync::Mutex;

pub async fn exec(misoca_cli: misoca::Client, now: DateTime<Utc>) -> CoreResult<()> {
    let conn = ddb::establish_connection();
    let user_dao: ddb::Dao<domain::user::User> = ddb::Dao::new();
    let invoice_dao: ddb::Dao<domain::invoice::Invoice> = ddb::Dao::new();

    let users = user_dao
        .get_all_with_suppliers(&conn)
        .map_err(CoreError::from)?;

    for user in users {
        let only_user = user.0;
        let suppliers = user.1;

        let access_token = domain::service::get_misoca_token::exec(
            Mutex::new(&conn),
            user_dao.clone(),
            misoca_cli.clone(),
            only_user.id.clone(),
            now,
        )
        .await?;

        for supplier in suppliers {
            let subject = supplier.subject_in_this_month(now).clone();
            let (issue_date, payment_due_on) = supplier.payment_date_in_this_month(now);

            println!("請求先: {}", supplier.name.clone());
            println!("発行日: {}", issue_date);
            println!("支払い期日: {}", payment_due_on);

            let exist =
                invoice_dao.exist_by_subject(&conn, supplier.id.clone(), subject.clone())?;
            if exist {
                println!(
                    "請求先[{}]の請求書はすでに存在します",
                    supplier.name.clone()
                );
                continue;
            }

            let invoice = misoca_cli
                .create_invoice(misoca::invoice::create_invoice::Input {
                    access_token: access_token.clone(),
                    supplier_id: supplier.id.clone(),
                    contact_id: supplier.contact_id.clone(),
                    subject,
                    issue_date,
                    payment_due_on,
                    billing_amount: supplier.billing_amount.clone(),
                    now,
                })
                .await?;

            invoice_dao.insert(&conn, &invoice)?;
        }
    }

    Ok(())
}
