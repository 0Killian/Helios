mod devices;
mod services;

pub use devices::*;
pub use services::*;
use thiserror::Error;

pub trait Repository<UnitOfWorkProvider> {}

#[async_trait::async_trait]
pub trait UnitOfWorkProvider: Sync {
    type UnitOfWork<'a>: Send
    where
        Self: 'a;

    async fn begin_transaction<'a>(&'a self) -> RepositoryResult<Self::UnitOfWork<'a>>;
    async fn commit<'a>(&'a self, unit_of_work: Self::UnitOfWork<'a>) -> RepositoryResult<()>;
    async fn rollback<'a>(&'a self, unit_of_work: Self::UnitOfWork<'a>) -> RepositoryResult<()>;
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    #[error("The requested resource could not be found.")]
    NotFound,

    #[error("A unique constraint was violated. The resource likely already exists.")]
    UniqueViolation,

    #[error("A foreign key constraint was violated. A related resource does not exist.")]
    ForeignKeyViolation,

    #[error("A check constraint was violated. The data is invalid.")]
    CheckViolation,

    #[error("Could not connect to the database or the connection was lost.")]
    ConnectionFailed,

    #[error("An unexpected database error occurred.")]
    Unknown,
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;
