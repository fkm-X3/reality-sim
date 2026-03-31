//! Reality Simulator - A Humanity Simulator with Neural Network Agents
//!
//! This crate provides a framework for simulating thousands of agents
//! that perceive their environment, process stimuli through neural networks,
//! and persist their experiences to JSON.

pub mod agent;
pub mod config;
pub mod neural;
pub mod persistence;
pub mod simulation;
pub mod world;

pub use config::WorldConfig;
