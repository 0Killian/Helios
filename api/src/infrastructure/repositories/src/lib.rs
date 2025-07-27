mod devices;
mod services;

pub use devices::*;
use entities::SharedLockedReference;
use ports::repositories::UnitOfWorkProvider;
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

    async fn begin_transaction<'a>(&'a self) -> PostgresUoW<'a> {
        let pool = self.pool.lock().await;
        pool.begin().await.unwrap()
    }

    async fn commit<'a>(&'a self, uow: PostgresUoW<'a>) {
        uow.commit().await.unwrap();
    }

    async fn rollback<'a>(&'a self, uow: PostgresUoW<'a>) {
        uow.rollback().await.unwrap();
    }
}
