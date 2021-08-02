use crate::ddb;
use crate::domain;
use crate::misoca;
use crate::{CoreError, CoreResult};
use diesel::MysqlConnection;

pub async fn exec(
    conn: &MysqlConnection,
    supplier_dao: ddb::Dao<domain::supplier::Supplier>,
    invoice_dao: ddb::Dao<domain::invoice::Invoice>,
    misoca_cli: misoca::Client,
    user_id: String,
    access_token: String,
) -> CoreResult<()> {
    let suppliers = supplier_dao
        .get_all_by_user(conn, user_id)
        .map_err(CoreError::from)?;

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

        invoice_dao.tx(conn, || {
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

    Ok(())
}
