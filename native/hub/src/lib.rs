//! This `hub` crate is the
//! entry point of the Rust logic.

mod common;
mod messages;

mod async_event;
mod async_tasks;

use std::{cell::RefCell, sync::OnceLock, time::Duration};

use crate::common::*;
use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use async_event::AppExtensions;
use async_tasks::{TokioTasksHandle, TokioTasksPlugin};
use bevy_app::{AppExit, ScheduleRunnerPlugin};
use bevy_async_ecs::AsyncWorld;
use messages::basic::SmallText;
use os_thread_local::ThreadLocal;
use rinf::{debug_print, DartSignal};
use tokio; // Comment this line to target the web.
// use tokio_with_wasm::alias as tokio; // Uncomment this line to target the web.

rinf::write_interface!();
//static ASYNC_WORLD: OnceLock<AsyncWorld> = OnceLock::new();

// Use `tokio::spawn` to run concurrent tasks.
// Always use non-blocking async functions
// such as `tokio::fs::File::open`.
// If you really need to use blocking code,
// use `tokio::task::spawn_blocking`.
async fn main() {
    tokio::spawn(communicate());

    debug_print!("main() - finished");
}

async fn communicate() -> Result<()> {
    let mut app = App::new();
    app.add_event_channel(SmallText::get_dart_signal_receiver().expect("couldn't get receiver"))
        .add_plugins(TokioTasksPlugin {
            make_handle: Box::new(|| {
                let h = tokio::runtime::Handle::current();
                let _guard = h.enter();
                h
            }),
            ..default()
        })
        /*.add_systems(Startup, |world: &mut World| {
			ASYNC_WORLD.get_or_init(|| {
                AsyncWorld::from_world(world)
            });
		})*/
        //.add_event::<AppExit>()
        .add_systems(Startup, send_initial_number)
        .add_systems(Update, hello)
        .add_systems(Update, listen_for_smalltext)
        .add_systems(Startup, example_system);
        //.run();
        loop {
            app.update();
            std::thread::sleep(Duration::from_millis(100));
            tokio::task::yield_now().await;
        }

    debug_print!("communicate exit...");

    Ok(())
}

fn hello() {
    debug_print!("hello");
}

fn send_initial_number() {
    use messages::basic::*;
    // Send signals to Dart like below.
    SmallNumber { number: 7 }.send_signal_to_dart();
}

fn example_system(handle: ResMut<TokioTasksHandle>) {
    debug_print!("example-system");
    handle.spawn_background_task(|mut ctx| async move {
        debug_print!("This print executes from a background Tokio runtime thread");
        ctx.run_on_main_thread(move |ctx| {
            // The inner context gives access to a mutable Bevy World reference.
            let _world: &mut World = ctx.world;
            debug_print!("MAIN thread here");
        }).await;
    });
    debug_print!("example system finished");
}

fn listen_for_smalltext(mut events: EventReader<DartSignal<SmallText>>) {
    for event in events.read() {
        debug_print!("EVENT: {}", event.message.text);
    }
}