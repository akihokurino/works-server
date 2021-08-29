use crate::ddb::Tx;
use crate::domain;
use crate::graphql;
use crate::misoca;
use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};

pub async fn exec(ctx: &graphql::Context, now: DateTime<Utc>) -> CoreResult<String> {
    let user_dao = ctx.ddb_dao::<domain::user::User>();
    let conn = ctx.get_new_connection();
    let misoca_cli = ctx.misoca_cli.clone();
    let authenticated_user_id = ctx
        .authenticated_user_id
        .clone()
        .ok_or(CoreError::UnAuthenticate)?;

    let mut user = user_dao.get(&conn, authenticated_user_id.clone())?;

    if user.misoca_refresh_token.is_empty() {
        return Err(CoreError::BadRequest(
            "misocaへの接続が必要です".to_string(),
        ));
    }

    let tokens = misoca_cli
        .refresh_tokens(misoca::tokens::refresh_tokens::Input {
            refresh_token: user.misoca_refresh_token.clone(),
        })
        .await?;

    let access_token = tokens.access_token;
    let refresh_token = tokens.refresh_token;

    user.update_misoca_refresh_token(refresh_token, now);
    Tx::run(&conn, || {
        user_dao.update(&conn, &user)?;
        Ok(())
    })?;

    Ok(access_token)
}
