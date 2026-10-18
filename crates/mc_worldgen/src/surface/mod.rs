//! Surface Rule Engine & Biome Strata AST.
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceRules (dph.class)
//! REA Reference: net.minecraft.world.level.levelgen.SurfaceSystem (dpi.class)

pub mod ast;
pub mod rules;

pub use ast::{Condition, SurfaceContext, SurfaceRule};
pub use rules::{build_overworld_rules, SurfaceSystem};
