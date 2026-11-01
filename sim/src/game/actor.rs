//! Anything that walks: the player, villagers and creatures share movement,
//! path following and animation state.

use serde::{Deserialize, Serialize};

use crate::art::people::{FRAMES_PER_DIR, IDLE_FRAMES, WALK_FIRST, WALK_FRAMES, WORK_FIRST};
use crate::world::path::{simplify, PathFinder};
use crate::world::{World, TILE};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Facing {
    Down,
    Up,
    Left,
    Right,
}

/// What the actor is visibly doing; drives the animation and particle fx.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Activity {
    None = 0,
    Chop = 1,
    Mine = 2,
    Hoe = 3,
    Fish = 4,
    Smith = 5,
    Pray = 6,
    Talk = 7,
    Sweep = 8,
    Play = 9,
    Fight = 10,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    pub x: f32,
    pub y: f32,
    pub facing: Facing,
    pub moving: bool,
    pub anim: f32,
    pub speed: f32,
    pub activity: Activity,
    /// Waypoints in world pixels.
    pub path: Vec<(f32, f32)>,
    pub path_i: usize,
    pub sheet: u16,
    pub hidden: bool,
    pub blink: f32,
    pub flash: f32,
}

impl Actor {
    pub fn new(x: f32, y: f32, sheet: u16, speed_tiles: f32) -> Actor {
        Actor {
            x,
            y,
            facing: Facing::Down,
            moving: false,
            anim: 0.0,
            speed: speed_tiles * TILE as f32,
            activity: Activity::None,
            path: Vec::new(),
            path_i: 0,
            sheet,
            hidden: false,
            blink: 3.0,
            flash: 0.0,
        }
    }

    #[inline]
    pub fn tile(&self) -> (i32, i32) {
        ((self.x / TILE as f32).floor() as i32, (self.y / TILE as f32).floor() as i32)
    }

    pub fn face_toward(&mut self, tx: f32, ty: f32) {
        let dx = tx - self.x;
        let dy = ty - self.y;
        self.facing = if dx.abs() > dy.abs() {
            if dx > 0.0 { Facing::Right } else { Facing::Left }
        } else if dy > 0.0 {
            Facing::Down
        } else {
            Facing::Up
        };
    }

    /// Plan a path to a tile. Returns false if unreachable.
    pub fn path_to(&mut self, world: &World, pf: &mut PathFinder, goal: (i32, i32), max_nodes: usize) -> bool {
        let start = self.tile();
        let start = if world.walkable(start.0, start.1) { start } else { nearest_walkable(world, start).unwrap_or(start) };
        match pf.walk(world, start, goal, max_nodes) {
            Some(p) => {
                let s = simplify(&p);
                self.path = s
                    .iter()
                    .skip(1)
                    .map(|&(x, y)| ((x * TILE + TILE / 2) as f32, (y * TILE + TILE / 2 + 6) as f32))
                    .collect();
                self.path_i = 0;
                true
            }
            None => false,
        }
    }

    pub fn has_path(&self) -> bool {
        self.path_i < self.path.len()
    }

    pub fn clear_path(&mut self) {
        self.path.clear();
        self.path_i = 0;
    }

    /// Follow the current path. Returns true when the destination is reached.
    pub fn follow(&mut self, dt: f32) -> bool {
        if !self.has_path() {
            self.moving = false;
            return true;
        }
        let (tx, ty) = self.path[self.path_i];
        let dx = tx - self.x;
        let dy = ty - self.y;
        let d = (dx * dx + dy * dy).sqrt();
        let step = self.speed * dt;
        if d <= step.max(0.5) {
            self.x = tx;
            self.y = ty;
            self.path_i += 1;
            if !self.has_path() {
                self.moving = false;
                return true;
            }
        } else {
            self.x += dx / d * step;
            self.y += dy / d * step;
            self.facing = if dx.abs() > dy.abs() * 1.1 {
                if dx > 0.0 { Facing::Right } else { Facing::Left }
            } else if dy > 0.0 {
                Facing::Down
            } else {
                Facing::Up
            };
        }
        self.moving = true;
        false
    }

    /// Free movement with tile collision (player keyboard / joystick).
    pub fn walk_free(&mut self, world: &World, vx: f32, vy: f32, dt: f32) {
        let len = (vx * vx + vy * vy).sqrt();
        if len < 0.01 {
            self.moving = false;
            return;
        }
        let (nx, ny) = (vx / len, vy / len);
        let step = self.speed * dt;
        let try_move = |a: &Actor, x: f32, y: f32| -> bool { can_stand(world, a.x, a.y, x, y) };
        let tx = self.x + nx * step;
        if try_move(self, tx, self.y) {
            self.x = tx;
        }
        let ty = self.y + ny * step;
        if try_move(self, self.x, ty) {
            self.y = ty;
        }
        self.facing = if nx.abs() > ny.abs() {
            if nx > 0.0 { Facing::Right } else { Facing::Left }
        } else if ny > 0.0 {
            Facing::Down
        } else {
            Facing::Up
        };
        self.moving = true;
    }

    pub fn update_anim(&mut self, dt: f32) {
        self.anim += dt;
        self.blink -= dt;
        if self.blink < -0.12 {
            self.blink = 2.0 + (self.x * 13.7 + self.y).sin().abs() * 3.0;
        }
        self.flash = (self.flash - dt).max(0.0);
    }

    /// Current sprite id and horizontal flip.
    pub fn sprite(&self) -> (u16, bool) {
        let (dir, flip) = match self.facing {
            Facing::Down => (0u16, false),
            Facing::Up => (1, false),
            Facing::Right => (2, false),
            Facing::Left => (2, true),
        };
        let f = if self.moving {
            WALK_FIRST + ((self.anim * 10.0) as u16 % WALK_FRAMES)
        } else if self.activity != Activity::None && self.activity != Activity::Talk && self.activity != Activity::Pray {
            WORK_FIRST + ((self.anim * 3.0) as u16 % 2)
        } else if dir == 0 && self.blink < 0.0 {
            return (self.sheet + FRAMES_PER_DIR * 3, flip);
        } else {
            (self.anim * 2.5) as u16 % IDLE_FRAMES
        };
        (self.sheet + dir * FRAMES_PER_DIR + f, flip)
    }
}

/// Collision test for moving a body from (fx, fy) to (x, y). The body is a
/// small box around the feet.
pub fn can_stand(world: &World, fx: f32, fy: f32, x: f32, y: f32) -> bool {
    let t = TILE as f32;
    let from = ((fx / t).floor() as i32, (fy / t).floor() as i32);
    let (rx, ry) = (7.0, 4.0);
    for (ox, oy) in [(-rx, -ry), (rx, -ry), (-rx, ry), (rx, ry), (0.0, 0.0)] {
        let tile = (((x + ox) / t).floor() as i32, ((y + oy) / t).floor() as i32);
        if tile == from {
            continue;
        }
        if !world.walkable(tile.0, tile.1) {
            return false;
        }
        // Level transitions need a straight step through stairs.
        if world.level(tile.0, tile.1) != world.level(from.0, from.1) {
            let dx = tile.0 - from.0;
            let dy = tile.1 - from.1;
            if dx.abs() + dy.abs() != 1 || !world.can_step(from.0, from.1, tile.0, tile.1) {
                return false;
            }
        } else if (world.is_face(tile.0, tile.1) || world.is_face(from.0, from.1)) && tile.1 == from.1 {
            return false;
        }
    }
    true
}

pub fn nearest_walkable(world: &World, (x, y): (i32, i32)) -> Option<(i32, i32)> {
    for r in 1..8i32 {
        for dy in -r..=r {
            for dx in -r..=r {
                let (dx, dy): (i32, i32) = (dx, dy);
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }
                if world.walkable(x + dx, y + dy) {
                    return Some((x + dx, y + dy));
                }
            }
        }
    }
    None
}
