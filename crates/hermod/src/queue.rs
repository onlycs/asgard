use async_std::{stream::StreamExt, sync::Arc};
use futures::{
    channel::mpsc::{self, SendError},
    future::BoxFuture,
    SinkExt,
};
use std::error::Error;

type ResultFuture = BoxFuture<'static, Result<(), Box<dyn Error>>>;
type Reciever<T, Data> = fn(T, &mut Data) -> ResultFuture;

/// # Sender
///
/// A queue that can be used from anywhere. Wrapper for
/// `futures::channel::mpsc::UnboundedSender` and `UnboundedReceiver`.
///
/// ## Example
/// ```
/// use lazy_static::lazy_static;
/// use hermod::Sender;
/// use std::sync::Arc;
/// use std::error::Error;
/// use async_std::task::spawn_blocking;
///
/// lazy_static! {
/// 	/*
/// 	 * Queue can be used from anywhere. It does not require any mutable references,
/// 	 * and probably should not be used with them
/// 	 */
///     static ref QUEUE: Arc<Sender<String>> = Arc::new(Sender::new(|event, data| Box::pin(async move {
/// 		listener(event).await
/// 	}), 0u32));
/// }
///
/// async fn listener(event: String) -> Result<(), Box<dyn Error>> {
/// 	assert_eq!(event, "Hello, world!");
///
/// 	Ok(())
/// }
///
/// pub fn get_instance() -> Arc<Sender<String>> {
///     Arc::clone(&QUEUE)
/// }
///
///
/// spawn_blocking(|| async move { get_instance().emit("Hello, world!").await; }); // emit takes impl Into<T> as argument
/// ```
pub struct Sender<T: Send + Sync + 'static> {
    sender: mpsc::UnboundedSender<T>,
}

impl<T> Sender<T>
where
    T: Send + Sync + 'static,
{
    pub fn new<D: Send + Sync + 'static>(listener: Reciever<T, D>, data: D) -> Self {
        let (sender, mut receiver) = mpsc::unbounded();

        async_std::task::spawn(async move {
            let mut data = data;

            while let Some(event) = receiver.next().await {
                if let Err(e) = listener(event, &mut data).await {
                    log::error!("Error while handling event: {}", e);
                    continue;
                }
            }
        });

        Sender { sender }
    }

    pub async fn emit(self: Arc<Self>, event: impl Into<T>) -> Result<(), SendError> {
        self.sender.clone().send(event.into()).await
    }
}
