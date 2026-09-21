pub mod capability;
pub mod catalog;
pub mod diagnostics;
pub mod fixture;
pub mod manifest;
pub mod state;

pub use capability::CapabilityClass;
pub use catalog::FixtureCatalog;
pub use diagnostics::{DiagnosticEntry, DiagnosticLevel};
pub use fixture::{
    FixtureDef, FixtureDeterminism, FixtureGroup, FixtureId, FixtureIdError, FixtureProvenance,
};
pub use manifest::RenderLabManifest;
pub use state::{
    ClockMode, MotionVariant, QualityLevel, RenderLabEvent, RenderLabState, TopologyVariant,
};

pub const RENDER_LAB_SCHEMA_VERSION: u32 = 1;
