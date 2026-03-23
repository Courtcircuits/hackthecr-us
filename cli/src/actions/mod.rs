use std::{future::Future, pin::Pin};

use thiserror::Error;

pub mod config_gen;
pub mod meals;
pub mod restaurants;
pub mod schedule;
pub mod schools;

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
