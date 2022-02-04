

use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct LoginProviderRepository {
    pool: PgPool,
}

impl LoginProviderRepository {
    pub fn new(pool: PgPool) -> LoginProviderRepository {
        LoginProviderRepository { pool }
    }
}
