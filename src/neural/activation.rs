use serde::{Deserialize, Serialize};

/// Activation function types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activation {
    ReLU,
    Sigmoid,
    Tanh,
    Linear,
    LeakyReLU,
}

/// Trait for activation functions
pub trait ActivationFn: Send + Sync {
    fn activate(&self, x: f32) -> f32;
    fn derivative(&self, x: f32) -> f32;
    fn activate_batch(&self, inputs: &[f32], outputs: &mut [f32]) {
        for (i, &x) in inputs.iter().enumerate() {
            outputs[i] = self.activate(x);
        }
    }
}

impl ActivationFn for Activation {
    #[inline]
    fn activate(&self, x: f32) -> f32 {
        match self {
            Activation::ReLU => x.max(0.0),
            Activation::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            Activation::Tanh => x.tanh(),
            Activation::Linear => x,
            Activation::LeakyReLU => if x > 0.0 { x } else { 0.01 * x },
        }
    }

    #[inline]
    fn derivative(&self, x: f32) -> f32 {
        match self {
            Activation::ReLU => if x > 0.0 { 1.0 } else { 0.0 },
            Activation::Sigmoid => {
                let s = self.activate(x);
                s * (1.0 - s)
            }
            Activation::Tanh => {
                let t = x.tanh();
                1.0 - t * t
            }
            Activation::Linear => 1.0,
            Activation::LeakyReLU => if x > 0.0 { 1.0 } else { 0.01 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relu() {
        let relu = Activation::ReLU;
        assert_eq!(relu.activate(5.0), 5.0);
        assert_eq!(relu.activate(-5.0), 0.0);
        assert_eq!(relu.activate(0.0), 0.0);
    }

    #[test]
    fn test_sigmoid() {
        let sigmoid = Activation::Sigmoid;
        assert!((sigmoid.activate(0.0) - 0.5).abs() < 0.001);
        assert!(sigmoid.activate(100.0) > 0.99);
        assert!(sigmoid.activate(-100.0) < 0.01);
    }

    #[test]
    fn test_tanh() {
        let tanh = Activation::Tanh;
        assert!((tanh.activate(0.0)).abs() < 0.001);
        assert!(tanh.activate(100.0) > 0.99);
        assert!(tanh.activate(-100.0) < -0.99);
    }
}
