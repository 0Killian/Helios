mod devices;
mod services;

pub use devices::*;
pub use services::*;

pub trait Repository<UnitOfWorkProvider> {}

#[async_trait::async_trait]
pub trait UnitOfWorkProvider: Sync {
    type UnitOfWork<'a>: Send
    where
        Self: 'a;

    async fn begin_transaction<'a>(&'a self) -> Self::UnitOfWork<'a>;
    async fn commit<'a>(&'a self, unit_of_work: Self::UnitOfWork<'a>);
    async fn rollback<'a>(&'a self, unit_of_work: Self::UnitOfWork<'a>);
}
