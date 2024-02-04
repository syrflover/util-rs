use sea_orm::{DbErr, SqlErr};

pub fn is_unique_constraint_violation(db_err: &DbErr) -> bool {
    match db_err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => true,
        _ => false,
    }
}

pub fn is_foreign_key_constraint_violation(db_err: &DbErr) -> bool {
    match db_err.sql_err() {
        Some(SqlErr::ForeignKeyConstraintViolation(_)) => true,
        _ => false,
    }
}
