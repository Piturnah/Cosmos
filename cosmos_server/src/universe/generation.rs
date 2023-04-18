use std::f32::consts::{E, TAU};

use bevy::prelude::{Commands, Query, Res, Resource, With};
use cosmos_core::{entities::player::Player, physics::location::Location};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::star::Star;

// Calculates the distance from the origin of a spiral arm given an angle.
fn spiral_function(theta: f32) -> f32 {
    E.powf(theta / 2.0)
}

// Calculates what offset must be necessary for spiral_function to output r given the angle (theta - offset).
// Update this whenever spiral_function is changed.
fn inverse_spiral_function(r: f32, theta: f32) -> f32 {
    theta - 2.0 * r.ln()
}

fn distance_from_star_spiral(x: f32, y: f32) -> f32 {
    // Number of spiral arms in the galaxy.
    let num_spirals: f32 = 8.0;

    let r: f32 = (x * x + y * y).sqrt();
    if r.abs() < 0.0001 {
        // Origin case, trig math gets messed up, but all arms are equally close anyways.
        return spiral_function(0.0);
    }
    let theta: f32 = y.atan2(x);

    let offset: f32 = inverse_spiral_function(r, theta);
    let spiral_index: f32 = (offset * num_spirals / TAU).round();
    let spiral_offset: f32 = spiral_index * TAU / num_spirals;

    (spiral_function(theta - spiral_offset) - r).abs() * (r / 4.0)
}

#[derive(Debug, Resource)]
pub struct ServerSeed(u64);

pub fn get_star_in_system(sx: i64, sy: i64, sz: i64, seed: &ServerSeed) -> Option<Star> {
    if sy != 0 {
        return None;
    }

    let bounds = 100.0;
    let max = 22.0;

    let ratio = max / bounds;

    let at_x = sx as f32 * ratio;
    let at_z = sz as f32 * ratio;

    if at_x.abs() > 1.0 || at_z.abs() > 1.0 {
        return None;
    }

    let seed_x = (at_x + max + 2.0) as u64;
    let seed_z = (at_z + max + 2.0) as u64;

    let local_seed = seed
        .0
        .wrapping_mul(seed_x)
        .wrapping_add(seed_z)
        .wrapping_mul(seed_z)
        .wrapping_sub(seed_x);

    let mut rng = ChaCha8Rng::seed_from_u64(local_seed);

    let distance = distance_from_star_spiral(at_x, at_z);

    let prob = 1.0 / (distance * distance);
    let num = rng.gen_range(0..10_000) as f32 / 10_000.0;

    if num < prob {
        Some(Star::new(1000.0))
    } else {
        None
    }
}

fn load_stars_near_players(
    players: Query<&Location, With<Player>>,
    seed: Res<ServerSeed>,
    mut commands: Commands,
) {
    for loc in players.iter() {
        let (sx, sy, sz) = loc.get_system_coordinates();

        if let Some(star) = get_star_in_system(sx, sy, sz, &seed) {
            commands.spawn(star);
        }
    }
}
