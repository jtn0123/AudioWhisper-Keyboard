//! Shared keyboard dispatch policy. Hosts supply clocks and execute complete ordered plans.
#![forbid(unsafe_code)]
pub mod control;
pub mod delete;
pub mod feedback;
pub mod lifecycle;
pub mod pointer;
pub mod protocol;
pub mod routing;
pub mod visual;
