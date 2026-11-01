use sim::game::Game;
use std::collections::BTreeMap;
fn main() {
    let mut g = Game::new(99);
    for v in &g.world.villages { println!("{} bld {} homes {}", v.name, v.buildings.len(), v.buildings.iter().filter(|b| b.kind == sim::world::villages::BuildingKind::Home).count()); }
    g.speed = 4.0;
    for hour in 0..24 {
        for _ in 0..(60.0 * 0.6 / 4.0 / 0.1) as usize { g.tick(0.1); }
        let mut m: BTreeMap<String, usize> = BTreeMap::new();
        for v in g.villagers.iter().filter(|v| v.alive) {
            *m.entry(format!("{:?}/{:?}", v.goal, v.phase).chars().take(22).collect()).or_default() += 1;
        }
        println!("h{:>5.1} {:?}", g.clock.hour(), m);
        let _ = hour;
    }
}
