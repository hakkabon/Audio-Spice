use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimError {
    #[error("Singular matrix at {freq} Hz (condition number: {cond})")]
    SingularMatrix { freq: f64, cond: f64 },

    #[error("DC convergence failed after {iter} iterations (tolerance: {tol})")]
    NoConvergence { iter: usize, tol: f64 },

    #[error("Transient simulation failed at t={time}: {reason}")]
    TransientError { time: f64, reason: String },

    #[error("Parse error at line {line}: {msg}")]
    ParseError { line: usize, msg: String },

    #[error("Component not found: {name}")]
    ComponentNotFound { name: String },

    #[error("Node index out of bounds: {node} (max: {max})")]
    NodeOutOfBounds { node: usize, max: usize },

    #[error("Model not found: {name}")]
    ModelNotFound { name: String },

    #[error("Invalid component value: {value} for {name}")]
    InvalidValue { name: String, value: String },

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<String> for SimError {
    fn from(s: String) -> Self {
        SimError::Internal(s)
    }
}

impl From<&str> for SimError {
    fn from(s: &str) -> Self {
        SimError::Internal(s.to_string())
    }
}

pub type SimResult<T> = Result<T, SimError>;