//! Unit tests for dynamic Jigsaw Template Pool JSON deserialization,
//! element weighting, cumulative sampling, and piece expansion integration.
//!
//! REA References:
//! - `net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool` (eap.class)
//! - `net.minecraft.world.level.levelgen.structure.pools.SinglePoolElement` (eam.class)
//! - `net.minecraft.world.level.levelgen.structure.pools.JigsawPlacement` (eaj.class)

use std::path::Path;
use mc_worldgen::seed_pipeline::random::Xoroshiro128PlusPlus;
use mc_worldgen::structures::{
    JigsawStructureAssembler, PoolElement, PoolProjection, StructureTemplatePool,
    TemplatePoolEntry, TemplatePoolRegistry, WeightedAliasTable,
};

#[test]
fn test_template_pool_json_deserialization_roundtrip() {
    let json_data = r#"{
        "name": "minecraft:village/plains/houses",
        "fallback": "minecraft:village/plains/terminators",
        "elements": [
            {
                "weight": 2,
                "element": {
                    "element_type": "minecraft:legacy_single_pool_element",
                    "location": "minecraft:village/plains/houses/plains_small_house_1",
                    "processors": "minecraft:mossify_10_percent",
                    "projection": "rigid"
                }
            },
            {
                "weight": 3,
                "element": {
                    "element_type": "minecraft:legacy_single_pool_element",
                    "location": "minecraft:village/plains/houses/plains_small_house_2",
                    "processors": { "processors": [] },
                    "projection": "terrain_matching"
                }
            }
        ]
    }"#;

    let mut pool: StructureTemplatePool = serde_json::from_str(json_data).expect("Failed to deserialize pool");
    pool.compile();

    assert_eq!(pool.name, "minecraft:village/plains/houses");
    assert_eq!(pool.fallback, "minecraft:village/plains/terminators");
    assert_eq!(pool.elements.len(), 2);
    assert_eq!(pool.total_weight(), 5);

    let elem0 = &pool.elements[0];
    assert_eq!(elem0.weight, 2);
    assert_eq!(elem0.element.location.as_deref(), Some("minecraft:village/plains/houses/plains_small_house_1"));
    assert_eq!(elem0.element.projection, Some(PoolProjection::Rigid));

    let elem1 = &pool.elements[1];
    assert_eq!(elem1.weight, 3);
    assert_eq!(elem1.element.location.as_deref(), Some("minecraft:village/plains/houses/plains_small_house_2"));
    assert_eq!(elem1.element.projection, Some(PoolProjection::TerrainMatching));
}

#[test]
fn test_weighted_element_sampling_distribution() {
    let mut pool = StructureTemplatePool::new(
        "test:weighted_pool",
        "minecraft:empty",
        vec![
            TemplatePoolEntry::new(10, PoolElement::single("piece_a", PoolProjection::Rigid)),
            TemplatePoolEntry::new(20, PoolElement::single("piece_b", PoolProjection::Rigid)),
            TemplatePoolEntry::new(70, PoolElement::single("piece_c", PoolProjection::Rigid)),
        ],
    );
    pool.compile();
    assert_eq!(pool.total_weight(), 100);

    let mut rng = Xoroshiro128PlusPlus::new(42, 1337);
    let mut counts = [0usize; 3];
    let total_samples = 20_000;

    for _ in 0..total_samples {
        let sample = pool.sample_element_xoroshiro(&mut rng).expect("Must return element");
        match sample.location.as_deref() {
            Some("piece_a") => counts[0] += 1,
            Some("piece_b") => counts[1] += 1,
            Some("piece_c") => counts[2] += 1,
            _ => panic!("Unknown piece sampled"),
        }
    }

    let p0 = counts[0] as f64 / total_samples as f64;
    let p1 = counts[1] as f64 / total_samples as f64;
    let p2 = counts[2] as f64 / total_samples as f64;

    // Check convergence to weights [0.10, 0.20, 0.70] with tolerance ±0.02
    assert!((p0 - 0.10).abs() < 0.02, "p0 expected ~0.10, got {}", p0);
    assert!((p1 - 0.20).abs() < 0.02, "p1 expected ~0.20, got {}", p1);
    assert!((p2 - 0.70).abs() < 0.02, "p2 expected ~0.70, got {}", p2);
}

#[test]
fn test_weighted_alias_table_sampling() {
    let weights = [15, 35, 50];
    let alias_table = WeightedAliasTable::new(&weights);
    assert_eq!(alias_table.total_weight(), 100);

    let mut rng = Xoroshiro128PlusPlus::new(999, 888);
    let mut counts = [0usize; 3];
    let total_samples = 20_000;

    for _ in 0..total_samples {
        let idx = alias_table.sample_xoroshiro(&mut rng);
        counts[idx] += 1;
    }

    let p0 = counts[0] as f64 / total_samples as f64;
    let p1 = counts[1] as f64 / total_samples as f64;
    let p2 = counts[2] as f64 / total_samples as f64;

    assert!((p0 - 0.15).abs() < 0.02, "p0 expected ~0.15, got {}", p0);
    assert!((p1 - 0.35).abs() < 0.02, "p1 expected ~0.35, got {}", p1);
    assert!((p2 - 0.50).abs() < 0.02, "p2 expected ~0.50, got {}", p2);
}

#[test]
fn test_client_jar_template_pool_loading() {
    let jar_path = Path::new("client.jar");
    let fallback_path = Path::new("../client.jar");
    let target_path = if jar_path.exists() {
        jar_path
    } else if fallback_path.exists() {
        fallback_path
    } else {
        Path::new("c:/Users/khant/OneDrive/Desktop/Falix/Minecraft reverce in rust/client.jar")
    };

    if target_path.exists() {
        let registry = TemplatePoolRegistry::load_from_jar(target_path)
            .expect("Must load template pools from client.jar");

        // Verify base 141 pools + 41 trial chambers pools + synthesized woodland mansion pools
        assert!(registry.len() >= 140, "Expected at least 140 pools, found {}", registry.len());

        // Check Plains Village pools
        assert!(registry.contains_key("village/plains/houses"));
        assert!(registry.contains_key("village/plains/streets"));
        assert!(registry.contains_key("village/plains/town_centers"));
        let plains_houses = registry.get_pool("village/plains/houses").unwrap();
        assert!(!plains_houses.elements.is_empty());
        assert!(plains_houses.total_weight() > 0);

        // Check Savanna Village pools
        assert!(registry.contains_key("village/savanna/houses"));
        assert!(registry.contains_key("village/savanna/streets"));
        assert!(registry.contains_key("village/savanna/town_centers"));
        let savanna_houses = registry.get_pool("village/savanna/houses").unwrap();
        assert!(!savanna_houses.elements.is_empty());

        // Check Desert Village pools
        assert!(registry.contains_key("village/desert/houses"));
        assert!(registry.contains_key("village/desert/streets"));
        assert!(registry.contains_key("village/desert/town_centers"));
        let desert_houses = registry.get_pool("village/desert/houses").unwrap();
        assert!(!desert_houses.elements.is_empty());

        // Check Trial Chambers pools
        assert!(registry.contains_key("trial_chambers/corridor"));
        assert!(registry.contains_key("trial_chambers/hallway"));
        assert!(registry.contains_key("trial_chambers/chamber/addon"));
        assert!(registry.contains_key("trial_chambers/spawner/all"));

        // Check Woodland Mansion pools
        assert!(registry.contains_key("woodland_mansion/entrance"));
        assert!(registry.contains_key("woodland_mansion/corridors"));
        assert!(registry.contains_key("woodland_mansion/rooms_1x1"));
        assert!(registry.contains_key("woodland_mansion/rooms_1x2"));
        assert!(registry.contains_key("woodland_mansion/rooms_2x2"));
    }
}

#[test]
fn test_jigsaw_assembler_structure_expansions() {
    let registry = TemplatePoolRegistry::load_default();
    assert!(!registry.is_empty(), "Registry must contain default pools");

    let assembler = JigsawStructureAssembler::new(&registry, 5);
    let mut rng = Xoroshiro128PlusPlus::new(12345, 67890);

    // 1. Plains Village expansion
    let plains_pieces = assembler.assemble_plains_village(&mut rng);
    assert!(!plains_pieces.is_empty(), "Plains village must resolve pieces");
    assert_eq!(plains_pieces[0].pool_name, "minecraft:village/plains/town_centers");
    assert!(plains_pieces.iter().any(|p| p.pool_name.contains("streets")));
    assert!(plains_pieces.iter().any(|p| p.pool_name.contains("houses")));

    // 2. Savanna Village expansion
    let savanna_pieces = assembler.assemble_savanna_village(&mut rng);
    assert!(!savanna_pieces.is_empty(), "Savanna village must resolve pieces");
    assert_eq!(savanna_pieces[0].pool_name, "minecraft:village/savanna/town_centers");
    assert!(savanna_pieces.iter().any(|p| p.pool_name.contains("streets")));
    assert!(savanna_pieces.iter().any(|p| p.pool_name.contains("houses")));

    // 3. Desert Village expansion
    let desert_pieces = assembler.assemble_desert_village(&mut rng);
    assert!(!desert_pieces.is_empty(), "Desert village must resolve pieces");
    assert_eq!(desert_pieces[0].pool_name, "minecraft:village/desert/town_centers");
    assert!(desert_pieces.iter().any(|p| p.pool_name.contains("streets")));
    assert!(desert_pieces.iter().any(|p| p.pool_name.contains("houses")));

    // 4. Woodland Mansion expansion
    let mansion_pieces = assembler.assemble_woodland_mansion(&mut rng);
    assert!(!mansion_pieces.is_empty(), "Woodland mansion must resolve pieces");
    assert_eq!(mansion_pieces[0].pool_name, "minecraft:woodland_mansion/entrance");
    assert!(mansion_pieces.iter().any(|p| p.pool_name.contains("corridors") || p.pool_name.contains("rooms")));

    // 5. Trial Chambers expansion
    let trial_pieces = assembler.assemble_trial_chambers(&mut rng);
    assert!(!trial_pieces.is_empty(), "Trial chambers must resolve pieces");
    assert_eq!(trial_pieces[0].pool_name, "minecraft:trial_chambers/corridor");
    assert!(trial_pieces.iter().any(|p| p.pool_name.contains("hallway") || p.pool_name.contains("spawner")));
}

#[test]
fn test_shuffled_elements_fisher_yates_parity() {
    let mut pool = StructureTemplatePool::new(
        "test:shuffle_pool",
        "minecraft:empty",
        vec![
            TemplatePoolEntry::new(1, PoolElement::single("piece_1", PoolProjection::Rigid)),
            TemplatePoolEntry::new(2, PoolElement::single("piece_2", PoolProjection::Rigid)),
            TemplatePoolEntry::new(3, PoolElement::single("piece_3", PoolProjection::Rigid)),
        ],
    );
    pool.compile();

    let mut rng = Xoroshiro128PlusPlus::new(101, 202);
    let shuffled = pool.get_shuffled_elements(|b| rng.next_int_bounded(b));

    // Total elements must equal sum of weights: 1 + 2 + 3 = 6
    assert_eq!(shuffled.len(), 6);

    let count_p1 = shuffled.iter().filter(|e| e.location.as_deref() == Some("piece_1")).count();
    let count_p2 = shuffled.iter().filter(|e| e.location.as_deref() == Some("piece_2")).count();
    let count_p3 = shuffled.iter().filter(|e| e.location.as_deref() == Some("piece_3")).count();

    assert_eq!(count_p1, 1);
    assert_eq!(count_p2, 2);
    assert_eq!(count_p3, 3);
}
