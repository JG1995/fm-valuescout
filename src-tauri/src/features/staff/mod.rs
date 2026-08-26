#[cfg_attr(not(test), allow(dead_code))]
mod assignment_optimizer;
mod assignment_optimizer_query;
#[cfg(test)]
mod assignment_optimizer_query_tests;
#[cfg(test)]
mod assignment_optimizer_tests;
pub(crate) mod assignment_targets;
pub mod commands;
pub mod filter;
mod metrics;
pub mod query;
pub mod scoring;
pub mod service;
