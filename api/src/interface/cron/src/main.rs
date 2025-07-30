use std::{sync::Arc, time::Instant};

use common::{CONFIG, RouterKind};
use domain::{PeriodicUseCase, SyncDevicesUseCase};
use repositories::{PostgresDevicesRepository, PostgresUWP};
use router_api::bouygues::BboxRouterApi;
use sqlx::PgPool;
use tokio::sync::Mutex;

struct CronJob {
    job: Box<dyn PeriodicUseCase>,
    next_execution: Option<Instant>,
}

impl CronJob {
    fn new(job: Box<dyn PeriodicUseCase>) -> Self {
        let next_execution = job.next_execution();

        CronJob {
            job,
            next_execution,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router_api = Arc::new(match CONFIG.router_api.kind {
        RouterKind::Bbox => {
            BboxRouterApi::new(
                CONFIG.router_api.base_url.clone(),
                CONFIG.router_api.password.clone(),
            )
            .await?
        }
    });

    let pg_pool = Arc::new(Mutex::new(
        PgPool::connect(CONFIG.database.url.as_str()).await?,
    ));

    let unit_of_work_provider = PostgresUWP::new(pg_pool);

    let mut jobs: Vec<CronJob> = vec![CronJob::new(Box::new(SyncDevicesUseCase::<
        PostgresDevicesRepository,
        PostgresUWP,
    >::new(
        unit_of_work_provider, router_api
    )))];

    loop {
        let now = Instant::now();
        let mut next_execution = jobs[0]
            .next_execution
            .ok_or_else(|| anyhow::anyhow!("No next execution found"))?;

        for job in &mut jobs {
            if let Some(next_exec) = job.next_execution {
                if next_exec <= now {
                    job.job.execute().await;
                    let next_exec_optional = job.job.next_execution();

                    if let Some(next_exec) = next_exec_optional {
                        if next_exec < next_execution {
                            next_execution = next_exec;
                        }
                    }

                    job.next_execution = next_exec_optional;
                }
            }
        }

        tokio::time::sleep_until(next_execution.into()).await;
    }
}
