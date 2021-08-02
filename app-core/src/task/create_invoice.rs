use crate::ddb;
use crate::ddb::Tx;
use crate::domain;
use crate::misoca;
use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};

pub async fn exec(misoca_cli: misoca::Client, now: DateTime<Utc>) -> CoreResult<()> {
    let conn = ddb::establish_connection();
    let user_dao: ddb::Dao<domain::user::User> = ddb::Dao::new();
    let invoice_dao: ddb::Dao<domain::invoice::Invoice> = ddb::Dao::new();

    let users = user_dao
        .get_all_with_suppliers(&conn)
        .map_err(CoreError::from)?;

    for user in users {
        let mut only_user = user.0;
        let suppliers = user.1;

        if only_user.misoca_refresh_token.is_empty() {
            continue;
        }

        let tokens = misoca_cli
            .refresh_tokens(misoca::tokens::refresh_tokens::Input {
                refresh_token: only_user.misoca_refresh_token.clone(),
            })
            .await?;

        let access_token = tokens.access_token;
        let refresh_token = tokens.refresh_token;

        only_user.update_misoca_refresh_token(refresh_token, now);
        Tx::run(&conn, || {
            user_dao.update(&conn, &only_user)?;
            Ok(())
        })?;

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
