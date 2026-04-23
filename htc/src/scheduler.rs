use std::pin::Pin;
use chrono::Utc;
use cron_parser::parse;
use thiserror::Error;

pub trait Executable {
    fn execute(&self) -> Pin<Box<dyn Future<Output = Result<(), ExecutionResult>> + Send + '_>>;
}

#[derive(Debug, Error)]
pub enum ExecutionResult {
    #[error("Success !!")]
    Success,
    #[error("Failed : {0}")]
    Failure(String),
}


pub struct SchedulableAction<A>
where
    A: Executable,
{
    executable: A,
    schedule: String,
}

impl<A> SchedulableAction<A>
where
    A: Executable,
{
    pub fn new(executable: A, schedule: String) -> Self {
        Self {
            executable,
            schedule,
        }
    }
    pub async fn schedule(&self) -> Result<(), ExecutionResult> {
        loop {
            let now = Utc::now();
            let next = parse(&self.schedule, &now)
                .map_err(|e| ExecutionResult::Failure(format!("Invalid cron expression: {e}")))?;
            let delay = next - now;
            println!("Next schedule in {}", delay);
            if let Ok(std_duration) = delay.to_std() {
                tokio::select! {
                    _ = tokio::time::sleep(std_duration) => {}
                    _ = tokio::signal::ctrl_c() => { return Ok(()); }
                }
            }
            self.executable.execute().await?;
        }
    }
}
