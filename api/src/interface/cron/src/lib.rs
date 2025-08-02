use std::{sync::Arc, time::Instant};

use domain::{AgentPing, PeriodicUseCase, SyncDevicesUseCase};
use ports::{
    agent_connection::AgentConnectionManager,
    api::RouterApi,
    repositories::{DevicesRepository, UnitOfWorkProvider},
};
use thiserror::Error;
use tracing::info;

struct CronJob {
    name: &'static str,
    job: Box<dyn PeriodicUseCase>,
    next_execution: Option<Instant>,
}

impl CronJob {
    fn new(name: &'static str, job: Box<dyn PeriodicUseCase>) -> Self {
        CronJob {
            name,
            job,
            next_execution: Some(Instant::now()),
        }
    }
}

#[derive(Error, Debug)]
pub enum CronError {
    #[error("Failed to initialize cron service")]
    InitializationFailed,
}

pub async fn cron<DR: DevicesRepository<UWP> + 'static, UWP: UnitOfWorkProvider + 'static>(
    router_api: Arc<dyn RouterApi>,
    unit_of_work_provider: UWP,
    acm: Arc<dyn AgentConnectionManager>,
) -> Result<impl Future<Output = ()>, CronError> {
    tracing::info!("Starting cron service");

    let mut jobs: Vec<CronJob> = vec![
        CronJob::new(
            "Sync Devices",
            Box::new(SyncDevicesUseCase::<DR, UWP>::new(
                unit_of_work_provider,
                router_api,
            )),
        ),
        CronJob::new("Agent Ping", Box::new(AgentPing::new(acm))),
    ];

    let mut next_execution = jobs[0]
        .next_execution
        .ok_or_else(|| CronError::InitializationFailed)?;

    Ok(async move {
        loop {
            let now = Instant::now();

            for job in &mut jobs {
                if let Some(next_exec) = job.next_execution {
                    if next_exec <= now {
                        info!("Executing cron job {}", job.name);
                        job.job.execute().await;
                        let next_exec_optional = job.job.next_execution();

                        if let Some(next_exec) = next_exec_optional {
                            info!(
                                "Job finished, next execution in {}s",
                                (next_exec - now).as_secs()
                            );
                            if next_exec < next_execution {
                                next_execution = next_exec;
                            }
                        } else {
                            info!("Job finished, no next execution");
                        }

                        job.next_execution = next_exec_optional;
                    }
                }
            }

            tokio::time::sleep_until(next_execution.into()).await;
        }
    })
}
