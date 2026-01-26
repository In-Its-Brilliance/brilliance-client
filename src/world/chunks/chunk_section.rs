use super::objects_container::ObjectsContainer;
use crate::{
    utils::bridge::{GodotPositionConverter, IntoNetworkVector},
    world::{
        physics::{PhysicsProxy, PhysicsType},
        world_manager::{PLAYER_GROUP, WORLD_FAR_GROUP, WORLD_NEAR_GROUP},
        worlds_manager::WorldMaterials,
    },
};
use common::{
    blocks::chunk_collider_info::ChunkColliderInfo, chunks::chunk_position::ChunkPosition, CHUNK_SIZE,
    CHUNK_SIZE_BOUNDARY,
};
use godot::{
    classes::{ArrayMesh, MeshInstance3D},
    prelude::*,
};
use ndshape::{ConstShape, ConstShape3u32};
use physics::PhysicsColliderBuilder;
use physics::{
    physics::{IPhysicsCollider, IPhysicsColliderBuilder},
    PhysicsCollider,
};
use std::borrow::BorrowMut;

const TRANSPARENCY_SPEED: f32 = 5.0;

//pub type ChunkShape = ConstShape3u32<CHUNK_SIZE_BOUNDARY, CHUNK_SIZE_BOUNDARY, CHUNK_SIZE_BOUNDARY>;
pub type ChunkBordersShape = ConstShape3u32<CHUNK_SIZE_BOUNDARY, CHUNK_SIZE_BOUNDARY, CHUNK_SIZE_BOUNDARY>;

//pub type ChunkData = [BlockInfo; ChunkShape::SIZE as usize];
pub type ChunkColliderDataBordered = [ChunkColliderInfo; ChunkBordersShape::SIZE as usize];

/// One chunk section
/// Contains mesh and data of the chunk section blocks
#[derive(GodotClass)]
#[class(no_init, tool, base=Node3D)]
pub struct ChunkSection {
    pub(crate) base: Base<Node3D>,

    mesh: Gd<MeshInstance3D>,
    mesh_transparent: Gd<MeshInstance3D>,
    objects_container: Gd<ObjectsContainer>,

    chunk_position: ChunkPosition,
    y: u8,

    collider: Option<PhysicsCollider>,
    colider_builder: Option<PhysicsColliderBuilder>,
    need_update_collider: bool,

    set_geometry_first_time: bool,
    transparancy: f32,
}

impl ChunkSection {
    pub fn create(base: Base<Node3D>, materials: &WorldMaterials, y: u8, chunk_position: ChunkPosition) -> Self {
        let mut mesh = MeshInstance3D::new_alloc();
        mesh.set_name(&format!("ChunkMesh {}", y));
        mesh.set_material_override(&materials.get_material_3d());

        let mut mesh_transparent = MeshInstance3D::new_alloc();
        mesh_transparent.set_name(&format!("ChunkMesh {} Transparent", y));
        mesh_transparent.set_material_override(&materials.get_material_3d_transparent());

        Self {
            base,
            mesh,
            mesh_transparent,
            chunk_position,
            y,

            collider: None,
            colider_builder: None,
            need_update_collider: false,

            objects_container: ObjectsContainer::new_alloc(),
            set_geometry_first_time: false,
            transparancy: 1.0,
        }
    }

    pub fn _get_chunk_position(&self) -> &ChunkPosition {
        &self.chunk_position
    }

    pub fn get_section_local_position(&self) -> Vector3 {
        Vector3::new(0.0, GodotPositionConverter::get_chunk_y_local(self.y), 0.0)
    }

    pub fn get_section_position(&self) -> Vector3 {
        Vector3::new(
            self.chunk_position.x as f32 * CHUNK_SIZE as f32,
            GodotPositionConverter::get_chunk_y_local(self.y),
            self.chunk_position.z as f32 * CHUNK_SIZE as f32,
        )
    }

    pub fn get_objects_container_mut(&mut self) -> &mut Gd<ObjectsContainer> {
        &mut self.objects_container
    }

    pub fn set_collider(&mut self, collider_builder: Option<PhysicsColliderBuilder>) {
        self.need_update_collider = true;
        self.colider_builder = collider_builder;
    }

    /// Updates the mesh from a separate thread
    pub fn set_new_mesh(&mut self, new_mesh: &Gd<ArrayMesh>, new_mesh_transparent: &Gd<ArrayMesh>) {
        // Set active only for sections that conatains vertices
        let has_mesh = new_mesh.get_surface_count() > 0 || new_mesh_transparent.get_surface_count() > 0;

        let mesh = self.mesh.borrow_mut();
        mesh.set_mesh(new_mesh);

        let mesh_transparent = self.mesh_transparent.borrow_mut();
        mesh_transparent.set_mesh(new_mesh_transparent);

        if has_mesh && !self.set_geometry_first_time {
            self.set_geometry_first_time = true;
            self.transparancy = 1.0;
            mesh.set_transparency(self.transparancy);
        } else {
            self.base_mut().set_process(false);
        }
    }

    pub fn update_collider_group(&self, is_near: bool) {
        let Some(collider) = self.collider.as_ref() else {
            return;
        };
        if is_near {
            collider.set_collision_mask(WORLD_NEAR_GROUP, PLAYER_GROUP);
        } else {
            collider.set_collision_mask(WORLD_FAR_GROUP, 0);
        }
    }

    pub fn is_collider_update_needed(&self) -> bool {
        self.need_update_collider
    }

    /// Causes an update in the main thread after the entire chunk has been loaded
    pub fn update_collider(&mut self, physics: &PhysicsProxy) {
        self.need_update_collider = false;

        // Set or create new colider
        if let Some(colider_builder) = self.colider_builder.take() {
            if let Some(collider) = self.collider.as_mut() {
                collider.set_shape(colider_builder.get_shape());
            } else {
                let mut collider = physics.create_collider(
                    colider_builder,
                    Some(PhysicsType::ChunkMeshCollider(self.chunk_position.clone())),
                );
                // Указание при создании коллайдера
                collider.set_collision_mask(WORLD_NEAR_GROUP, PLAYER_GROUP);

                let pos = self.get_section_position().clone();
                collider.set_position(pos.to_network());
                self.collider = Some(collider);
            }
        } else {
            // Remove old collider if exists
            if let Some(mut collider) = self.collider.take() {
                collider.remove();
            }
        }
    }

    pub fn destory(&mut self) {
        if let Some(mut collider) = self.collider.take() {
            collider.remove();
        }
        self.objects_container.bind_mut().destory();
    }
}

#[godot_api]
impl INode3D for ChunkSection {
    fn ready(&mut self) {
        let mesh = self.mesh.clone();
        self.base_mut().add_child(&mesh);

        let mesh_transparent = self.mesh_transparent.clone();
        self.base_mut().add_child(&mesh_transparent);

        let objects_container = self.objects_container.clone();
        self.base_mut().add_child(&objects_container);
    }

    fn process(&mut self, delta: f64) {
        #[cfg(feature = "trace")]
        let _span = tracy_client::span!("chunk_section.process");

        let _span = crate::span!("chunk_section.process");

        if self.transparancy > 0.0 {
            let mesh = self.mesh.borrow_mut();
            self.transparancy -= TRANSPARENCY_SPEED * delta as f32;
            mesh.set_transparency(self.transparancy);

            let mesh_transparent = self.mesh_transparent.borrow_mut();
            mesh_transparent.set_transparency(self.transparancy);
        } else {
            self.base_mut().set_process(false);
        }
    }
}
