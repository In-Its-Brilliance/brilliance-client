use godot::classes::base_material_3d::TextureParam;
use godot::classes::StandardMaterial3D;
use godot::prelude::*;
use godot::{classes::Material, prelude::Gd};
use parking_lot::lock_api::{RwLockReadGuard, RwLockWriteGuard};
use parking_lot::RwLock;
use std::sync::Arc;

use super::block_storage::BlockStorage;
use super::world_manager::{WorldManager, NEAR_DISTANCE};
use crate::client_scripts::resource_manager::ResourceStorage;
use crate::controller::entity_movement::EntityMovement;
use crate::controller::player_controller::PlayerController;
use crate::scenes::components::block_mesh_storage::BlockMeshStorage;
use crate::scenes::main_scene::ResourceManagerType;
use crate::utils::bridge::{ChunkPositionGd, IntoChunkPositionVector};
use crate::utils::textures::texture_mapper::TextureMapper;

pub type TextureMapperType = Arc<RwLock<TextureMapper>>;
pub type BlockStorageType = Arc<RwLock<BlockStorage>>;

#[derive(Clone)]
pub struct WorldMaterials {
    material_3d_id: InstanceId,
    material_3d_transparent_id: InstanceId,
}

impl WorldMaterials {
    pub fn create(material_3d: Gd<Material>, material_3d_transparent: Gd<Material>) -> Self {
        Self {
            material_3d_id: material_3d.instance_id(),
            material_3d_transparent_id: material_3d_transparent.instance_id(),
        }
    }

    pub fn get_material_3d(&self) -> Gd<Material> {
        let material: Gd<Material> = Gd::from_instance_id(self.material_3d_id);
        material
    }

    pub fn get_material_3d_transparent(&self) -> Gd<Material> {
        let material: Gd<Material> = Gd::from_instance_id(self.material_3d_transparent_id);
        material
    }
}

#[derive(GodotClass)]
#[class(init, tool, base=Node)]
pub struct WorldsManager {
    base: Base<Node>,

    world: Option<Gd<WorldManager>>,
    player_controller: Option<Gd<PlayerController>>,

    pub(crate) resource_manager: Option<ResourceManagerType>,

    #[init(val = Arc::new(RwLock::new(Default::default())))]
    texture_mapper: TextureMapperType,

    #[init(val = Arc::new(RwLock::new(Default::default())))]
    block_storage: BlockStorageType,

    #[export]
    terrain_material: Option<Gd<StandardMaterial3D>>,

    #[export]
    terrain_material_transparent: Option<Gd<StandardMaterial3D>>,

    block_mesh_storage: Option<Gd<BlockMeshStorage>>,
}

impl WorldsManager {
    pub fn build_textures(&mut self, resources_storage: &ResourceStorage) -> Result<(), String> {
        let now = std::time::Instant::now();

        let mut texture_mapper = self.texture_mapper.write();
        let block_storage = self.block_storage.read();

        texture_mapper.clear();

        let image_texture = match texture_mapper.build(&*block_storage, resources_storage) {
            Ok(i) => i,
            Err(e) => return Err(e),
        };

        let material_3d = self.terrain_material.as_mut().expect("terrain_material is not set");
        material_3d.set_texture(TextureParam::ALBEDO, &image_texture);

        log::info!(target: "main", "Textures builded successfily; texture blocks:{} textures loaded:{} (executed:{:.2?})", block_storage.textures_blocks_count(), texture_mapper.len(), now.elapsed());
        return Ok(());
    }

    pub fn get_resource_manager(&self) -> std::cell::Ref<'_, crate::client_scripts::resource_manager::ResourceManager> {
        self.resource_manager.as_ref().unwrap().borrow()
    }

    pub fn on_network_connected(&mut self) {
        let block_mesh_storage = {
            BlockMeshStorage::init(
                &*self.get_block_storage(),
                &self.get_materials(),
                &self.get_resource_manager(),
                &*self.get_texture_mapper(),
            )
        };
        self.block_mesh_storage = Some(block_mesh_storage);
    }

    pub fn get_block_mesh_storage(&self) -> Option<&Gd<BlockMeshStorage>> {
        self.block_mesh_storage.as_ref()
    }

    pub fn get_block_storage_lock(&self) -> &BlockStorageType {
        &self.block_storage
    }

    pub fn get_block_storage(&self) -> RwLockReadGuard<'_, parking_lot::RawRwLock, BlockStorage> {
        self.block_storage.read()
    }

    pub fn get_block_storage_mut(&self) -> RwLockWriteGuard<'_, parking_lot::RawRwLock, BlockStorage> {
        self.block_storage.write()
    }

    pub fn get_texture_mapper(&self) -> RwLockReadGuard<'_, parking_lot::RawRwLock, TextureMapper> {
        self.texture_mapper.read()
    }

    pub fn get_world(&self) -> Option<&Gd<WorldManager>> {
        match self.world.as_ref() {
            Some(w) => Some(&w),
            None => None,
        }
    }

    pub fn get_world_mut(&mut self) -> Option<&mut Gd<WorldManager>> {
        match self.world.as_mut() {
            Some(w) => Some(w),
            None => None,
        }
    }

    pub fn get_player_controller(&self) -> &Option<Gd<PlayerController>> {
        &self.player_controller
    }

    pub fn get_player_controller_mut(&mut self) -> &mut Option<Gd<PlayerController>> {
        &mut self.player_controller
    }

    pub fn create_player(&mut self, world: &Gd<WorldManager>) -> Gd<PlayerController> {
        let player_controller = Gd::<PlayerController>::from_init_fn(|base| {
            PlayerController::create(base, world.bind().get_physics().clone())
        });

        self.base_mut().add_child(&player_controller.clone());

        self.player_controller = Some(player_controller.clone());
        player_controller
    }

    pub fn get_materials(&self) -> WorldMaterials {
        let material_3d = self
            .terrain_material
            .as_ref()
            .expect("Terrain StandardMaterial3D is not set")
            .clone();
        let material_3d_transparent = self
            .terrain_material_transparent
            .as_ref()
            .expect("Terrain StandardMaterial3D is not set")
            .clone();
        WorldMaterials::create(
            material_3d.upcast::<Material>(),
            material_3d_transparent.upcast::<Material>(),
        )
    }

    pub fn create_world(&mut self, world_slug: String) -> Gd<WorldManager> {
        let now = std::time::Instant::now();

        let mut world = Gd::<WorldManager>::from_init_fn(|base| {
            WorldManager::create(
                base,
                world_slug.clone(),
                self.texture_mapper.clone(),
                self.get_materials(),
                self.block_storage.clone(),
                self.resource_manager.as_ref().unwrap().clone(),
            )
        });

        world
            .bind_mut()
            .base_mut()
            .set_name(&format!("World \"{}\"", world_slug));

        self.base_mut().add_child(&world);

        {
            let mut world = world.bind_mut();
            let mut chunk_map = world.get_chunk_map_mut();
            chunk_map
                .signals()
                .chunk_loeded()
                .connect_other(&self.to_gd(), Self::chunk_loeded);
        }

        self.world = Some(world.clone());

        log::info!(target: "world", "World \"{}\" created; (executed:{:.2?})", self.world.as_ref().unwrap().bind().get_slug(), now.elapsed());

        world
    }

    pub fn destroy_world(&mut self) {
        let now = std::time::Instant::now();

        let mut base = self.base_mut().clone();

        let world_slug;
        if let Some(world) = self.world.as_mut().take() {
            world_slug = world.bind().get_slug().clone();
            base.remove_child(&world.clone());
        } else {
            panic!("destroy_world: world is not exists");
        }

        if let Some(player_controller) = self.player_controller.as_mut().take() {
            base.remove_child(&player_controller.clone());
        }
        log::info!(target: "world", "World \"{}\" destroyed; (executed:{:.2?})", world_slug, now.elapsed());
    }
}

#[godot_api]
impl WorldsManager {
    #[func]
    pub fn handler_player_move(&mut self, movement: Gd<EntityMovement>, new_chunk: bool) {
        #[cfg(feature = "trace")]
        let _span = tracy_client::span!("worlds_manager.handler_player_move");

        let _span = crate::span!("worlds_manager.handler_player_move");

        let world = self.world.as_ref().unwrap().bind();

        let chunk_map = world.get_chunk_map();

        if let Some(player_controller) = self.player_controller.as_mut() {
            let chunk_pos = player_controller.get_position().to_chunk_position();

            let chunk_loaded = match chunk_map.get_chunk(&chunk_pos) {
                Some(c) => c.read().is_loaded(),
                None => false,
            };
            player_controller.bind_mut().set_frozen(!chunk_loaded);
        }

        if new_chunk {
            let new_chunk = movement.bind().get_position().to_chunk_position();
            for (_chunk_position, chunk_column_lock) in chunk_map.iter() {
                let chunk_column = chunk_column_lock.read();
                let is_near = chunk_column.get_position().to_chunk_position().get_distance(&new_chunk) < NEAR_DISTANCE;

                chunk_column.update_collider_group(is_near);
            }
        }
    }

    pub fn chunk_loeded(&mut self, chunk_position: Gd<ChunkPositionGd>) {
        let chunk_position = chunk_position.bind().get_inner().clone();

        if let Some(player_controller) = self.player_controller.as_ref() {
            let chunk_pos = player_controller.get_position().to_chunk_position();

            if chunk_pos == chunk_position {
                player_controller.bind().set_frozen(false);
            }
        }
    }
}

#[godot_api]
impl INode for WorldsManager {}
