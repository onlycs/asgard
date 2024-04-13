use async_std::sync::Arc;
use futures::future::{self, BoxFuture};
use log::error;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    error::Error,
    marker::PhantomData,
};

type Listener<Ev, Err> = fn(Arc<<Ev as Event>::Message>) -> ResultFuture<Err>;
type ResultFuture<Err> = BoxFuture<'static, Result<(), Err>>;
type EventList = Vec<Box<dyn Any + Send + Sync>>;

/// # The `Event` Trait
///
/// Specify that a type can be used as an event, and specify
/// the type of data that will be sent to the emitter.
pub trait Event: Send + Sync + 'static {
    type Message: Send + Sync + 'static;
}

/// # EventEmitter
///
/// The `EventEmitter` is used to emit events and to listen
/// to them. You can listen to an event from anywhere, and emit it from anywhere.
///
/// Only one Error type can be used, for all listeners. Different error types on a
/// per-listener basis cannot be done.
///
/// ```no_run
/// use mimir::{Event, EventEmitter};
///
/// pub struct SomethingHappened;
///
/// impl Event for SomethingHappened {
///     type Message = String;
/// }
///
/// let mut emitter = EventEmitter::new();
///
/// emitter.on::<SomethingHappened>(|msg| {
///     assert_eq!(msg, "Hi there!");
/// });
///
/// emitter.emit::<SomethingHappened>(String::from("Hi there!")).await;
/// ```
pub struct EventEmitter<Err: Error + 'static> {
    _phantom: PhantomData<Err>,
    listeners: HashMap<TypeId, EventList>,
}

impl<Err: Error + 'static> EventEmitter<Err> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
            listeners: HashMap::new(),
        }
    }

    pub fn on<Ev: Event>(&mut self, listener: Listener<Ev, Err>) {
        self.listeners
            .entry(TypeId::of::<Ev>())
            .or_default()
            .push(Box::new(listener));
    }

    pub async fn emit<Ev: Event>(&self, arg: Ev::Message) {
        let arg = Arc::new(arg);

        if let Some(event_list) = self.listeners.get(&TypeId::of::<Ev>()) {
            let futures = event_list
                .iter()
                .filter_map(|n| n.downcast_ref::<Listener<Ev, Err>>())
                .map(|n| async { n(Arc::clone(&arg)).await });

            for result in future::join_all(futures).await {
                if let Err(e) = result {
                    error!("Error in callback: {e}");
                }
            }
        }
    }
}
