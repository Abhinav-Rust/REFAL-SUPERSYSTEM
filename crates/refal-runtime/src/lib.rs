//! Runtime object model and pattern matcher for Refal execution.

pub mod interpreter;
pub mod matcher;
pub mod value;

pub use interpreter::{EvalError, Evaluator};
pub use matcher::{Bindings, MatchError, match_pattern};
pub use value::{Slice, Value};
