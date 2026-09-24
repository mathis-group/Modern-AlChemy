// Global Imports
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ExpressionType {
    UntypedLambda,
    SimplyTypedLambda,
    Haskell,
}