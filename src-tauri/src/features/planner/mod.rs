pub mod commands;
pub mod depth;
pub(crate) mod fit;
mod fm_moves;
mod fm_training;
mod loans;
pub(crate) mod optimizer;
mod outlook;
mod role_reference;
pub mod squad;
mod standout;
pub mod suggested_training;
pub mod tactic;
pub(crate) mod teams;

#[cfg(test)]
mod depth_tests;
#[cfg(test)]
mod fit_tests;
#[cfg(test)]
mod fm_moves_tests;
#[cfg(test)]
mod fm_training_tests;
#[cfg(test)]
mod loans_tests;
#[cfg(test)]
mod optimizer_tests;
#[cfg(test)]
mod outlook_tests;
#[cfg(test)]
mod role_reference_tests;
#[cfg(test)]
mod squad_tests;
#[cfg(test)]
mod standout_tests;
#[cfg(test)]
mod teams_tests;
#[cfg(test)]
mod test_support;
