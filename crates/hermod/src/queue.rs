use async_std::{stream::StreamExt, sync::Arc};
use futures::{
    channel::mpsc::{self, SendError, UnboundedReceiver as MRecv, UnboundedSender as MSend},
    future::BoxFuture,
    SinkExt, StreamExt,
};

/// # Sender
///
/// A queue that can be used from anywhere. Wrapper for
/// `futures::channel::mpsc::UnboundedSender` and `UnboundedReceiver`. Calling `.emit()` returns
/// an `UnboundedReciever` when `Ok`. It will recieve one event, and then close (unless sending
/// fails).
///
/// ## Example
/// ```
/// use lazy_static::lazy_static;
/// use std::sync::Arc;
/// use hermod::Sender;
/// use async_std::stream::StreamExt;
///
/// lazy_static! {
///     static ref QUEUE: Arc<Sender<String, u32>> = Arc::new(Sender::new(
///         |event, uref| Box::pin(async move {
///             *uref += 1;
///             println!("{event}");
///             0
///         }), 0u32
///     ));
/// }
///
/// async fn asy_main() {
///     let queue = Arc::clone(&QUEUE);
///
///     let mut res = queue.emit("Hello, world!".to_string()).await.unwrap();
///     assert_eq!(res.next().await.unwrap(), 0);
/// }
///
/// async_std::task::block_on(asy_main());
/// ```
pub struct Sender<T, R>
where
    T: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    sender: MSend<(T, MSend<R>)>,
}

impl<T, R> Sender<T, R>
where
    T: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    pub fn new<D: Send + Sync + 'static>(
        listener: for<'a> fn(T, &'a mut D) -> BoxFuture<'a, R>,
        data: D,
    ) -> Self {
        let (sender, mut receiver) = mpsc::unbounded::<(T, MSend<R>)>();

        async_std::task::spawn(async move {
            let mut data = data;

            while let Some((event, mut sender)) = receiver.next().await {
                let res = listener(event, &mut data).await;

                if let Err(e) = sender.send(res).await {
                    eprintln!("Error sending response: {:?}", e);
                }
            }
        });

        Sender { sender }
    }

    pub async fn emit(self: Arc<Self>, event: impl Into<T>) -> Result<MRecv<R>, SendError> {
        let (sender, receiver) = mpsc::unbounded();
        self.sender.clone().send((event.into(), sender)).await?;

        Ok(receiver)
    }

    pub async fn emit_responseless(self: Arc<Self>, event: impl Into<T>) -> Result<(), SendError> {
        self.sender
            .clone()
            .send((event.into(), mpsc::unbounded().0))
            .await
    }
}
