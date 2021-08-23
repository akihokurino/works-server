use crate::ddb;
use crate::ddb::Tx;
use crate::domain;
use crate::misoca;
use crate::task::get_misoca_token;
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

        let access_token = get_misoca_token::exec(
            Mutex::new(&conn),
            user_dao.clone(),
            misoca_cli.clone(),
            only_user.id.clone(),
            now,
        )
        .await?;

        for supplier in suppliers {
            let invoices = misoca_cli
                .get_invoices(misoca::invoice::get_invoices::Input {
                    access_token: access_token.clone(),
                    page: 1,
                    per_page: 100,
                    supplier_id: supplier.id.clone(),
                    contact_group_id: supplier.contact_group_id.clone(),
                })
                .await?;

            Tx::run(&conn, || {
                for invoice in invoices {
                    match invoice_dao.get(&conn, invoice.id.clone()) {
                        Ok(current) => {
                            if current.should_update(&invoice) {
                                invoice_dao.update(&conn, &invoice)?;
                            }
                        }
                        Err(CoreError::NotFound) => {
                            invoice_dao.insert(&conn, &invoice)?;
                        }
                        Err(_) => {}
                    }
                }
                Ok(())
            })?;
        }
    }

    Ok(())
}
