#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "AUTHORITY_LEVEL", rename_all = "snake_case")]
pub enum AuthorityLevel {
    User,
    Admin,
}
