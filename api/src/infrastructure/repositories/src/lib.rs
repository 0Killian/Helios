mod devices;
mod services;

pub use devices::*;
use entities::SharedLockedReference;
use ports::repositories::{RepositoryError, RepositoryResult, UnitOfWorkProvider};
pub use services::*;
use sqlx::PgTransaction;

type PostgresUoW<'a> = PgTransaction<'a>;

#[derive(Clone)]
pub struct PostgresUWP {
    pool: SharedLockedReference<sqlx::PgPool>,
}

impl PostgresUWP {
    pub fn new(pool: SharedLockedReference<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UnitOfWorkProvider for PostgresUWP {
    type UnitOfWork<'a>
        = PostgresUoW<'a>
    where
        Self: 'a;

    async fn begin_transaction<'a>(&'a self) -> RepositoryResult<PostgresUoW<'a>> {
        let pool = self.pool.lock().await;
        pool.begin().await.map_err(map_sqlx_error)
    }

    async fn commit<'a>(&'a self, uow: PostgresUoW<'a>) -> RepositoryResult<()> {
        uow.commit().await.map_err(map_sqlx_error)
    }

    async fn rollback<'a>(&'a self, uow: PostgresUoW<'a>) -> RepositoryResult<()> {
        uow.rollback().await.map_err(map_sqlx_error)
    }
}

pub(crate) fn map_sqlx_error(err: sqlx::Error) -> RepositoryError {
    match err {
        sqlx::Error::RowNotFound => RepositoryError::NotFound,

        sqlx::Error::Io(_)
        | sqlx::Error::Tls(_)
        | sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed
        | sqlx::Error::Configuration(_)
        | sqlx::Error::Protocol(_)
        | sqlx::Error::BeginFailed => {
            println!(
                "An error occurred while connecting to the database: {}",
                err
            );
            RepositoryError::ConnectionFailed
        }

        sqlx::Error::Database(db_err) => {
            if db_err.is_unique_violation() {
                return RepositoryError::UniqueViolation;
            }
            if db_err.is_foreign_key_violation() {
                return RepositoryError::ForeignKeyViolation;
            }
            if db_err.is_check_violation() {
                return RepositoryError::CheckViolation;
            }

            println!("An unhandled database error occurred");
            RepositoryError::Unknown
        }

        _ => {
            println!("An unhandled error occurred: {}", err);
            RepositoryError::Unknown
        }
    }
}
