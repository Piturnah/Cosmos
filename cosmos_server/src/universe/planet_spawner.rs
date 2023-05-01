//! Responsible for spawning planets near stars, but for now just spawns a planet at 0, 0, 0.

use bevy::prelude::{in_state, App, Commands, IntoSystemConfig, Query, With};
use cosmos_core::{
    entities::player::Player,
    physics::location::{Location, SECTOR_DIMENSIONS},
    structure::{
        planet::{planet_builder::TPlanetBuilder, Planet},
        Structure,
    },
};

use crate::{
    persistence::is_sector_loaded,
    state::GameState,
    structure::planet::{
        biosphere::{grass_biosphere::GrassBiosphere, TBiosphere},
        server_planet_builder::ServerPlanetBuilder,
    },
};

fn spawn_planet(
    query: Query<&Location, With<Planet>>,
    players: Query<&Location, With<Player>>,
    mut commands: Commands,
) {
    if !players.iter().any(|l| {
        l.distance_sqrd(&Location::default()) < (SECTOR_DIMENSIONS * 5.0 * SECTOR_DIMENSIONS * 5.0)
    }) {
        return;
    }

    if is_sector_loaded((0, 0, 0)) {
        return;
    }

    for loc in query.iter() {
        if loc.sector_x == 0 && loc.sector_y == 0 && loc.sector_z == 0 {
            return;
        }
    }

    let loc = Location::default();

    let mut entity_cmd = commands.spawn_empty();

    let mut structure = Structure::new(50, 50, 50);

    let biosphere = GrassBiosphere::default();
    let marker = biosphere.get_marker_component();
    let builder = ServerPlanetBuilder::default();

    builder.insert_planet(&mut entity_cmd, loc, &mut structure);

    entity_cmd.insert(structure).insert(marker);
}

pub(super) fn register(app: &mut App) {
    app.add_system(spawn_planet.run_if(in_state(GameState::Playing)));
}
