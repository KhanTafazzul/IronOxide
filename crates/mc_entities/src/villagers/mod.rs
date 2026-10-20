// REA Verified: net.minecraft.world.entity.npc (mapped: cev.class)
//
// Villager Profession Schedule, Trading Matrices & Golem Defense:
// - VillagerSchedule: daily 0..24000 tick clock, Work, Gather, Sleep, Rest, and Panic.
// - VillagerProfession: 15 professions, workstation POI binding, locking & leveling.
// - VillagerTrades: Merchant offer execution, tier unlocking, demand pricing.
// - Golems: Iron Golem village summon checks & fling attack; Snow Golem snow trails & snowball loop.
// - WanderingTrader: 10-12 day spawner, 2 trader llamas, invisibility and despawn timer.

pub mod golems;
pub mod profession;
pub mod schedule;
pub mod trades;
pub mod villager;
pub mod wandering_trader;

pub use golems::*;
pub use profession::*;
pub use schedule::*;
pub use trades::*;
pub use villager::*;
pub use wandering_trader::*;
