// REA Verified: net.minecraft.world.entity.monster (mapped: bxu.class)
//
// Hostile Mob Registry, Attack Logic & Projectile Physics:
// - Overworld Undead & Arachnids: Zombie, Husk, Drowned, Skeleton, Stray, Bogged, Spider, CaveSpider, Creeper, Phantom, Witch.
// - Illagers & Raid Monsters: Pillager, Vindicator, Evoker, Vex, Ravager.
// - Nether Hostiles: Ghast, Blaze, MagmaCube, Piglin, ZombifiedPiglin.
// - Specialized Monsters: Guardian, ElderGuardian, Warden, Breeze, Creaking.
// - Projectiles & Explosions: ArrowProjectile, FireballProjectile, WindChargeProjectile, EvokerFangs, VoxelExplosion.

pub mod illagers;
pub mod nether;
pub mod projectiles;
pub mod specialized;
pub mod undead_arachnid;

pub use illagers::*;
pub use nether::*;
pub use projectiles::*;
pub use specialized::*;
pub use undead_arachnid::*;
