

use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct AccountRepository {
    pool: PgPool,
}

impl AccountRepository {
    pub fn new(pool: PgPool) -> AccountRepository {
        AccountRepository { pool }
    }
}
