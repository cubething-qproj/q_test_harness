// Adapted from https://github.com/bevyengine/bevy/blob/main/examples/app/log_layers_ecs.rs

use std::sync::mpsc;

use bevy::{
    asset::uuid::{NoContext, Timestamp},
    log::{
        BoxedLayer, Level,
        tracing::{self, Subscriber},
        tracing_subscriber::{self, Layer},
    },
    prelude::*,
};

/// A basic message. This is what we will be sending from the [`CaptureLayer`] to [`CapturedLogMessages`] non-send resource.
#[derive(Debug, Message)]
pub struct LogMessage {
    pub timestamp: Timestamp,
    pub message: String,
    pub level: Level,
}

/// This non-send resource temporarily stores [`LogMessage`]s before they are
/// written to [`Messages<LogEvent>`] by [`transfer_log_messages`].
#[derive(Deref, DerefMut)]
struct CapturedLogMessages(mpsc::Receiver<LogMessage>);

/// Transfers information from the [`CapturedLogMessages`] resource to [`Messages<LogEvent>`](LogMessage).
fn transfer_log_messages(
    receiver: NonSend<CapturedLogMessages>,
    mut message_writer: MessageWriter<LogMessage>,
) {
    // Make sure to use `try_iter()` and not `iter()` to prevent blocking.
    message_writer.write_batch(receiver.try_iter());
}

/// This is the [`Layer`] that we will use to capture log messages and then send them to Bevy's
/// ECS via its [`mpsc::Sender`].
struct CaptureLayer {
    sender: mpsc::Sender<LogMessage>,
}

impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // In order to obtain the log message, we have to create a struct that implements
        // Visit and holds a reference to our string. Then we use the `record` method and
        // the struct to modify the reference to hold the message string.
        let mut message = None;
        event.record(&mut CaptureLayerVisitor(&mut message));
        if let Some(message) = message {
            let metadata = event.metadata();

            self.sender
                .send(LogMessage {
                    timestamp: Timestamp::now(NoContext),
                    message,
                    level: *metadata.level(),
                })
                .expect("LogEvents resource no longer exists!");
        }
    }
}

/// A [`Visit`](tracing::field::Visit)or that records log messages that are transferred to [`CaptureLayer`].
struct CaptureLayerVisitor<'a>(&'a mut Option<String>);
impl tracing::field::Visit for CaptureLayerVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        // This if statement filters out unneeded events sometimes show up
        if field.name() == "message" {
            *self.0 = Some(format!("{value:?}"));
        }
    }
}
pub(crate) fn custom_layer(app: &mut App) -> Option<BoxedLayer> {
    let (sender, receiver) = mpsc::channel();

    let layer = CaptureLayer { sender };
    let resource = CapturedLogMessages(receiver);

    app.insert_non_send_resource(resource);
    app.add_message::<LogMessage>();
    app.add_systems(Update, transfer_log_messages);

    Some(layer.boxed())
}
