//! Neural network module for agent cognition

mod activation;
mod layer;
mod network;
mod plasticity;

pub use activation::{Activation, ActivationFn};
pub use layer::DenseLayer;
pub use network::{NeuralNet, NeuralNetwork};
pub use plasticity::{EvolutionarySelector, LearningRule, PlasticityConfig};
