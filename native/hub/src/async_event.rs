// This introduces event channels, on one side of which is mpsc::Sender<T>, and on another
// side is bevy's EventReader<T>, and it automatically bridges between the two.

use bevy::{prelude::*, utils::tracing::event};
use bevy_ecs::event::event_update_system;
use tokio::sync::mpsc::UnboundedReceiver;
use std::sync::Mutex;

#[derive(Resource, Deref, DerefMut)]
struct ChannelReceiver<T>(Mutex<UnboundedReceiver<T>>);

pub trait AppExtensions {
    // Allows you to create bevy events using mpsc Sender
    fn add_event_channel<T: Event>(&mut self, receiver: UnboundedReceiver<T>) -> &mut Self;
}

impl AppExtensions for App {
    fn add_event_channel<T: Event>(&mut self, receiver: UnboundedReceiver<T>) -> &mut Self {
        assert!(
            !self.world.contains_resource::<ChannelReceiver<T>>(),
            "this event channel is already initialized",
        );

        self.add_event::<T>();
        self.insert_resource(ChannelReceiver(Mutex::new(receiver)));
        println!("ChannelReceiver added");
        self.add_systems(PreUpdate,
            channel_to_event::<T>
                .after(event_update_system::<T>),
        );
        self
    }
}

fn channel_to_event<T: Event>(
    receiver: Res<ChannelReceiver<T>>,
    mut writer: EventWriter<T>,
) {
    // this should be the only system working with the receiver,
    // thus we always expect to get this lock
    let mut events: std::sync::MutexGuard<UnboundedReceiver<T>> = receiver.lock().expect("unable to acquire mutex lock");
    let mut pending = true;
    while pending {
        match events.try_recv() {
            Ok(event) => {writer.send(event);},
            Err(_e) => {
                pending = false;
            }
        }
    }
}