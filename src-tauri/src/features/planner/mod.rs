pub mod commands;
pub mod depth;
mod optimizer;
pub mod squad;
pub mod tactic;
mod teams;

#[cfg(test)]
mod depth_tests;
#[cfg(test)]
mod optimizer_tests;
#[cfg(test)]
mod squad_tests;
#[cfg(test)]
mod teams_tests;
#[cfg(test)]
mod test_support;
