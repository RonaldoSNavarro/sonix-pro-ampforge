//! Biblioteca compartilhada de 10 sub-modelos físicos valvulados (Spec 077).
//!
//! Todos os sub-modelos operam em `f64` interno, são `Copy` / inline sem
//! alocação na heap e mantêm segurança estrita em tempo real (zero-alloc, zero-lock).

pub mod bias;
pub mod cathode_follower;
pub mod output_transformer;
pub mod pentode;
pub mod phase_inverter;
pub mod power_stage;
pub mod power_supply;
pub mod reactive_load;
pub mod triode;

pub use bias::{BiasMode, BiasTopology};
pub use cathode_follower::CathodeFollowerStage;
pub use output_transformer::{OutputTransformer, OutputTransformerModel};
pub use pentode::PentodePreampStage;
pub use phase_inverter::PhaseInverterLtpStage;
pub use power_stage::PushPullPowerStage;
pub use power_supply::{PowerSupplyAndRectifier, PowerSupplySag, RectifierKind};
pub use reactive_load::ReactiveLoadAndNfb;
pub use triode::{TriodeParams, TriodeStage};
