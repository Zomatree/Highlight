use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::lock::Mutex;
use rand::random;
use stoat_models::v0::Message;
use tokio::sync::oneshot;

use crate::Error;

#[derive(Clone)]
struct Waiter<Arg> {
    check: Arc<Box<dyn Fn(&Arg) -> bool + Send + Sync + 'static>>,
    oneshot: Arc<Mutex<Option<oneshot::Sender<Arg>>>>,
}

type WaiterMap<M> = Arc<Mutex<HashMap<usize, Waiter<M>>>>;

#[derive(Default, Debug, Clone)]
pub struct Notifiers {
    messages: WaiterMap<Message>,
    typing_start: WaiterMap<(String, String)>,
}

impl Notifiers {
    async fn inner_wait<F: Fn(&M) -> bool + Send + Sync + 'static, M: Clone>(
        &self,
        waiters: &WaiterMap<M>,
        check: F,
        timeout: Option<Duration>,
    ) -> Result<M, Error> {
        let (sender, receiver) = oneshot::channel();

        let random_value = random();

        {
            let mut lock: futures::lock::MutexGuard<'_, HashMap<usize, Waiter<M>>> =
                waiters.lock().await;

            lock.insert(
                random_value,
                Waiter {
                    check: Arc::new(Box::new(check)),
                    oneshot: Arc::new(Mutex::new(Some(sender))),
                },
            );
        }

        let response = if let Some(timeout) = timeout {
            tokio::time::timeout(timeout, receiver)
                .await
                .map(|res| res.map_err(|_| Error::BrokenChannel))
                .map_err(|_| Error::Timeout)
        } else {
            Ok(receiver.await.map_err(|_| Error::BrokenChannel))
        };

        {
            let mut lock = waiters.lock().await;

            lock.remove(&random_value);
        }

        response?
    }

    async fn inner_invoke<M: Clone>(&self, waiters: &WaiterMap<M>, value: &M) -> Result<(), Error> {
        let lock = waiters.lock().await.clone();

        for (id, waiter) in lock {
            if (waiter.check)(value) {
                if let Some(oneshot) = waiter.oneshot.lock().await.take() {
                    oneshot
                        .send(value.clone())
                        .map_err(|_| Error::BrokenChannel)?;
                }

                waiters.lock().await.remove(&id);
            }
        }

        Ok(())
    }

    pub async fn wait_for_message<F: Fn(&Message) -> bool + Send + Sync + 'static>(
        &self,
        check: F,
        timeout: Option<Duration>,
    ) -> Result<Message, Error> {
        self.inner_wait(&self.messages, check, timeout).await
    }

    pub async fn invoke_message_waiters(&self, message: &Message) -> Result<(), Error> {
        self.inner_invoke(&self.messages, message).await
    }

    pub async fn wait_for_typing_start<F: Fn(&(String, String)) -> bool + Send + Sync + 'static>(
        &self,
        check: F,
        timeout: Option<Duration>,
    ) -> Result<(String, String), Error> {
        self.inner_wait(&self.typing_start, check, timeout).await
    }

    pub async fn invoke_typing_start_waiters(
        &self,
        values: &(String, String),
    ) -> Result<(), Error> {
        self.inner_invoke(&self.typing_start, values).await
    }
}
