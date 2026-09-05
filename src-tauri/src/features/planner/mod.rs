pub mod commands;
pub mod depth;
pub(crate) mod fit;
mod optimizer;
mod role_reference;
pub mod squad;
pub mod suggested_training;
pub mod tactic;
pub(crate) mod teams;

#[cfg(test)]
mod depth_tests;
#[cfg(test)]
mod fit_tests;
#[cfg(test)]
mod optimizer_tests;
#[cfg(test)]
mod role_reference_tests;
#[cfg(test)]
mod squad_tests;
#[cfg(test)]
mod teams_tests;
#[cfg(test)]
mod test_support;
