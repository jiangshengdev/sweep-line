pub mod geom;
pub mod preprocess;
pub mod rational;
pub mod run;
pub mod sweep;
pub mod trace;

pub use preprocess::{InputCoord, InputSegmentF64, PreprocessOutput, Warning, WarningKind, preprocess_segments};
pub use rational::Rational;
