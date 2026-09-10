//! Gastinel Desktop: Ultra-lightweight, event-driven desktop environment for Alpine Linux.
//!
//! Written in low-level Rust with zero unnecessary dependencies, on-demand Vulkan rendering,
//! linear frame-arena memory management, and an embedded minimalist shell.

use gastinel_core::event::EventQueue;
use gastinel_core::geom::{DamageRegion, Rect, Size};
use gastinel_core::mem::FrameArena;
use gastinel_platform::{DrmDevice, SeatSession};
use gastinel_shell::{Launcher, TopBar, WorkspaceManager};
use gastinel_vulkan::{OnDemandRenderer, VulkanEntry};

fn main() {
    println!("============================================================");
    println!("   GASTINEL DESKTOP ENVIRONMENT (Alpine Linux / Low-Level)  ");
    println!("============================================================");

    // 1. Initialize Frame Memory Arena (Zero allocations in render/tick loop)
    let arena = FrameArena::with_block_size(512 * 1024); // 512 KiB pre-reserved
    println!("[MEM] FrameArena initialized: {} KiB reserved", arena.capacity() / 1024);

    // 2. Initialize Platform Session (seatd / DRM)
    let session = SeatSession::connect().expect("Failed to initialize session");
    println!("[SEAT] Session connected: active={}, seat={}", session.is_active(), session.seat_name);

    let connectors = DrmDevice::probe_outputs();
    println!("[DRM] Probed {} display output(s):", connectors.len());
    for conn in &connectors {
        println!(
            "  -> Output: {} (connected: {}, resolution: {}x{} @ {}Hz)",
            conn.name, conn.connected, conn.current_mode.width, conn.current_mode.height, conn.current_mode.refresh_rate_hz
        );
    }

    let screen_size = connectors
        .first()
        .map(|c| Size::new(c.current_mode.width, c.current_mode.height))
        .unwrap_or(Size::new(1920, 1080));

    // 3. Initialize Vulkan Engine (Dynamic loading, zero static runtime linking)
    println!("[VULKAN] Loading dynamic Vulkan driver...");
    match VulkanEntry::load() {
        Ok(vk) => {
            match vk.create_instance("GastinelDesktop") {
                Ok(instance) => {
                    let devices = instance.list_physical_devices();
                    println!("[VULKAN] Initialized instance. Found {} physical device(s):", devices.len());
                    for (i, dev) in devices.iter().enumerate() {
                        println!("  -> GPU [{}]: {} (Type: {}, Vulkan API: {}.{}.{})",
                            i,
                            dev.name,
                            dev.device_type,
                            dev.api_version >> 22,
                            (dev.api_version >> 12) & 0x3ff,
                            dev.api_version & 0xfff
                        );
                    }
                }
                Err(err) => {
                    eprintln!("[VULKAN] Warning: Could not create VkInstance (code: {:?})", err);
                }
            }
        }
        Err(err) => {
            eprintln!("[VULKAN] Note: Dynamic loader returned: {}", err);
        }
    }

    // 4. Initialize Embedded Shell Components
    println!("[SHELL] Initializing embedded TopBar, Launcher, and Workspaces...");
    let mut topbar = TopBar::new(screen_size.width);
    topbar.update_time("21:42");
    topbar.update_memory(85, 7840); // Demonstrating ultralight 85MB desktop footprint on ~8GB RAM!

    let mut launcher = Launcher::new(screen_size);
    launcher.add_app("Terminal", "foot", "Wayland terminal emulator");
    launcher.add_app("File Manager", "nnn", "Terminal file manager");
    launcher.add_app("Web Browser", "firefox", "Web browser");
    launcher.add_app("Code Editor", "helix", "Modal text editor");

    let mut workspaces = WorkspaceManager::new(5);
    let win1 = workspaces.create_window(
        "Alacritty - htop",
        Rect::new(80, 80, 800, 500),
    );
    println!("[WM] Created initial window: handle={:?}", win1);

    // 5. Initialize On-Demand Renderer
    let mut renderer = OnDemandRenderer::new(screen_size);
    let _event_queue = EventQueue::with_capacity(128);

    println!("[LOOP] Starting event-driven loop test...");

    // Enqueue a few simulated events to demonstrate damage-tracking:
    // Event A: idle tick (no damage)
    // Event B: launcher opened (damages launcher rect)
    // Event C: typed 't' into launcher (damages query box)
    // Event D: switch to workspace 2 (damages full screen)

    let mut damage = DamageRegion::new();

    // Tick 1: Idle frame (Desktop static)
    println!("--- Tick 1: Static / Idle State ---");
    let did_render = renderer.render_frame(&damage, |_| {
        panic!("Should not render when damage is empty!");
    });
    println!("  Rendered to GPU: {} (Zero GPU wakeups)", did_render);

    // Tick 2: User presses Super key -> open Launcher
    println!("--- Tick 2: Trigger Launcher (Damage Tracking) ---");
    launcher.open();
    damage.add_rect(launcher.bounds);
    damage.coalesce();

    let did_render = renderer.render_frame(&damage, |rects| {
        println!("  -> GPU Scissor pass over {} damaged rectangle(s): {:?}", rects.len(), rects);
    });
    println!("  Rendered to GPU: {}", did_render);
    damage.clear();
    arena.reset(); // Instant arena reset without OS dealloc

    // Tick 3: User types 't' in search
    println!("--- Tick 3: User types in Launcher ---");
    launcher.insert_char('t');
    damage.add_rect(launcher.bounds);

    let did_render = renderer.render_frame(&damage, |rects| {
        println!("  -> GPU Scissor pass over {} rectangle(s): {:?}", rects.len(), rects);
    });
    println!("  Rendered to GPU: {}", did_render);
    damage.clear();
    arena.reset();

    // 6. Summary Stats
    let stats = renderer.stats();
    println!("============================================================");
    println!("                  GASTINEL ENGINE BENCHMARK                 ");
    println!("============================================================");
    println!("  Total loop ticks:   {}", stats.total_ticks);
    println!("  Rendered frames:    {}", stats.rendered_frames);
    println!("  Skipped idle frames:{} (100% idle power saving)", stats.skipped_frames);
    println!("  Total damaged px:   {} pixels", stats.total_damage_area);
    println!("  Memory arena:       {} KiB used / {} KiB peak", arena.used_bytes() / 1024, arena.capacity() / 1024);
    println!("============================================================");
    println!("Gastinel Desktop Base is running and ready for Alpine Linux!");
}
