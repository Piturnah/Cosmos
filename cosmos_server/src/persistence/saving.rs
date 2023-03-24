use bevy::{
    prelude::{App, Commands, Component, CoreSet, Entity, IntoSystemConfig, Query, With, Without},
    reflect::Reflect,
};
use bevy_rapier3d::prelude::Velocity;
use cosmos_core::physics::location::Location;
use rand::{distributions::Alphanumeric, Rng};
use std::fs;

use crate::persistence::get_save_file_path;

use super::{EntityId, SerializedData};

#[derive(Component, Debug, Default, Reflect)]
pub struct NeedsSaved;

fn check_needs_saved(
    query: Query<Entity, (With<NeedsSaved>, Without<SerializedData>)>,
    mut commands: Commands,
) {
    for ent in query.iter() {
        commands.entity(ent).insert(SerializedData::default());
    }
}

/// Make sure any systems that serialize data for saving are run after this
///
/// Make sure those systems are run before `done_saving` aswell.
pub fn begin_saving() {}

/// Make sure any systems that serialize data for saving are run before this
///
/// Make sure those systems are run after `begin_saving` aswell.
pub fn done_saving(
    query: Query<(Entity, &SerializedData, Option<&EntityId>), With<NeedsSaved>>,
    mut commands: Commands,
) {
    for (entity, sd, entity_id) in query.iter() {
        commands
            .entity(entity)
            .remove::<NeedsSaved>()
            .remove::<SerializedData>();

        let serialized = bincode::serialize(&sd).unwrap();

        let entity_id = if let Some(id) = entity_id {
            id.clone()
        } else {
            let res: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(64)
                .map(char::from)
                .collect();

            let entity_id = EntityId(res);

            commands.entity(entity).insert(entity_id.clone());

            entity_id
        };

        let path = get_save_file_path(
            sd.location.map(|l| (l.sector_x, l.sector_y, l.sector_z)),
            &entity_id,
        );

        let directory = &path[0..path.rfind("/").expect("No / found in file path!")];

        if let Err(e) = fs::create_dir_all(directory) {
            eprintln!("{e}");
            continue;
        }

        if let Err(e) = fs::write(path, serialized) {
            eprintln!("{e}");
            continue;
        }
    }
}

fn default_save(
    mut query: Query<(&mut SerializedData, Option<&Location>, Option<&Velocity>), With<NeedsSaved>>,
) {
    for (mut data, loc, vel) in query.iter_mut() {
        if let Some(loc) = loc {
            data.location = Some(*loc);
            // location is a private field, and may change in the future,
            // so serialize it twice to make sure code doesn't break if
            // the location field is changed to be something else.
            data.serialize_data("cosmos:location", &loc);
        }

        if let Some(vel) = vel {
            data.serialize_data("cosmos:velocity", vel);
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_system(check_needs_saved)
        // Put all saving-related systems after this
        .add_system(begin_saving.in_base_set(CoreSet::First))
        // Put all saving-related systems before this
        .add_system(done_saving.after(begin_saving))
        // Like this:
        .add_system(default_save.after(begin_saving).before(done_saving));
}
