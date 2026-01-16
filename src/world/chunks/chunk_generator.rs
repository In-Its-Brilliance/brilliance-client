use super::{
    chunk_data_formatter::format_chunk_data_with_boundaries,
    chunk_section::{ChunkColliderDataBordered, ChunkSection},
    chunks_map::ChunkLock,
    mesh::mesh_generator::{build_collider, generate_buffer, generate_mesh},
    near_chunk_data::NearChunksData,
};
use crate::{
    client_scripts::resource_manager::ResourceManager,
    utils::textures::texture_mapper::TextureMapper,
    world::{
        block_storage::BlockStorage,
        physics::PhysicsProxy,
        worlds_manager::{BlockStorageType, TextureMapperType},
    },
};
use common::VERTICAL_SECTIONS;
use flume::Sender;
use godot::{
    classes::Engine,
    obj::{Gd, InstanceId, Singleton},
};

/// Generate chunk data in separate thread
/// and send gd instance id to the main thread to add_child it to the main tree
pub(crate) fn generate_chunk(
    chunk_column: ChunkLock,
    chunks_near: NearChunksData,
    chunks_loaded: Sender<ChunkLock>,
    material_instance_id: InstanceId,

    texture_mapper: TextureMapperType,
    block_storage: BlockStorageType,

    physics: PhysicsProxy,
    resource_manager: &ResourceManager,
) {
    let resources_storage_lock = resource_manager.get_resources_storage_lock();
    rayon::spawn(move || {
        let data = chunk_column.read().get_data_lock().clone();

        let chunk_position = chunk_column.read().get_chunk_position().clone();

        chunk_column.read().spawn_sections(&material_instance_id);
        for y in 0..VERTICAL_SECTIONS {
            let (bordered_chunk_data, mesh_count) = format_chunk_data_with_boundaries(
                Some(&chunks_near),
                &data,
                &*block_storage.read(),
                y,
            )
            .unwrap();

            if mesh_count > 0 {
                let mut chunk_section = chunk_column.read().get_chunk_section(&y);
                generate_chunk_geometry(
                    &mut chunk_section,
                    &texture_mapper.read(),
                    &bordered_chunk_data,
                    &block_storage.read(),
                );

                let mut cs = chunk_section.bind_mut();
                let objects_container = cs.get_objects_container_mut();

                let d = data.read();
                let section_data = d.get(y).unwrap();
                objects_container
                    .bind_mut()
                    .setup(
                        y as u32,
                        &chunk_position,
                        section_data,
                        &*block_storage.read(),
                        &physics,
                        &*resources_storage_lock.read(),
                    )
                    .unwrap();
            }
        }

        chunks_loaded
            .send(chunk_column.clone())
            .expect("chunks_loaded channel poisoned");
    });
}

pub fn generate_chunk_geometry(
    chunk_section: &mut Gd<ChunkSection>,
    texture_mapper: &TextureMapper,
    chunk_collider_data: &ChunkColliderDataBordered,
    block_storage: &BlockStorage,
) {
    let buffer = generate_buffer(chunk_collider_data);
    let mut cs = chunk_section.bind_mut();

    let mesh_ist = generate_mesh(&texture_mapper, &buffer, &block_storage);
    cs.set_new_mesh(&mesh_ist);

    if !Engine::singleton().is_editor_hint() {
        let collider_builder = match mesh_ist.get_surface_count() > 0 {
            true => Some(build_collider(&buffer)),
            false => None,
        };
        cs.set_collider(collider_builder);
    }
}
