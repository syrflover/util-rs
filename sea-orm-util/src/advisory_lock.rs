use std::time::Duration;

use futures::Future;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

/// Refs: https://www.postgresql.org/docs/14/functions-admin.html#FUNCTIONS-ADVISORY-LOCKS
pub async fn advisory_lock<'a, F, R, E>(
    unique_key: &str,
    conn: &'a DatabaseConnection,
    f: fn(&'a DatabaseConnection) -> F,
) -> Result<bool, E>
where
    F: Future<Output = Result<R, E>>,
{
    let unique_key = make_unique_key(unique_key);

    let locked = lock(unique_key, conn)
        .await
        .expect("failed to pg_advisory_lock");

    let r = match locked {
        true => f(conn).await.map(|_| true),
        false => {
            // waiting for unlock
            loop {
                let locked = lock(unique_key, conn)
                    .await
                    .expect("failed to pg_advisory_lock");

                if locked {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }

            Ok::<_, E>(false)
        }
    };

    let _was_locked = unlock(unique_key, conn)
        .await
        .expect("failed to pg_advisory_unlock");

    r
}

async fn lock(unique_key: u32, conn: &DatabaseConnection) -> Result<bool, sea_orm::error::DbErr> {
    let backend = conn.get_database_backend();

    let locked = conn
        .query_one(Statement::from_string(
            backend,
            format!("SELECT pg_try_advisory_lock({unique_key}) as locked"),
        ))
        .await?
        .expect("failed to pg_advisory_lock")
        .try_get("", "locked")?;

    Ok(locked)
}

async fn unlock(unique_key: u32, conn: &DatabaseConnection) -> Result<bool, sea_orm::error::DbErr> {
    let backend = conn.get_database_backend();

    let was_locked = conn
        .query_one(Statement::from_string(
            backend,
            format!("SELECT pg_advisory_unlock({unique_key}) as was_locked"),
        ))
        .await?
        .expect("failed to pg_advisory_unlock")
        .try_get("", "was_locked")?;

    Ok(was_locked)
}

fn make_unique_key(x: &str) -> u32 {
    x.chars().map(|x| x as u32).sum()
}

#[test]
fn test_make_unique_key() {
    let x = make_unique_key("http://localhost:12345/avbcjkd-@m!");

    assert_eq!(x, 2905);
}
