use mc_worldgen::OverworldGenerator;

fn main() {
    let generator = OverworldGenerator::new(123456789);
    for y in 50..=56 {
        let sec_y = y / 16;
        let local_y = (y % 16) as usize;
        let sec = generator.generate_section_density(0, sec_y, 0);
        let d = sec.get(0, local_y, 0);
        let solid = sec.is_solid(0, local_y, 0);
        println!("Y={} (sec={}, ly={}) -> density={}, solid={}", y, sec_y, local_y, d, solid);
    }
}
