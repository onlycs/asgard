use async_std::sync::Arc;
use futures::future::{self, BoxFuture};
use log::error;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    error::Error,
    marker::PhantomData,
};

type Listener<Event, Err> = fn(Arc<Event>) -> ResultFuture<Err>;
type ResultFuture<Err> = BoxFuture<'static, Result<(), Err>>;
type EventList = Vec<Box<dyn Any + Send + Sync>>;

/// # EventEmitter
///
/// The `EventEmitter` is used to emit events and to listen to them. You can listen
/// to an event from anywhere, and emit it from anywhere.
///
/// Only one Error type can be used, for all listeners. Different error types on a
/// per-listener basis cannot be done.
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

    pub fn on<Event: Send + Sync + 'static>(&mut self, listener: Listener<Event, Err>) {
        self.listeners
            .entry(TypeId::of::<Event>())
            .or_default()
            .push(Box::new(listener));
    }

    pub async fn emit<Event: Send + Sync + 'static>(&self, event: Event) {
        let event = Arc::new(event);

        if let Some(event_list) = self.listeners.get(&TypeId::of::<Event>()) {
            let futures = event_list
                .iter()
                .filter_map(|n| n.downcast_ref::<Listener<Event, Err>>())
                .map(|n| async { n(Arc::clone(&event)).await });

            for result in future::join_all(futures).await {
                if let Err(e) = result {
                    error!("Error in event closure: {e}");
                }
            }
        }
    }
}
