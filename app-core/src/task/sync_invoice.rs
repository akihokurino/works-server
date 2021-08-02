use crate::ddb;
use crate::domain;
use crate::misoca;
use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};

pub async fn exec(misoca_cli: misoca::Client, now: DateTime<Utc>) -> CoreResult<()> {
    let conn = &ddb::establish_connection();
    let user_dao: ddb::Dao<domain::user::User> = ddb::Dao::new();
    let invoice_dao: ddb::Dao<domain::invoice::Invoice> = ddb::Dao::new();

    let users = user_dao
        .get_all_with_suppliers(conn)
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
        user_dao.tx(|| {
            user_dao.update(conn, &only_user)?;
            Ok(())
        })?;

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

            invoice_dao.tx(|| {
                for invoice in invoices {
                    match invoice_dao.get(conn, invoice.id.clone()) {
                        Ok(current) => {
                            if current.should_update(&invoice) {
                                invoice_dao.update(conn, &invoice)?;
                            }
                        }
                        Err(CoreError::NotFound) => {
                            invoice_dao.insert(conn, &invoice)?;
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
