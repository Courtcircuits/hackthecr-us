use std::{future::Future, pin::Pin};

use thiserror::Error;

pub mod meals;
pub mod restaurants;
pub mod schools;
pub mod schedule;
pub mod config_gen;


pub trait Executable {
    fn execute(&self) -> Pin<Box<dyn Future<Output = Result<(), ExecutionResult>> + Send + '_>>;
}

#[derive(Debug, Error)]
pub enum ExecutionResult {
    #[error("Success !!")]
    Success,
    #[error("Failed : {0}")]
    Failure(String)
}


