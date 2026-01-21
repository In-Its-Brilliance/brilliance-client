use super::{
    block_storage::BlockStorage,
    chunks::chunks_map::ChunkMap,
    physics::PhysicsProxy,
    worlds_manager::{BlockStorageType, TextureMapperType, WorldMaterials},
};
use crate::{
    client_scripts::resource_manager::{ResourceManager, ResourceStorage},
    controller::entity_movement::EntityMovement,
    entities::entities_manager::EntitiesManager,
    utils::bridge::IntoChunkPositionVector,
};
use common::chunks::{
    block_position::BlockPosition,
    chunk_data::{BlockDataInfo, ChunkData},
    chunk_position::ChunkPosition,
};
use godot::prelude::*;

pub const PLAYER_GROUP: u32 = 0b0001;
pub const WORLD_NEAR_GROUP: u32 = 0b0010;
pub const WORLD_FAR_GROUP: u32 = 0b0100;
pub const NEAR_DISTANCE: f32 = 2.0;

/// Godot world
/// Contains all things inside world
///
/// ChunkMap
/// ║
/// ╚ChunkColumn
///  ║
///  ╚ChunkSection
#[derive(GodotClass)]
#[class(no_init, tool, base=Node)]
pub struct WorldManager {
    base: Base<Node>,
    slug: String,
    chunk_map: Gd<ChunkMap>,

    physics: PhysicsProxy,

    entities_manager: Gd<EntitiesManager>,

    texture_mapper: TextureMapperType,
    materials: WorldMaterials,
    block_storage: BlockStorageType,
}

impl WorldManager {
    pub fn create(
        base: Base<Node>,
        slug: String,
        texture_mapper: TextureMapperType,
        materials: WorldMaterials,
        block_storage: BlockStorageType,
    ) -> Self {
        let physics = PhysicsProxy::default();
        let mut chunk_map = Gd::<ChunkMap>::from_init_fn(|base| ChunkMap::create(base));
        chunk_map.bind_mut().base_mut().set_name("ChunkMap");

        Self {
            base,
            slug: slug,
            chunk_map,

            physics,

            entities_manager: Gd::<EntitiesManager>::from_init_fn(|base| EntitiesManager::create(base)),

            texture_mapper,
            materials,
            block_storage,
        }
    }

    pub fn _get_entities_manager(&self) -> GdRef<'_, EntitiesManager> {
        self.entities_manager.bind()
    }

    pub fn get_entities_manager_mut(&mut self) -> GdMut<'_, EntitiesManager> {
        self.entities_manager.bind_mut()
    }

    pub fn get_physics(&self) -> &PhysicsProxy {
        &self.physics
    }

    pub fn get_slug(&self) -> &String {
        &self.slug
    }

    pub fn get_chunk_map(&self) -> GdRef<'_, ChunkMap> {
        self.chunk_map.bind()
    }

    /// Recieve chunk data from network
    pub fn recieve_chunk(&mut self, center: ChunkPosition, chunk_position: ChunkPosition, data: ChunkData) {
        self.chunk_map
            .bind_mut()
            .create_chunk_column(center, chunk_position, data);
    }

    /// Recieve chunk unloaded from network
    pub fn unload_chunk(&mut self, chunk_position: ChunkPosition) {
        self.chunk_map.bind_mut().unload_chunk(chunk_position)
    }

    pub fn edit_block(
        &self,
        position: BlockPosition,
        block_storage: &BlockStorage,
        new_block_info: Option<BlockDataInfo>,
        resource_storage: &ResourceStorage,
    ) -> Result<(), String> {
        self.chunk_map
            .bind()
            .edit_block(position, block_storage, new_block_info, &self.physics, resource_storage)
    }

    pub fn physics_process(&mut self, delta: f64) {
        // Skip physics in tools mode
        if godot::classes::Engine::singleton().is_editor_hint() {
            return;
        }

        #[cfg(feature = "trace")]
        let _span = tracy_client::span!("world_manager.physics_process");

        #[cfg(feature = "trace")]
        let _span = if crate::debug::debug_info::DebugInfo::is_active() {
            Some(crate::debug::PROFILER.span("world_manager.physics_process"))
        } else {
            None
        };

        self.physics.step(delta as f32);
    }

    pub fn custom_process(&mut self, _delta: f64, resource_manager: &ResourceManager) {
        #[cfg(feature = "trace")]
        let _span = tracy_client::span!("world_manager.custom_process");

        #[cfg(feature = "trace")]
        let _span = if crate::debug::debug_info::DebugInfo::is_active() {
            Some(crate::debug::PROFILER.span("world_manager.custom_process"))
        } else {
            None
        };

        {
            #[cfg(feature = "trace")]
            let _span = if crate::debug::debug_info::DebugInfo::is_active() {
                Some(crate::debug::PROFILER.span("world_manager.custom_process::send_chunks_to_load"))
            } else {
                None
            };

            let map = self.chunk_map.bind();
            map.send_chunks_to_load(
                &self.materials,
                self.texture_mapper.clone(),
                self.block_storage.clone(),
                &self.physics,
                resource_manager,
            );
        }

        {
            #[cfg(feature = "trace")]
            let _span = if crate::debug::debug_info::DebugInfo::is_active() {
                Some(crate::debug::PROFILER.span("world_manager.custom_process::spawn_loaded_chunks"))
            } else {
                None
            };

            let mut map = self.chunk_map.bind_mut();
            map.spawn_loaded_chunks(&self.physics);
        }

        {
            #[cfg(feature = "trace")]
            let _span = if crate::debug::debug_info::DebugInfo::is_active() {
                Some(crate::debug::PROFILER.span("world_manager.custom_process::update_geometry"))
            } else {
                None
            };

            let bs = self.block_storage.read();
            let tm = self.texture_mapper.read();
            let map = self.chunk_map.bind();
            map.update_chunks_geometry(&self.physics, &bs, &tm);
        }
    }
}

#[godot_api]
impl WorldManager {
    #[func]
    pub fn handler_player_move(&mut self, movement: Gd<EntityMovement>, new_chunk: bool) {
        if !new_chunk {
            return;
        }
        let new_chunk = movement.bind().get_position().to_chunk_position();
        let chunk_map = self.chunk_map.bind();
        for (_chunk_position, chunk_column_lock) in chunk_map.iter() {
            let chunk_column = chunk_column_lock.read();
            let is_near = chunk_column.get_position().to_chunk_position().get_distance(&new_chunk) < NEAR_DISTANCE;

            chunk_column.update_collider_group(is_near);
        }
    }
}

#[godot_api]
impl INode for WorldManager {
    fn ready(&mut self) {
        let chunk_map = self.chunk_map.clone();
        self.base_mut().add_child(&chunk_map);

        let entities_manager = self.entities_manager.clone();
        self.base_mut().add_child(&entities_manager);
    }
}
