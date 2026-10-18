//! Overworld world generator with 1:1 seed-for-seed parity to vanilla Minecraft 1.20.4.

pub use mc_worldgen::{
    mix_stafford13, upgrade_seed_to_128bit, BlendedNoise, CaveNoiseRouter, ChunkPos,
    ClimateSampler, ConcentricRingsPlacement, ImprovedNoise, LegacyRandom, NormalNoise,
    OverworldBiomeRegistry, OverworldGenerator, ParameterPoint, ParameterRange, PerlinNoise,
    PositionalRandomFactory, RandomSpreadPlacement, SpreadType, StructureLocator, StructureRoot,
    StructureType, TargetPoint, Xoroshiro128PlusPlus,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iron_oxide_seed_for_seed_parity() {
        let generator = OverworldGenerator::new(123456789);

        // Biomes at quart coordinates
        let biome0 = generator.sample_biome(0, 16, 0);
        assert_eq!(biome0, "minecraft:river");

        let biome1 = generator.sample_biome(40, 16, -20);
        assert_eq!(biome1, "minecraft:forest");

        // 16x16 Chunk surface heightmaps (OCEAN_FLOOR_WG)
        let hm0 = generator.generate_chunk_heightmap(0, 0);
        assert_eq!(hm0[0], 55);
        assert_eq!(hm0[255], 54);

        let hm1 = generator.generate_chunk_heightmap(10, -5);
        assert_eq!(hm1[0], 66);
        assert_eq!(hm1[255], 66);

        // Structure generation hooks
        let sh_structures = generator.get_structures_in_chunk(-64, -107);
        assert!(sh_structures.iter().any(|s| s.structure_type == StructureType::Stronghold));

        let v_structures = generator.get_structures_in_chunk(21, 23);
        assert!(v_structures.iter().any(|s| s.structure_type == StructureType::Village));
    }
}

