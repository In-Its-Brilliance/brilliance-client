use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::world::{chunks::chunks_map::LIMIT_CHUNK_LOADING_AT_A_TIME, worlds_manager::WorldsManager};
use common::chunks::block_position::{BlockPosition, BlockPositionTrait};
use godot::{
    classes::{
        performance::Monitor, rendering_server::RenderingInfo, Engine, HBoxContainer, IMarginContainer, MarginContainer, Performance, RenderingServer, RichTextLabel, VBoxContainer
    },
    prelude::*,
};
use lazy_static::lazy_static;
use network::client::NetworkInfo;

lazy_static! {
    static ref DEBUG_ACTIVE: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

fn fps_bbcode_color(fps: f64) -> &'static str {
    const COLORS: [&str; 6] = [
        "#D96B6B", // <20
        "#E08A5C", // 20
        "#E6B85C", // 30
        "#B8D35C", // 40
        "#7FD96B", // 50
        "#5CDE7A", // 60+
    ];

    let idx = ((fps / 10.0).floor() as isize - 2).clamp(0, 5) as usize;
    COLORS[idx]
}

macro_rules! debug_first_string {
    () => {
        "[b]FPS: [color={fps_color}]{fps:.0}[/color][/b]
[color=#B3B3B3]Process:[/color] {process:.1}ms
[color=#B3B3B3]Physics:[/color] {physics:.1}ms
[b]Currently rendering:[/b]
[color=#B3B3B3]Objects:[/color] {total_objects_in_frame}
[color=#B3B3B3]Primitives:[/color] {total_primitives_in_frame:.1}K
[color=#B3B3B3]Draw calls:[/color] {total_draw_calls_in_frame}
[color=#B3B3B3]Video mem used:[/color] {video_mem_used:.1} MB"
    };
}
macro_rules! debug_world_string {
    () => {
        "[b]World: [color=#6FA8FF]{world_slug}[/color][/b]
[color=#B3B3B3]Position:[/color] {controller_positioin}
[color=#B3B3B3]Character state:[/color] {current_animation}
[color=#B3B3B3]Chunks:[/color] {chunks_count} [color=#B3B3B3]loading: [/color]{chunks_loading}/{loading_limit} [color=#B3B3B3]waiting: [/color]{chunks_waiting}
[color=#B3B3B3]Chunk position:[/color] {chunk_pos}
[color=#B3B3B3]Chunk info:[/color] {chunk_info}
[color=#B3B3B3]Look at:[/color] {look_at_message}
"
    };
}
macro_rules! debug_network_string {
    () => {
        "[b]Network connected: {is_connected}[/b]
[color=#B3B3B3]Received:[/color] {received_per_sec:.1} KB/sec
[color=#B3B3B3]Sent:[/color] {sent_per_sec:.1} KB/sec
[color=#B3B3B3]Packet loss:[/color] {packet_loss:.1}"
    };
}

#[derive(GodotClass)]
#[class(base=MarginContainer)]
pub struct DebugInfo {
    base: Base<MarginContainer>,
    first_row: Gd<HBoxContainer>,
    world_row: Gd<HBoxContainer>,
    network_row: Gd<HBoxContainer>,
}

impl DebugInfo {
    pub fn load_row() -> Gd<HBoxContainer> {
        load::<PackedScene>("res://scenes/debug_row.tscn").instantiate_as::<HBoxContainer>()
    }

    pub fn change_text(row: &Gd<HBoxContainer>, new_text: String) {
        let mut text = row.get_node_as::<RichTextLabel>("PanelContainer/MarginContainer/RichTextLabel");
        text.set_text(&new_text);
    }

    pub fn is_active() -> bool {
        DEBUG_ACTIVE.load(Ordering::Relaxed)
    }

    pub fn toggle(&mut self, state: bool) {
        DEBUG_ACTIVE.store(state, Ordering::Relaxed);

        self.base_mut().set_visible(DebugInfo::is_active());
    }

    pub fn update_debug(&mut self, worlds_manager: &Gd<WorldsManager>, network_info: NetworkInfo) {
        if !DebugInfo::is_active() {
            return;
        }

        let mut rendering_server = RenderingServer::singleton();
        let performance = Performance::singleton();
        let first_text = format!(
            debug_first_string!(),
            fps_color=fps_bbcode_color(Engine::singleton().get_frames_per_second()),
            fps=Engine::singleton().get_frames_per_second(),
            process = performance.get_monitor(Monitor::TIME_PROCESS),
            physics = performance.get_monitor(Monitor::TIME_PHYSICS_PROCESS),
            total_objects_in_frame=rendering_server.get_rendering_info(RenderingInfo::TOTAL_OBJECTS_IN_FRAME),
            total_primitives_in_frame=rendering_server.get_rendering_info(RenderingInfo::TOTAL_PRIMITIVES_IN_FRAME) as f32 * 0.001,
            total_draw_calls_in_frame=rendering_server.get_rendering_info(RenderingInfo::TOTAL_DRAW_CALLS_IN_FRAME),
            video_mem_used=rendering_server.get_rendering_info(RenderingInfo::VIDEO_MEM_USED) as f32 / (1024.0 * 1024.0),
        );
        DebugInfo::change_text(&self.first_row, first_text);

        let wm = worlds_manager.bind();
        let world_text = match wm.get_world() {
            Some(w) => {
                let player_controller = wm.get_player_controller().as_ref().unwrap().bind();
                let world = w.bind();
                let controller_pos = player_controller.get_position();
                let controller_positioin = format!(
                    "{:.1} {:.1} {:.1} [color=#B3B3B3]y:[/color]{:.0} [color=#B3B3B3]p:[/color]{:.0}",
                    controller_pos.x,
                    controller_pos.y,
                    controller_pos.z,
                    player_controller.get_yaw(),
                    player_controller.get_pitch(),
                );

                let chunk_pos = BlockPosition::new(
                    controller_pos.x as i64,
                    controller_pos.y as i64,
                    controller_pos.z as i64,
                )
                .get_chunk_position();

                let chunk_info = match world.get_chunk_map().get_chunk(&chunk_pos) {
                    Some(c) => {
                        let c = c.read();
                        format!("loaded:{}", c.is_loaded())
                    }
                    None => "-".to_string(),
                };

                let chunk_map = world.get_chunk_map();
                format!(
                    debug_world_string!(),
                    world_slug=world.get_slug(),
                    controller_positioin=controller_positioin,
                    current_animation=match player_controller.get_current_animation() {
                        Some(s) => s,
                        None => String::from("-"),
                    },
                    chunks_count=chunk_map.get_loaded_chunks_count(),
                    chunks_loading=chunk_map.get_loading_chunks_count(),
                    loading_limit=LIMIT_CHUNK_LOADING_AT_A_TIME,
                    chunks_waiting=chunk_map.get_waiting_chunks_count(),
                    chunk_pos=chunk_pos,
                    chunk_info=chunk_info,
                    look_at_message=player_controller.get_look_at_message(),
                )
            }
            None => "World: -".to_string(),
        };
        DebugInfo::change_text(&self.world_row, world_text);

        let network_text = format!(
            debug_network_string!(),
            is_connected=!network_info.is_disconnected,
            received_per_sec=network_info.bytes_received_per_sec / 1024.0,
            sent_per_sec=network_info.bytes_sent_per_sec / 1024.0,
            packet_loss=network_info.packet_loss / 1024.0,
        );
        DebugInfo::change_text(&self.network_row, network_text);
    }
}

#[godot_api]
impl IMarginContainer for DebugInfo {
    fn init(base: Base<MarginContainer>) -> Self {
        Self {
            base: base,
            first_row: DebugInfo::load_row(),
            world_row: DebugInfo::load_row(),
            network_row: DebugInfo::load_row(),
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_visible(false);

        let mut base = self
            .base()
            .get_node_as::<VBoxContainer>("MarginContainer/VBoxContainer");
        base.add_child(&self.first_row);
        base.add_child(&self.world_row);
        base.add_child(&self.network_row);
    }
}
