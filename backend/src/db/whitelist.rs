

use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct WhitelistRepository {
    pool: PgPool,
}

impl WhitelistRepository {
    pub fn new(pool: PgPool) -> WhitelistRepository {
        WhitelistRepository { pool }
    }
}
