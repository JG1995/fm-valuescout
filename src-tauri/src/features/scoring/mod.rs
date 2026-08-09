pub mod catalog;
pub mod combine;
#[allow(dead_code)] // Read-model commits consume this public projection API next.
pub mod projection;
pub mod score;

mod projection_profiles;
