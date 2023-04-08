pub fn uuid_to_sqlx(uuid: uuid::Uuid) -> sqlx::types::Uuid {
    sqlx::types::Uuid::from_u128(uuid.as_u128())
}

pub fn uuid_from_sqlx(uuid: sqlx::types::Uuid) -> uuid::Uuid {
    uuid::Uuid::from_u128(uuid.as_u128())
}
