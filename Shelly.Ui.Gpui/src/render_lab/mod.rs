pub mod capability;
pub mod catalog;
pub mod confront;
pub mod diagnostics;
pub mod fixture;
pub mod ledger;
pub mod manifest;
pub mod recipe;
pub mod state;
pub mod texture;

pub use capability::CapabilityClass;
pub use catalog::FixtureCatalog;
pub use diagnostics::{DiagnosticEntry, DiagnosticLevel};
pub use fixture::{
    FixtureDef, FixtureDeterminism, FixtureGroup, FixtureId, FixtureIdError, FixtureProvenance,
};
pub use ledger::{CapabilityLedger, CapabilityObservation, STOCK_BACKEND_ID};
pub use manifest::RenderLabManifest;
pub use state::{
    ClockMode, MotionVariant, QualityLevel, RenderLabEvent, RenderLabState, TopologyVariant,
};

pub const RENDER_LAB_SCHEMA_VERSION: u32 = 2;
