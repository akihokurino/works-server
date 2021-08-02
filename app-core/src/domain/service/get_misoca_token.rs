use crate::ddb;
use crate::domain;
use crate::misoca;
use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};
use diesel::MysqlConnection;

pub async fn exec(
    conn: &MysqlConnection,
    user_dao: ddb::Dao<domain::user::User>,
    misoca_cli: misoca::Client,
    user_id: String,
    now: DateTime<Utc>,
) -> CoreResult<String> {
    let mut user = user_dao.get(conn, user_id)?;

    if user.misoca_refresh_token.is_empty() {
        return Err(CoreError::Forbidden);
    }

    let tokens = misoca_cli
        .refresh_tokens(misoca::tokens::refresh_tokens::Input {
            refresh_token: user.misoca_refresh_token.clone(),
        })
        .await?;

    let access_token = tokens.access_token;
    let refresh_token = tokens.refresh_token;

    user.update_misoca_refresh_token(refresh_token, now);
    user_dao.tx(conn, || {
        user_dao.update(conn, &user)?;
        Ok(())
    })?;

    Ok(access_token)
}
