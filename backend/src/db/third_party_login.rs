

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
