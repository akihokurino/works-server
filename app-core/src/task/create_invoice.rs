use crate::ddb;
use crate::domain;
use crate::misoca;
use crate::task::get_misoca_token;
use crate::{CoreError, CoreResult};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use std::sync::Mutex;

pub async fn exec(misoca_cli: misoca::Client, now: DateTime<Utc>) -> CoreResult<()> {
    let conn = ddb::establish_connection();
    let user_dao: ddb::Dao<domain::user::User> = ddb::Dao::new();
    let invoice_dao: ddb::Dao<domain::invoice::Invoice> = ddb::Dao::new();
    let bank_dao: ddb::Dao<domain::bank::Bank> = ddb::Dao::new();
    let sender_dao: ddb::Dao<domain::sender::Sender> = ddb::Dao::new();

    let users = user_dao
        .get_all_with_suppliers(&conn)
        .map_err(CoreError::from)?;

    for user in users {
        let only_user = user.0;
        let suppliers = user.1;

        let access_token = get_misoca_token::exec(
            Mutex::new(&conn),
            user_dao.clone(),
            misoca_cli.clone(),
            only_user.id.clone(),
            now,
        )
        .await?;

        let banks = bank_dao.get_all_by_user(&conn, only_user.id.clone())?;
        let senders = sender_dao.get_all_by_user(&conn, only_user.id.clone())?;

        for supplier in suppliers
            .clone()
            .into_iter()
            .filter(|v| v.billing_type == domain::supplier::BillingType::Monthly)
            .collect::<Vec<_>>()
        {
            let subject = supplier.subject_in_this_month(now).clone();
            let (issue_date, payment_due_on) = supplier.payment_date_in_this_month(now);

            println!("月々の請求");
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
                    bank: banks.first().cloned(),
                    sender: senders.first().cloned(),
                    now,
                })
                .await?;

            invoice_dao.insert(&conn, &invoice)?;
        }

        let first_day_in_last_month = NaiveDate::from_ymd(now.year(), now.month() - 1, 1);
        let last_month = domain::YM {
            year: first_day_in_last_month.year() as u32,
            month: first_day_in_last_month.month(),
        };

        for supplier in suppliers
            .clone()
            .into_iter()
            .filter(|v| v.billing_type == domain::supplier::BillingType::OneTime)
            .collect::<Vec<_>>()
        {
            if supplier.end_ym != last_month {
                continue;
            }

            let subject = supplier.subject_in_this_month(now).clone();
            let (issue_date, payment_due_on) = supplier.payment_date_in_this_month(now);

            println!("1回のみの請求");
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
                    bank: banks.first().cloned(),
                    sender: senders.first().cloned(),
                    now,
                })
                .await?;

            invoice_dao.insert(&conn, &invoice)?;
        }
    }

    Ok(())
}
