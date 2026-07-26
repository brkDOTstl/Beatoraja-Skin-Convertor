pub mod definitions;
mod dxa;
mod edit;
pub mod engine;
pub mod error;
mod model;
mod path_rewrite;
mod rules;
mod theme;
mod workspace;

pub use definitions::{RuleDefinition, all_rule_definitions, preset_rule_ids};
pub use engine::{dry_run, generate};
pub use error::{RuleError, RuleResult};
pub use model::{
    AspectProtection, ConversionOptions, ConversionReport, FileChange, OutputManifest, RuleWarning,
    TargetAspect,
};
