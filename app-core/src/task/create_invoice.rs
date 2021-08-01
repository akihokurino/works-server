use crate::ddb;
use crate::domain;
use crate::misoca;
use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};

pub async fn exec(misoca_cli: misoca::Client, now: DateTime<Utc>) -> CoreResult<()> {
    let user_dao: ddb::Dao<domain::user::User> = ddb::Dao::new(ddb::establish_connection());
    let invoice_dao: ddb::Dao<domain::invoice::Invoice> =
        ddb::Dao::new(ddb::establish_connection());

    let users = user_dao.get_all_with_suppliers().map_err(CoreError::from)?;

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
            user_dao.update(&only_user)?;
            Ok(())
        })?;

        for supplier in suppliers {
            let invoice = misoca_cli
                .create_invoice(misoca::invoice::create_invoice::Input {
                    access_token: access_token.clone(),
                    supplier_id: supplier.id.clone(),
                    contact_id: supplier.contact_id.clone(),
                    subject: supplier.subject.clone(),
                    billing_amount: supplier.billing_amount.clone(),
                    now,
                })
                .await?;

            invoice_dao.insert(&invoice)?;
        }
    }

    Ok(())
}
