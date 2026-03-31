use serde::{Deserialize, Serialize};

use super::activation::Activation;
use super::layer::DenseLayer;
use super::plasticity::{LearningRule, PlasticityConfig};

/// Neural network for agent decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralNetwork {
    pub layers: Vec<DenseLayer>,
    pub plasticity: PlasticityConfig,
    // Cached buffers for inference (avoid allocations)
    #[serde(skip)]
    buffer_a: Vec<f32>,
    #[serde(skip)]
    buffer_b: Vec<f32>,
}

/// Trait for neural network operations
pub trait NeuralNet: Send + Sync {
    fn forward(&mut self, inputs: &[f32]) -> Vec<f32>;
    fn forward_inference(&self, inputs: &[f32]) -> Vec<f32>;
    fn adjust_weights(&mut self, reward: f32);
    fn export_latent(&self) -> Vec<f32>;
    fn import_latent(&mut self, latent: &[f32]);
    fn mutate(&mut self, mutation_rate: f32);
}

impl NeuralNetwork {
    /// Create a new neural network with the specified architecture
    pub fn new(layer_sizes: &[usize], hidden_activation: Activation, output_activation: Activation) -> Self {
        assert!(layer_sizes.len() >= 2, "Need at least input and output layers");

        let mut layers = Vec::with_capacity(layer_sizes.len() - 1);

        for i in 0..layer_sizes.len() - 1 {
            let activation = if i == layer_sizes.len() - 2 {
                output_activation
            } else {
                hidden_activation
            };
            layers.push(DenseLayer::new(layer_sizes[i], layer_sizes[i + 1], activation));
        }

        let max_size = *layer_sizes.iter().max().unwrap();

        Self {
            layers,
            plasticity: PlasticityConfig::default(),
            buffer_a: vec![0.0; max_size],
            buffer_b: vec![0.0; max_size],
        }
    }

    /// Create a standard agent network (8 inputs -> 16 -> 16 -> 8 outputs)
    pub fn new_agent_network() -> Self {
        Self::new(
            &[8, 16, 16, 8],
            Activation::LeakyReLU,
            Activation::Tanh,
        )
    }

    /// Create a network with custom input/output sizes
    pub fn new_custom(input_size: usize, output_size: usize, hidden_sizes: &[usize]) -> Self {
        let mut sizes = Vec::with_capacity(hidden_sizes.len() + 2);
        sizes.push(input_size);
        sizes.extend_from_slice(hidden_sizes);
        sizes.push(output_size);

        Self::new(&sizes, Activation::LeakyReLU, Activation::Tanh)
    }

    fn ensure_buffers(&mut self) {
        let max_size = self.layers.iter()
            .map(|l| l.input_size.max(l.output_size))
            .max()
            .unwrap_or(0);
        
        if self.buffer_a.len() < max_size {
            self.buffer_a.resize(max_size, 0.0);
            self.buffer_b.resize(max_size, 0.0);
        }
    }

    /// Get total number of parameters (weights + biases)
    pub fn parameter_count(&self) -> usize {
        self.layers.iter()
            .map(|l| l.weights.len() + l.biases.len())
            .sum()
    }

    /// Clone weights from another network
    pub fn clone_from(&mut self, other: &NeuralNetwork) {
        for (self_layer, other_layer) in self.layers.iter_mut().zip(other.layers.iter()) {
            self_layer.weights.copy_from_slice(&other_layer.weights);
            self_layer.biases.copy_from_slice(&other_layer.biases);
        }
    }

    /// Crossover with another network (for genetic algorithms)
    pub fn crossover(&self, other: &NeuralNetwork, crossover_rate: f32) -> NeuralNetwork {
        let mut child = self.clone();
        let mut rng = rand::thread_rng();

        for (child_layer, other_layer) in child.layers.iter_mut().zip(other.layers.iter()) {
            for (cw, ow) in child_layer.weights.iter_mut().zip(other_layer.weights.iter()) {
                if rand::Rng::gen::<f32>(&mut rng) < crossover_rate {
                    *cw = *ow;
                }
            }
        }

        child
    }
}

impl NeuralNet for NeuralNetwork {
    /// Forward pass through all layers (training mode - caches activations)
    fn forward(&mut self, inputs: &[f32]) -> Vec<f32> {
        let mut current = inputs.to_vec();
        for layer in &mut self.layers {
            current = layer.forward(&current);
        }
        current
    }

    /// Forward pass for inference only (no caching, more efficient)
    fn forward_inference(&self, inputs: &[f32]) -> Vec<f32> {
        let mut current = inputs.to_vec();
        let mut next = Vec::new();

        for layer in &self.layers {
            next.resize(layer.output_size, 0.0);
            layer.forward_inference(&current, &mut next);
            std::mem::swap(&mut current, &mut next);
        }

        current
    }

    /// Adjust weights based on reward signal
    fn adjust_weights(&mut self, reward: f32) {
        let lr = self.plasticity.learning_rate;
        let decay = self.plasticity.weight_decay;

        match self.plasticity.rule {
            LearningRule::Hebbian => {
                // Hebbian: strengthen connections between co-active neurons
                for layer in &mut self.layers {
                    if layer.last_inputs.is_empty() || layer.last_outputs.is_empty() {
                        continue;
                    }
                    for j in 0..layer.output_size {
                        for i in 0..layer.input_size {
                            let delta = lr * reward * layer.last_inputs[i] * layer.last_outputs[j];
                            layer.adjust_weight(i, j, delta);
                        }
                    }
                }
            }
            LearningRule::RewardModulated => {
                // Simple reward-modulated learning
                for layer in &mut self.layers {
                    for w in &mut layer.weights {
                        *w += lr * reward * (*w).signum();
                        *w *= 1.0 - decay; // Weight decay
                    }
                }
            }
            LearningRule::Evolutionary => {
                // No online learning - weights adjusted through selection
            }
        }

        // Clip weights to prevent explosion
        for layer in &mut self.layers {
            layer.clip_weights(-5.0, 5.0);
        }
    }

    /// Export hidden layer activations as latent representation
    fn export_latent(&self) -> Vec<f32> {
        // Export the weights of the first hidden layer as "knowledge"
        if self.layers.len() >= 2 {
            self.layers[0].weights.clone()
        } else {
            Vec::new()
        }
    }

    /// Import latent representation to influence network
    fn import_latent(&mut self, latent: &[f32]) {
        if !self.layers.is_empty() && latent.len() == self.layers[0].weights.len() {
            // Blend imported latent with current weights
            for (w, l) in self.layers[0].weights.iter_mut().zip(latent.iter()) {
                *w = *w * 0.8 + *l * 0.2; // 80% current, 20% imported
            }
        }
    }

    /// Mutate weights for evolutionary algorithms
    fn mutate(&mut self, mutation_rate: f32) {
        for layer in &mut self.layers {
            layer.add_noise(mutation_rate);
            layer.clip_weights(-5.0, 5.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_creation() {
        let net = NeuralNetwork::new(&[4, 8, 2], Activation::ReLU, Activation::Tanh);
        assert_eq!(net.layers.len(), 2);
        assert_eq!(net.layers[0].input_size, 4);
        assert_eq!(net.layers[0].output_size, 8);
        assert_eq!(net.layers[1].input_size, 8);
        assert_eq!(net.layers[1].output_size, 2);
    }

    #[test]
    fn test_forward_pass() {
        let mut net = NeuralNetwork::new(&[3, 4, 2], Activation::Linear, Activation::Linear);
        let inputs = vec![1.0, 0.5, -0.5];
        let outputs = net.forward(&inputs);
        assert_eq!(outputs.len(), 2);
    }

    #[test]
    fn test_agent_network() {
        let net = NeuralNetwork::new_agent_network();
        assert_eq!(net.layers.len(), 3);
        assert_eq!(net.layers[0].input_size, 8);
        assert_eq!(net.layers[2].output_size, 8);
    }
}
