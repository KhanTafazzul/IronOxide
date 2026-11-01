use mc_worldgen::OverworldGenerator;

fn main() {
    let generator = OverworldGenerator::new(123456789);
    let section = generator.generate_section_density(0, 3, 0);

    for ly in 0..16 {
        let y = 48 + ly;
        let d = section.get(0, ly, 0);
        let solid = section.is_solid(0, ly, 0);
        println!("Y={}: density={:.8}, solid={}", y, d, solid);
    }
}
