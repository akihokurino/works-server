use crate::ddb;
use crate::domain;
use crate::errors;
use crate::misoca;

pub async fn exec(
    supplier_dao: ddb::Dao<domain::supplier::Supplier>,
    invoice_dao: ddb::Dao<domain::invoice::Invoice>,
    misoca_cli: misoca::Client,
    user_id: String,
    access_token: String,
) -> errors::CoreResult<()> {
    let suppliers = supplier_dao
        .get_all_by_user(user_id)
        .map_err(errors::CoreError::from)?;

    for supplier in suppliers {
        let invoices = misoca_cli
            .get_invoices(
                misoca::get_invoices::Input {
                    access_token: access_token.clone(),
                    page: 1,
                    per_page: 100,
                },
                &supplier,
            )
            .await
            .map_err(errors::CoreError::from)?;

        invoice_dao
            .tx(|| {
                for invoice in invoices {
                    match invoice_dao.get(invoice.id.clone()) {
                        Ok(current) => {
                            if current.should_update(&invoice) {
                                invoice_dao.update(&invoice)?;
                            }
                        }
                        Err(errors::CoreError::NotFound) => {
                            invoice_dao.insert(&invoice)?;
                        }
                        Err(_) => {}
                    }
                }
                Ok(())
            })
            .map_err(errors::CoreError::from)?;
    }

    Ok(())
}
