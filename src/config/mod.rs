use error::AppError;
use once_cell::sync::Lazy;
use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource;

use crate::constants::{APP_NAME, ENVIRONMENT};

pub type MangJooResult<T> = std::result::Result<T, AppError>;

pub static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![
        KeyValue::new(resource::SERVICE_NAME, APP_NAME),
        KeyValue::new(resource::SERVICE_VERSION, "0.0"),
        KeyValue::new(resource::TELEMETRY_SDK_LANGUAGE, "rust"),
        KeyValue::new(resource::TELEMETRY_SDK_NAME, "opentelemetry"),
        KeyValue::new(resource::TELEMETRY_SDK_VERSION, "0.27.1"),
        KeyValue::new("deployment.environment", *ENVIRONMENT),
    ])
});

pub mod app_state;
pub mod db;
pub mod error;
pub mod hash;
pub mod jwt;
pub mod session;
pub mod telemetry;
