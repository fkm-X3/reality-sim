use rand::Rng;
use serde::{Deserialize, Serialize};

use super::activation::{Activation, ActivationFn};

/// A dense (fully connected) layer in the neural network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenseLayer {
    pub weights: Vec<f32>,
    pub biases: Vec<f32>,
    pub input_size: usize,
    pub output_size: usize,
    pub activation: Activation,
    // Cached for backprop/plasticity
    #[serde(skip)]
    pub last_inputs: Vec<f32>,
    #[serde(skip)]
    pub last_outputs: Vec<f32>,
}

impl DenseLayer {
    /// Create a new dense layer with Xavier initialization
    pub fn new(input_size: usize, output_size: usize, activation: Activation) -> Self {
        let mut rng = rand::thread_rng();
        let scale = (2.0 / (input_size + output_size) as f32).sqrt();

        let weights: Vec<f32> = (0..input_size * output_size)
            .map(|_| rng.gen_range(-scale..scale))
            .collect();

        let biases = vec![0.0; output_size];

        Self {
            weights,
            biases,
            input_size,
            output_size,
            activation,
            last_inputs: Vec::with_capacity(input_size),
            last_outputs: Vec::with_capacity(output_size),
        }
    }

    /// Forward pass through the layer
    pub fn forward(&mut self, inputs: &[f32]) -> Vec<f32> {
        debug_assert_eq!(inputs.len(), self.input_size);

        self.last_inputs.clear();
        self.last_inputs.extend_from_slice(inputs);

        let mut outputs = vec![0.0; self.output_size];

        // Matrix multiplication: output = weights * input + bias
        for j in 0..self.output_size {
            let mut sum = self.biases[j];
            for i in 0..self.input_size {
                sum += self.weights[j * self.input_size + i] * inputs[i];
            }
            outputs[j] = self.activation.activate(sum);
        }

        self.last_outputs.clear();
        self.last_outputs.extend_from_slice(&outputs);

        outputs
    }

    /// Forward pass without caching (for inference only)
    #[inline]
    pub fn forward_inference(&self, inputs: &[f32], outputs: &mut [f32]) {
        debug_assert_eq!(inputs.len(), self.input_size);
        debug_assert_eq!(outputs.len(), self.output_size);

        for j in 0..self.output_size {
            let mut sum = self.biases[j];
            for i in 0..self.input_size {
                sum += self.weights[j * self.input_size + i] * inputs[i];
            }
            outputs[j] = self.activation.activate(sum);
        }
    }

    /// Get weight at position (input_idx, output_idx)
    #[inline]
    pub fn get_weight(&self, input_idx: usize, output_idx: usize) -> f32 {
        self.weights[output_idx * self.input_size + input_idx]
    }

    /// Set weight at position (input_idx, output_idx)
    #[inline]
    pub fn set_weight(&mut self, input_idx: usize, output_idx: usize, value: f32) {
        self.weights[output_idx * self.input_size + input_idx] = value;
    }

    /// Adjust weight by delta at position (input_idx, output_idx)
    #[inline]
    pub fn adjust_weight(&mut self, input_idx: usize, output_idx: usize, delta: f32) {
        self.weights[output_idx * self.input_size + input_idx] += delta;
    }

    /// Clip weights to prevent explosion
    pub fn clip_weights(&mut self, min: f32, max: f32) {
        for w in &mut self.weights {
            *w = w.clamp(min, max);
        }
        for b in &mut self.biases {
            *b = b.clamp(min, max);
        }
    }

    /// Add Gaussian noise to weights (for exploration/mutation)
    pub fn add_noise(&mut self, std_dev: f32) {
        let mut rng = rand::thread_rng();
        for w in &mut self.weights {
            *w += rng.gen::<f32>() * std_dev * 2.0 - std_dev;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = DenseLayer::new(10, 5, Activation::ReLU);
        assert_eq!(layer.input_size, 10);
        assert_eq!(layer.output_size, 5);
        assert_eq!(layer.weights.len(), 50);
        assert_eq!(layer.biases.len(), 5);
    }

    #[test]
    fn test_forward_pass() {
        let mut layer = DenseLayer::new(3, 2, Activation::Linear);
        // Set known weights for testing
        layer.weights = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        layer.biases = vec![0.0, 0.0];

        let inputs = vec![1.0, 2.0, 3.0];
        let outputs = layer.forward(&inputs);

        assert_eq!(outputs.len(), 2);
        assert!((outputs[0] - 1.0).abs() < 0.001); // 1*1 + 0*2 + 0*3
        assert!((outputs[1] - 2.0).abs() < 0.001); // 0*1 + 1*2 + 0*3
    }
}
