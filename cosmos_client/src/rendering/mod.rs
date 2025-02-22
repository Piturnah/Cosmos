//! Handles most of the rendering logic

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use cosmos_core::{
    block::{Block, BlockFace},
    registry::{
        identifiable::Identifiable,
        many_to_one::{self, ManyToOneRegistry},
        Registry,
    },
};

use crate::state::game_state::GameState;

mod structure_renderer;

#[derive(Component, Debug)]
/// The player's active camera will have this component
pub struct MainCamera;

#[derive(Default, Debug, Reflect, FromReflect, Clone)]
/// Stores all the needed information for a mesh
pub struct MeshInformation {
    /// The indicies of the model
    pub indices: Vec<u32>,
    /// The uv coordinates of the model - not based on the atlas but based off the texture for just this block. For example, stone's uvs are in the range of [0.0, 1.0].
    pub uvs: Vec<[f32; 2]>,
    /// The positions of this model, where (0.0, 0.0, 0.0) is the center. A default block's range is [-0.5, 0.5]
    pub positions: Vec<[f32; 3]>,
    /// The normals for the faces of the model.
    pub normals: Vec<[f32; 3]>,
}

impl MeshInformation {
    /// Scales this mesh by a given amount
    pub fn scale(&mut self, scale: Vec3) {
        self.positions.iter_mut().for_each(|x| {
            x[0] *= scale.x;
            x[1] *= scale.y;
            x[2] *= scale.z;
        });
    }
}

#[derive(Default, Debug, Reflect, FromReflect)]
/// Default way to create a mesh from many different combined `MeshInformation` objects.
pub struct CosmosMeshBuilder {
    last_index: u32,
    indices: Vec<u32>,
    uvs: Vec<[f32; 2]>,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
}

/// Used to create a mesh from many different combined `MeshInformation` objects.
///
/// Implemented by default in `CosmosMeshBuilder`
pub trait MeshBuilder {
    /// Adds the information to this mesh builder
    fn add_mesh_information(&mut self, mesh_info: &MeshInformation, position: Vec3, uvs: Rect);

    /// Creates the bevy mesh from the information given so far
    fn build_mesh(self) -> Mesh;
}

impl MeshBuilder for CosmosMeshBuilder {
    #[inline]
    fn add_mesh_information(&mut self, mesh_info: &MeshInformation, position: Vec3, uvs: Rect) {
        let diff = [uvs.max.x - uvs.min.x, uvs.max.y - uvs.min.y];

        let mut max_index = -1;

        self.positions.extend(
            mesh_info
                .positions
                .iter()
                .map(|x| [x[0] + position.x, x[1] + position.y, x[2] + position.z]),
        );
        self.normals.extend(mesh_info.normals.iter());

        self.uvs.extend(
            mesh_info
                .uvs
                .iter()
                .map(|x| [x[0] * diff[0] + uvs.min.x, x[1] * diff[1] + uvs.min.y]),
        );

        for index in mesh_info.indices.iter() {
            self.indices.push(*index + self.last_index);
            max_index = max_index.max(*index as i32);
        }

        self.last_index += (max_index + 1) as u32;
    }

    fn build_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.set_indices(Some(Indices::U32(self.indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);

        mesh
    }
}

#[derive(Default, Debug, Reflect, FromReflect)]
/// Stores all the mesh information for a block
pub struct BlockMeshInformation {
    /// Make sure this is in the same order as the [`BlockFace::index`] method.
    mesh_info: [MeshInformation; 6],

    id: u16,
    unlocalized_name: String,
}

impl Identifiable for BlockMeshInformation {
    fn id(&self) -> u16 {
        self.id
    }

    fn unlocalized_name(&self) -> &str {
        &self.unlocalized_name
    }

    fn set_numeric_id(&mut self, id: u16) {
        self.id = id;
    }
}

impl BlockMeshInformation {
    /// Creates the mesh information for a block.
    ///
    /// Make sure the mesh information is given in the proper order
    pub fn new(
        unlocalized_name: impl Into<String>,

        right: MeshInformation,
        left: MeshInformation,
        top: MeshInformation,
        bottom: MeshInformation,
        front: MeshInformation,
        back: MeshInformation,
    ) -> Self {
        // If this ever fails, change the `mesh_info` ordering + comment below
        debug_assert!(BlockFace::Right.index() == 0);
        debug_assert!(BlockFace::Left.index() == 1);
        debug_assert!(BlockFace::Top.index() == 2);
        debug_assert!(BlockFace::Bottom.index() == 3);
        debug_assert!(BlockFace::Front.index() == 4);
        debug_assert!(BlockFace::Back.index() == 5);

        Self {
            /*
               BlockFace::Right => 0,
               BlockFace::Left => 1,
               BlockFace::Top => 2,
               BlockFace::Bottom => 3,
               BlockFace::Front => 4,
               BlockFace::Back => 5,
            */
            mesh_info: [right, left, top, bottom, front, back],
            id: 0,
            unlocalized_name: unlocalized_name.into(),
        }
    }

    /// Gets the mesh information for that block face
    pub fn info_for_face(&self, face: BlockFace) -> &MeshInformation {
        &self.mesh_info[face.index()]
    }
}

fn register_meshes(mut registry: ResMut<BlockMeshRegistry>) {
    // Model for a basic cube.
    registry.insert_value(BlockMeshInformation::new(
        "cosmos:base_block",
        MeshInformation {
            indices: vec![0, 1, 2, 2, 3, 0],
            uvs: vec![[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            positions: vec![
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [0.5, 0.5, 0.5],
                [0.5, -0.5, 0.5],
            ],
            normals: [[1.0, 0.0, 0.0]; 4].to_vec(),
        },
        MeshInformation {
            indices: vec![0, 1, 2, 2, 3, 0],
            uvs: vec![[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            positions: vec![
                [-0.5, -0.5, 0.5],
                [-0.5, 0.5, 0.5],
                [-0.5, 0.5, -0.5],
                [-0.5, -0.5, -0.5],
            ],
            normals: [[-1.0, 0.0, 0.0]; 4].to_vec(),
        },
        MeshInformation {
            indices: vec![0, 1, 2, 2, 3, 0],
            uvs: vec![[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
            positions: vec![
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [-0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
            ],
            normals: [[0.0, 1.0, 0.0]; 4].to_vec(),
        },
        MeshInformation {
            indices: vec![0, 1, 2, 2, 3, 0],
            uvs: vec![[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            positions: vec![
                [0.5, -0.5, 0.5],
                [-0.5, -0.5, 0.5],
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
            ],
            normals: [[0.0, -1.0, 0.0]; 4].to_vec(),
        },
        MeshInformation {
            indices: vec![0, 1, 2, 2, 3, 0],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            positions: vec![
                [-0.5, 0.5, -0.5],
                [0.5, 0.5, -0.5],
                [0.5, -0.5, -0.5],
                [-0.5, -0.5, -0.5],
            ],
            normals: [[0.0, 0.0, -1.0]; 4].to_vec(),
        },
        MeshInformation {
            indices: vec![0, 1, 2, 2, 3, 0],
            uvs: vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
            positions: vec![
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
            normals: [[0.0, 0.0, 1.0]; 4].to_vec(),
        },
    ));
}

fn register_block_meshes(
    blocks: Res<Registry<Block>>,
    mut model_registry: ResMut<BlockMeshRegistry>,
) {
    for block in blocks.iter() {
        model_registry
            .add_link(block, "cosmos:base_block")
            .expect("cosmos:base_block model link wasn't inserted successfully!");
    }
}

/// This is a `ManyToOneRegistry` mapping Blocks to `BlockMeshInformation`.
pub type BlockMeshRegistry = ManyToOneRegistry<Block, BlockMeshInformation>;

pub(super) fn register(app: &mut App) {
    many_to_one::create_many_to_one_registry::<Block, BlockMeshInformation>(app);
    structure_renderer::register(app);

    app.add_systems((
        register_meshes.in_schedule(OnEnter(GameState::Loading)),
        register_block_meshes.in_schedule(OnExit(GameState::PostLoading)),
    ));
}
