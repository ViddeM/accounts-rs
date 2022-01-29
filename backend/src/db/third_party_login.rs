use crate::models::third_party_login::ThirdPartyLogin;
use crate::util::accounts_error::AccountsResult;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct ThirdPartyLoginRepository {
    pool: PgPool,
}

impl ThirdPartyLoginRepository {
    pub fn new(pool: PgPool) -> ThirdPartyLoginRepository {
        ThirdPartyLoginRepository { pool }
    }
}
