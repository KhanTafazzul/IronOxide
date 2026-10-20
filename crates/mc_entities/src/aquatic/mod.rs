// REA Verified: net.minecraft.world.entity.animal (mapped: bxx.class)
//
// Aquatic, Amphibious & Fluid Mobs:
// - WaterPathNavigation: 3D swimming with water drag and breath/drowning meters.
// - Fish & Cephalopods: Cod, Salmon, Pufferfish, TropicalFish, Squid, GlowSquid.
// - Amphibious: Turtle, Frog, Tadpole, Axolotl.
// - Nether Fluid: Strider.

pub mod amphibious;
pub mod fish;
pub mod fluid;
pub mod water_navigation;

pub use amphibious::*;
pub use fish::*;
pub use fluid::*;
pub use water_navigation::*;
