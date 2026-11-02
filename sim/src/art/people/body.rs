//! Poses and the body: one parametric skeleton per frame. `build` turns a
//! `Look` and a pose into joints in pixels (head, neck, chest, waist, hips,
//! crotch) and `limbs` hangs the arms and legs off it. Every other module
//! reads its heights from here; nothing else owns a y offset of the body.
//!
//! Proportions, from pixel-dissecting the 64x128 reference idle/walk sheets
//! (study only) and three drawing guides:
//!
//! | source                     | heads | head | torso | legs | legs/h | shoulders |
//! |----------------------------|-------|------|-------|------|--------|-----------|
//! | ref sheet front (89px)     | 2.9   | 30px | 33px  | 23px | 26%    | 33px      |
//! | ref sheet side             | 2.9   | 30px | 33px  | 23px | 26%    | depth 13  |
//! | chibi 3-head guide (Clip)  | 3.0   | 1.0  | 1.0   | 1.0  | 33%    | ~1 head   |
//! | pixel sprite guide (pixnote)| 4-6  | -    | -     | -    | ~45%   | > head    |
//! | classical adult (lifedraw) | 7.5   | 1.0  | 3.0   | 3.5  | 47%    | 2 head-w  |
//! | ours, average              | 3.0   | 31px | 33px  | 31px | 32%    | 24px      |
//! | ours, petite / tall        | 2.8/3.2 | 31 | 31/35 | 26/35 | 29%/35% | 23/25px |
//!
//! Chibi guides agree: the head is fixed, height lives in the legs (longer
//! legs read older and taller, longer torso cuter), the knee sits halfway
//! down the leg, women's hips match their shoulders, men's hips are ~0.55
//! of theirs. Our head (31px with hair) matches the reference; the torso
//! and legs are read off it per height step so petite/average/tall differ
//! by a clear 5-6px per step at 1x. All bodies are adults.
//!
//! The joint set is the one proven rigs use (SpriteKiln's humanoid bones,
//! pixelfx's pixel-unit rig): head, neck, chest, waist, pelvis/hips,
//! shoulders, elbows, hands, knees, feet, with segment lengths in pixels and
//! feet placed from the ground up (two-bone IK from a foot target in the
//! front and back views; authored gait angles in profile).

use super::*;

#[derive(Clone, Copy)]
pub(super) struct Pose {
    /// Walk cycle angle (radians, one full stride pair = 2π).
    pub(super) walk: Option<f32>,
    /// Inhale amount 0..1 for idle breathing.
    pub(super) breathe: f32,
    pub(super) work: u8,
    pub(super) blink: bool,
    /// 0 idle, 1 walk, 2 work, and the sheet frame index.
    pub(super) cycle: u8,
    pub(super) frame: u16,
}

pub(super) fn pose_for(f: u16) -> Pose {
    use std::f32::consts::TAU;
    let base = Pose { walk: None, breathe: 0.0, work: 0, blink: false, cycle: 0, frame: f };
    match f {
        // In for two frames, out for two: a held breath, not a twitch.
        0..=3 => Pose { breathe: if f >= 2 { 1.0 } else { 0.0 }, ..base },
        4..=9 => {
            let ph = (f - WALK_FIRST) as f32 / WALK_FRAMES as f32;
            Pose { walk: Some(TAU * ph + 0.5), cycle: 1, ..base }
        }
        10 => Pose { work: 1, cycle: 2, ..base },
        11 => Pose { work: 2, cycle: 2, ..base },
        _ => Pose { blink: true, ..base },
    }
}

/// Bust secondary motion. Researched model (2D rig physics, breast
/// biomechanics, the hand-animated reference): in a looping walk the bust is
/// a forced follower that trails the torso by one frame — its height equals
/// the torso's height a frame earlier — so with our bob (-1,-2,-1 per step)
/// the offset relative to the chest is [0, +1, -1] per step. Real walking
/// moves ~1.3px at this scale, so medium only drags, large follows the lag
/// rule, huge adds overshoot. Shape steps are whole pixels and follow the
/// lag (lagging down = stretch, pulled up = squash); the silhouette never
/// shimmers. Idle follows the breath (the chest lifts on frame 2).
/// Entries are (offset px, + = down; shape -1 squash / +1 stretch).
pub(super) type Jig = (i32, i32);

pub(super) const JIGGLE_WALK: [[Jig; 6]; 5] = [
    [(0, 0); 6],
    [(0, 0); 6],
    [(0, 0), (1, 0), (0, 0), (0, 0), (1, 0), (0, 0)],
    [(0, 0), (1, 1), (-1, -1), (0, 0), (1, 1), (-1, -1)],
    [(-1, -1), (1, 1), (-1, -1), (-1, -1), (1, 1), (-1, -1)],
];

pub(super) const JIGGLE_IDLE: [[Jig; 4]; 5] = [
    [(0, 0); 4],
    [(0, 0); 4],
    [(0, 0), (0, 0), (1, 0), (0, 0)],
    [(-1, -1), (0, 0), (1, 1), (0, 0)],
    [(-1, -1), (0, 0), (1, 1), (0, 0)],
];

pub(super) fn jiggle(figure: u8, pose: &Pose, frame: u16) -> Jig {
    jiggle_at(figure, pose, frame, 0)
}

/// The table entry `back` frames earlier in the same cycle.
pub(super) fn jiggle_at(figure: u8, pose: &Pose, frame: u16, back: u16) -> Jig {
    let f = figure.min(4) as usize;
    match pose.cycle {
        1 => JIGGLE_WALK[f][((frame - WALK_FIRST + WALK_FRAMES - back) % WALK_FRAMES) as usize],
        0 => JIGGLE_IDLE[f][((frame + IDLE_FRAMES - back) % IDLE_FRAMES) as usize],
        _ => (0, 0),
    }
}

/// The skeleton: joints in pixels for one frame, derived from race, sex,
/// height, build and figure plus the pose.
pub(super) struct Build {
    pub(super) cx: f32,
    pub(super) ground: f32,
    /// Skull radii used by hats and ears.
    pub(super) head_r: (f32, f32),
    pub(super) head_c: V2,
    /// Top of the skull; ears and horns root a few rows below it.
    pub(super) crown_y: f32,
    /// Face landmarks (rows): eyes, mouth. Nose and blush sit above the mouth.
    pub(super) eye_y: i32,
    pub(super) mouth_y: i32,
    /// Neck: under the chin down to the shoulder line, and its half-width.
    pub(super) neck_top: f32,
    pub(super) neck: V2,
    pub(super) neck_w: f32,
    /// Chest joint (bust centre at rest) and the bust row this frame.
    pub(super) chest: V2,
    pub(super) bust_y: f32,
    pub(super) shoulder_y: f32,
    pub(super) shoulder_w: f32,
    pub(super) chest_w: f32,
    pub(super) waist_y: f32,
    pub(super) waist_w: f32,
    pub(super) hip_y: f32,
    pub(super) hip_w: f32,
    pub(super) crotch_y: f32,
    pub(super) hip_joint_y: f32,
    pub(super) thigh: f32,
    pub(super) calf: f32,
    pub(super) thigh_r: f32,
    pub(super) calf_r: f32,
    pub(super) upper_arm: f32,
    pub(super) forearm: f32,
    pub(super) arm_r: f32,
    pub(super) bust: f32,
    /// Adult woman: curvy hips, seat and fitted clothes (independent of bust).
    pub(super) woman: bool,
    /// Lag applied to soft parts (hair, skirts) this frame.
    pub(super) lag: f32,
    /// Bust shape step this frame (-1 squash, +1 stretch); its offset is in
    /// `bust_y`.
    pub(super) jsq: f32,
    /// Highlight lag relative to the bust (whole px).
    pub(super) jhl: f32,
    /// Sideways swing of hair, skirts and capes: side view only. Seen from
    /// the front or back, soft parts bounce vertically instead.
    pub(super) sway: f32,
    /// Authored walk bob before the feet are planted.
    pub(super) bob: f32,
}

pub(super) fn build(look: &Look, pose: Pose, plant: f32) -> Build {
    let dwarf = look.race == RACE_DWARF;
    let elf = look.race == RACE_ELF;
    let f = look.female;
    let (leg_k, torso_k, width_k) = if dwarf {
        (0.66, 0.92, 1.22)
    } else if elf {
        (1.08, 1.02, 0.94)
    } else {
        (1.0, 1.0, 1.0)
    };
    // Height: legs carry most of it, the torso some, the frame a touch
    // (see the table above). The head never changes.
    let (hl, ht, hw) = [(0.86, 0.94, 0.96), (1.0, 1.0, 1.0), (1.1, 1.05, 1.03)][look.height.min(2) as usize];
    let (leg_k, torso_k, width_k) = (leg_k * hl, torso_k * ht, width_k * hw);
    // Long legs: about a third of the standing height.
    let thigh = 14.0 * leg_k;
    let calf = 13.5 * leg_k;
    let ground = 116.0;
    // Work poses keep the feet planted; only soft parts lag.
    let (bob, lag) = match pose.walk {
        Some(ph) => (-(ph.sin().abs()) * 1.5, -ph.cos() * 1.4),
        None if pose.work == 1 => (0.0, -0.9),
        None if pose.work == 2 => (0.0, 1.1),
        None => (0.0, 0.0),
    };
    // Whole-pixel motion: fractional bobbing makes every frame re-rasterise
    // differently (shimmering outlines), so the body moves in pixel steps.
    let hip_joint_y = ground - 3.0 - calf - thigh + (bob + plant).round();
    let crotch_y = hip_joint_y + 2.0;
    // Breathing lifts the chest and head; hips and feet stay put.
    let torso0 = if f { 30.0 } else { 31.0 } * torso_k;
    let torso = torso0 + pose.breathe.floor();
    let shoulder_y = crotch_y - torso;
    // LPC base body: the head is 24 wide x 23 tall under the 3/4 top-down
    // camera and its chin overlaps the shoulder line by two rows, so no
    // neck shows between the chin and the collar.
    let head_r = (12.0, 11.0);
    let head_c = v(32.0, shoulder_y - 9.0);
    let chin_y = head_c.y + head_r.1;
    let (jig, jsq) = jiggle(look.figure, &pose, pose.frame);
    // The highlight trails the bust by another frame: softness without
    // moving the outline.
    let jhl = jiggle_at(look.figure, &pose, pose.frame, 1).0 - jig;
    let fig = f && look.mature;
    let bd = look.build.min(3) as usize;
    // Hourglass for women (waist about 0.6 of the hips), V-taper for men;
    // each build scales shoulders, chest, waist, hips, thighs and arms.
    let (sw, chest, waist, hip, thr, arm) = if f {
        [(8.6, 8.8, 6.0, 10.2, 5.0, 2.5), (9.0, 9.2, 6.6, 11.2, 5.6, 2.7), (9.2, 9.4, 6.8, 12.4, 6.2, 2.8), (9.8, 10.2, 8.2, 13.2, 6.8, 3.1)][bd]
    } else {
        [(11.0, 10.4, 8.4, 9.2, 4.7, 3.0), (12.0, 11.5, 9.0, 9.8, 5.0, 3.4), (13.4, 13.0, 9.6, 10.2, 5.8, 4.1), (12.6, 13.0, 12.2, 11.4, 6.0, 3.7)][bd]
    };
    let bust: f32 = if fig {
        [0.0f32, 4.2, 5.4, 6.4, 7.2][look.figure.min(4) as usize]
    } else {
        0.0
    };
    let jig = if bust > 0.0 { jig as f32 } else { 0.0 };
    // The chest joint sits a third of the way down the torso; fuller busts
    // hang a little lower. Rounded so the bust never re-rasterises between
    // frames: the jiggle tables move it in whole pixels.
    let chest_j = v(32.0, (shoulder_y + torso0 * 0.317 + (bust - 5.4).max(0.0) * 0.6).round());

    Build {
        cx: 32.0,
        ground,
        head_r,
        head_c,
        crown_y: head_c.y - head_r.1,
        eye_y: (chin_y - 11.0).round() as i32,
        mouth_y: (chin_y - 1.0).round() as i32,
        neck_top: chin_y - 2.0,
        neck: v(32.0, shoulder_y),
        neck_w: if f { 3.0 } else { 3.6 },
        chest: chest_j,
        bust_y: chest_j.y + jig,
        shoulder_y,
        shoulder_w: sw * width_k,
        chest_w: chest * width_k,
        waist_y: shoulder_y + torso * 0.62,
        waist_w: waist * width_k,
        hip_y: shoulder_y + torso * 0.87,
        hip_w: if fig || !f { hip } else { hip - 1.5 } * width_k,
        crotch_y,
        hip_joint_y,
        thigh,
        calf,
        thigh_r: if f && !fig { thr - 0.6 } else { thr } * width_k.max(0.9),
        calf_r: if f { 3.7 } else { 4.0 } * width_k.max(0.9),
        // Dwarves are broad, but their arms stay in proportion to their
        // short legs: hands at mid-thigh, not the knee.
        upper_arm: 12.0 * torso_k * if dwarf { 0.86 } else { 1.0 },
        forearm: 11.0 * torso_k * if dwarf { 0.86 } else { 1.0 },
        arm_r: arm * width_k.clamp(0.9, 1.06),
        bust,
        woman: fig,
        lag,
        jsq: if bust > 0.0 { jsq as f32 } else { 0.0 },
        jhl: if bust > 0.0 { jhl as f32 } else { 0.0 },
        sway: 0.0,
        bob,
    }
}

pub(super) struct Limbs {
    pub(super) hip_l: V2,
    pub(super) knee_l: V2,
    pub(super) ankle_l: V2,
    pub(super) hip_r: V2,
    pub(super) knee_r: V2,
    pub(super) ankle_r: V2,
    pub(super) sh_l: V2,
    pub(super) el_l: V2,
    pub(super) wr_l: V2,
    pub(super) sh_r: V2,
    pub(super) el_r: V2,
    pub(super) wr_r: V2,
}

pub(super) fn limbs(b: &Build, pose: Pose, dir: u8) -> Limbs {
    let side = dir == DIR_SIDE;
    let walking = pose.walk.is_some();
    let s = pose.walk.unwrap_or(0.0).sin();
    let hip_dx = if side { 0.0 } else { b.thigh_r };
    // 3/4 top-down: what's farther from the camera sits higher on screen,
    // so in profile the far leg stands 2px up (its foot above the near one)
    // instead of both feet on one side-scroller baseline.
    let far_up = if side { 2.0 } else { 0.0 };
    // Walking, the upper body leads: in profile it leans a pixel into the
    // stride; from the front or back it sways over the standing leg. Both
    // are done by setting the hips under it.
    let hip_shift = match pose.walk {
        Some(_) if side => -1.0,
        Some(ph) => -(ph.sin() * 1.2).round(),
        None => 0.0,
    };
    let hip_l = v(b.cx - hip_dx + hip_shift, b.hip_joint_y - far_up);
    let hip_r = v(b.cx + hip_dx + hip_shift, b.hip_joint_y);
    let walk_ang = pose.walk.unwrap_or(0.0);
    let leg = |hip: V2, sgn: f32, swing: f32, lift: f32| -> (V2, V2) {
        if side {
            if !walking {
                return (hip.add(v(0.0, b.thigh)), hip.add(v(0.0, b.thigh + b.calf)));
            }
            // Gait phase per leg (0 = heel strike). Stance (60%): the planted
            // leg sweeps back. Swing (40%): the knee bends most mid-swing as
            // the leg comes forward; it never kicks up while trailing.
            use std::f32::consts::{PI, TAU};
            let p = ((walk_ang + if sgn < 0.0 { 0.0 } else { PI }) / TAU).rem_euclid(1.0);
            let th = 0.38 * (TAU * p).cos();
            let flex = if p > 0.6 { 0.75 * (PI * (p - 0.6) / 0.4).sin() } else { 0.08 * (PI * p / 0.6).sin() };
            let knee = hip.add(V2::polar(b.thigh, th));
            let ankle = knee.add(V2::polar(b.calf, th - flex));
            let _ = (swing, lift);
            (knee, ankle)
        } else {
            // Knees drawn slightly together: the soft anime stance. Feet are
            // placed from the ground up, so a dip of the hips bends the legs
            // instead of sinking the feet.
            let ankle_y = b.ground - 3.0 - lift * 3.9;
            let reach = ankle_y - hip.y + lift * 3.9;
            let knee = v(hip.x - sgn * 1.2, hip.y + reach * b.thigh / (b.thigh + b.calf) - lift * 1.3);
            let ankle = v(knee.x - sgn * 0.3, ankle_y);
            (knee, ankle)
        }
    };
    let (sl, sr) = if walking { (s, -s) } else { (0.0, 0.0) };
    let lift_l = if walking { s.max(0.0) * 2.0 } else { 0.0 };
    let lift_r = if walking { (-s).max(0.0) * 2.0 } else { 0.0 };
    let (knee_l, ankle_l) = leg(hip_l, -1.0, sl, lift_l);
    let (knee_r, ankle_r) = leg(hip_r, 1.0, sr, lift_r);
    let sh_dx = if side { 0.0 } else { b.shoulder_w - 0.6 };
    // The arm's ball sits just under the shoulder line so the deltoid reads
    // as a one-pixel rounding, not a pad poking above the slope.
    // The torso's shoulder slope runs 4-5 rows (reference pack: half-width
    // grows 0.7 px per row), so the ball sits a little lower to stay under it.
    let sh_y = b.shoulder_y + 3.0 + b.arm_r * 0.6;
    // The torso turns a touch toward the camera in profile: the far
    // shoulder edges forward, peeking past the chest line.
    let sh_l = v(b.cx - sh_dx + if side { 1.5 } else { 0.0 }, sh_y - far_up * 0.5);
    let sh_r = v(b.cx + sh_dx, sh_y);
    let arm = |sh: V2, swing: f32, out: f32| -> (V2, V2) {
        match pose.work {
            1 => {
                // Wind-up: in profile the hands go up beside the back of the
                // head, clear of the face.
                if side {
                    let el = sh.add(v(-0.5, -b.upper_arm * 0.95));
                    let wr = el.add(v(-1.5, -b.forearm * 0.9));
                    return (el, wr);
                }
                let el = sh.add(v(out * 2.5, -b.upper_arm * 0.85));
                let wr = el.add(v(out * 1.0, -b.forearm * 0.8));
                (el, wr)
            }
            2 => {
                if side {
                    let el = sh.add(V2::polar(b.upper_arm, 1.0));
                    let wr = el.add(V2::polar(b.forearm, 1.25));
                    (el, wr)
                } else {
                    // Follow-through: arms swept down and out past the hips,
                    // the tool at the low point beside the body.
                    let el = sh.add(v(out * 3.6, b.upper_arm * 0.92));
                    let wr = el.add(v(out * 1.8, b.forearm * 0.88));
                    (el, wr)
                }
            }
            _ => {
                if side {
                    let el = sh.add(V2::polar(b.upper_arm, swing * 0.45));
                    let wr = el.add(V2::polar(b.forearm, swing * 0.45 + 0.25 * swing.abs() + 0.12));
                    (el, wr)
                } else {
                    // A relaxed A-pose: elbows and hands clear the hips. In a
                    // walk the forward arm foreshortens (hand rises and tucks
                    // in) and the trailing arm hangs straighter.
                    let el = sh.add(v(out * 4.2, b.upper_arm - 0.6 - swing.abs() * 1.2));
                    let wr = el.add(v(out * (2.6 - swing.max(0.0) * 1.2), b.forearm - swing.max(0.0) * 3.2));
                    (el, wr)
                }
            }
        }
    };
    let (el_l, wr_l) = arm(sh_l, -sl, -1.0);
    let (el_r, wr_r) = arm(sh_r, -sr, 1.0);
    Limbs { hip_l, knee_l, ankle_l, hip_r, knee_r, ankle_r, sh_l, el_l, wr_l, sh_r, el_r, wr_r }
}

// ---------------------------------------------------------------- palette

pub(super) fn leg(cv: &mut Canvas, look: &Look, b: &Build, c: &Colors, (hip, knee, ankle): (V2, V2, V2), far: bool, dir: u8) {
    let g = garb(look);
    let ramp = dye(c, g.legs);
    // The far leg sits clearly in shadow so the stride reads as alternating.
    let bias = if far { -0.32 } else { 0.0 };
    let wide = if g.wide { 1.22 } else { 1.0 };
    let (tr, cr) = (b.thigh_r * wide, b.calf_r * wide);
    let ankle_r = b.calf_r * 0.62;
    let front = dir != DIR_SIDE;
    if front {
        let left = hip.x < b.cx;
        cv.side = if left { 1 } else { 2 };
        cv.clip = Some(if left { (0, 31) } else { (32, FW - 1) });
    }
    cv.begin();
    // Thigh: soft and full most of the way, easing into the knee.
    let mid = v(hip.x + (knee.x - hip.x) * 0.55, hip.y + (knee.y - hip.y) * 0.55);
    cv.capsule(hip, mid, tr, tr * 0.94, &ramp, bias);
    cv.capsule(mid, knee, tr * 0.94, cr * 1.02, &ramp, bias);
    // Calf: swells on the outside just below the knee, then slims to a
    // narrow ankle (baggy trousers gather at the ankle).
    let out = if front { if hip.x < b.cx { -0.6 } else { 0.6 } } else { -0.6 };
    let calf_mid = v(knee.x + (ankle.x - knee.x) * 0.32 + out, knee.y + (ankle.y - knee.y) * 0.32);
    // From behind, the leg stepping away lifts its heel: the shin turns
    // into shadow and only a heel shows below it.
    let heel_up = dir == DIR_UP && ankle.y < b.ground - 4.5;
    let shin_bias = if heel_up { bias - 0.35 } else { bias };
    cv.capsule(knee, calf_mid, cr * 1.0, cr * 1.1, &ramp, shin_bias);
    cv.capsule(calf_mid, ankle, cr * 1.1, ankle_r * 0.9, &ramp, shin_bias);
    let side = dir == DIR_SIDE;
    if let Some((d, h)) = g.boots {
        // Shaft up the actual shin (knee-high riders reach over the knee),
        // so a raised, foreshortened leg keeps its boot below the knee.
        let k = (h / b.calf).min(1.25);
        let top = v(ankle.x + (knee.x - ankle.x) * k, ankle.y + (knee.y - ankle.y) * k);
        cv.begin();
        cv.capsule(top, ankle, b.calf_r * 1.04, ankle_r + 0.6, &dye(c, d), shin_bias);
    }
    cv.begin();
    let shoe = match g.boots {
        Some((d, _)) => dye(c, d),
        None if g.barefoot => c.skin,
        None => c.shoe,
    };
    if side {
        cv.ellipse(ankle.add(v(2.4, 1.6)), 4.4, 2.2, &shoe, bias);
    } else if heel_up {
        cv.ellipse(ankle.add(v(0.0, 1.0)), 2.6, 1.7, &shoe, bias - 0.3);
    } else {
        cv.ellipse(ankle.add(v(0.0, 1.8)), 3.2, 2.4, &shoe, bias);
    }
    cv.clip = None;
    cv.side = 0;
}

#[allow(clippy::too_many_arguments)]
pub(super) fn arm(cv: &mut Canvas, look: &Look, b: &Build, c: &Colors, (sh, el, wr): (V2, V2, V2), far: bool, dir: u8, pose: Pose) {
    let g = garb(look);
    let r = b.arm_r;
    let (kind, d) = g.sleeve;
    let sleeve = dye(c, d);
    let bias = if far { -0.18 } else { 0.0 };
    cv.begin();
    match kind {
        Sleeve::Bare => {
            cv.capsule(sh, el, r, r * 0.88, &c.skin, bias);
            cv.capsule(el, wr, r * 0.86, r * 0.72, &c.skin, bias);
        }
        Sleeve::Rolled => {
            cv.capsule(sh, el, r, r * 0.9, &sleeve, bias);
            cv.begin();
            cv.capsule(el, wr, r * 0.86, r * 0.72, &c.skin, bias);
            // The rolled cuff just above the elbow.
            cv.begin();
            cv.capsule(el.add(v((sh.x - el.x) * 0.12, (sh.y - el.y) * 0.12)), el, r * 1.02, r * 0.98, &sleeve, bias + 0.06);
        }
        Sleeve::Wide => {
            cv.capsule(sh, el, r, r * 0.95, &sleeve, bias);
            cv.capsule(el, wr, r * 0.95, r * 1.45, &sleeve, bias);
        }
        Sleeve::Long | Sleeve::Puff => {
            cv.capsule(sh, el, r, r * 0.88, &sleeve, bias);
            cv.capsule(el, wr, r * 0.88, r * 0.8, &sleeve, bias);
        }
    }
    if kind == Sleeve::Puff {
        cv.begin();
        cv.ellipse(sh.add(v(0.0, 2.2)), r + 2.0, r + 1.8, &sleeve, bias);
    }
    for p in &g.pieces {
        if let Piece::Pauldrons(both, trim) = *p {
            // A lone pauldron sits on the left shoulder: screen right from
            // the front, screen left from behind, the far side in profile.
            let mine = both || match dir {
                DIR_DOWN => sh.x > b.cx,
                DIR_UP => sh.x < b.cx,
                _ => far,
            };
            if mine {
                cv.begin();
                cv.ellipse(sh.add(v(0.0, 1.6)), r + 2.4, r + 1.6, &dye(c, trim), bias);
                cv.begin();
                cv.ellipse(sh.add(v(0.0, 0.4)), r + 2.4, r + 1.6, &c.metal, bias + 0.05);
            }
        }
    }
    // Hands are drawn generously: small hands read as stumps at this size.
    let dirv = v(wr.x - el.x, wr.y - el.y);
    let len = (dirv.x * dirv.x + dirv.y * dirv.y).sqrt().max(0.01);
    let hand = wr.add(v(dirv.x / len * 2.4, dirv.y / len * 2.4));
    cv.begin();
    cv.ellipse(hand, r + 0.3, r + 0.7, &c.skin, bias - 0.06);
    if dir != DIR_UP && !far {
        // Finger creases on the lower half of the fist.
        let hy = (hand.y + 0.5).floor() as i32;
        for dx in [-1.0f32, 1.0] {
            let hx = (hand.x + dx * 1.2).floor() as i32;
            cv.put(hx, hy, c.skin[1]);
            cv.put(hx, hy + 1, c.skin[1]);
        }
    }
    // The same hand in every view: screen right from the front, screen
    // left from behind, the near hand in profile.
    let holding = !far && match dir {
        DIR_DOWN => wr.x > b.cx,
        DIR_UP => wr.x < b.cx,
        _ => true,
    };
    if holding && look.held != held::NONE {
        cv.begin();
        held_item(cv, look, hand, dir, pose.work == 1, c);
    }
}

/// Side-view bust grown out of the chest line, as part of the same garment:
/// a gentle slope from the collarbone to the fullest point, then a round
/// underside curving back to the ribs, shaded darker underneath.
pub(super) fn bust_side(cv: &mut Canvas, b: &Build, by: f32, ramp: &R5, front: impl Fn(f32) -> f32) {
    let r = b.bust;
    let top = by - r * 1.15;
    let apex = by + r * 0.1;
    // Squash pushes it forward and lifts the underside; stretch droops.
    let bottom = by + r * 0.85 + b.jsq;
    // A gentle forward curve grown out of the chest line (2-3 px at this
    // scale, as in the reference pack and LPC), not a ball stuck on: a
    // full radius of depth read as a pillow in profile.
    let depth = r * 0.42 - b.jsq * 0.5;
    for y in top.floor() as i32..=bottom.ceil() as i32 {
        let fy = y as f32 + 0.5;
        if fy < top || fy > bottom {
            continue;
        }
        let (d, under) = if fy < apex {
            let t = (fy - top) / (apex - top);
            (depth * (t * (2.0 - t)).sqrt().min(1.0) * t.powf(0.35), 0.0)
        } else {
            let u = (fy - apex) / (bottom - apex);
            (depth * (1.0 - u * u).max(0.0).sqrt(), u)
        };
        let f = front(fy);
        let x0 = f.floor() as i32 - 1;
        let x1 = (f + d).floor() as i32;
        for x in x0..=x1 {
            let rim = x == x1;
            let i = if under > 0.75 {
                1
            } else if under > 0.35 || (rim && under > 0.0) {
                2
            } else if rim && fy > apex - 3.0 {
                4
            } else {
                3
            };
            cv.fill(x, y, ramp[i]);
        }
    }
}

/// The bust as two rounded forms, each its own part, so the contour pass
/// draws the cleavage and the line beneath. `by` already carries the lag.
pub(super) fn bust(cv: &mut Canvas, b: &Build, dir: u8, by: f32, ramp: &R5) {
    let r = b.bust;
    match dir {
        DIR_SIDE => {
            cv.begin();
            cv.ellipse(v(b.cx + 4.6, by), r * 0.85 - b.jsq * 0.5, r * 0.92 + b.jsq, ramp, 0.0);
        }
        DIR_DOWN => {
            // One soft mass, not two outlined balls: lit on top with a small
            // highlight, shaded underneath, a short cleavage line low in the
            // middle. The top blends into the chest; the contour pass lines
            // only the sides and underside.
            // Squash: a row shorter and half a pixel wider each side;
            // stretch: the opposite.
            let (rx, ry) = (r - b.jsq * 0.5, r * 0.9 + b.jsq);
            cv.begin();
            for y in (by - ry - 1.0).floor() as i32..=(by + ry + 1.0).ceil() as i32 {
                for x in 0..FW {
                    let px = x as f32 + 0.5;
                    let s = if px < b.cx { -1.0 } else { 1.0 };
                    let nx = (px - (b.cx + s * r * 0.92)) / rx;
                    let ny = (y as f32 + 0.5 - by) / ry;
                    if nx * nx + ny * ny > 1.0 + EPS {
                        continue;
                    }
                    // nx measured outward from the cleavage on both sides.
                    let nxo = nx * s;
                    let i = if ny > 0.78 {
                        1
                    } else if ny > 0.4 {
                        2
                    } else if (ny + 0.5 - b.jhl / ry).abs() < 0.1 && nxo > 0.0 && nxo < 0.42 {
                        4
                    } else {
                        3
                    };
                    cv.fill(x, y, ramp[i]);
                }
            }
            let c0 = (by - ry * 0.1).round() as i32;
            for y in c0..(by + ry * 0.75).round() as i32 {
                let col = if y < c0 + 2 { ramp[2] } else { ramp[1] };
                cv.put(31, y, col);
                cv.put(32, y, col);
            }
        }
        _ => {}
    }
}
