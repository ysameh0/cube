use crate::cachestore::CacheStore;
use crate::metastore::MetaStoreEvent;
use crate::util::WorkerLoop;
use crate::CubeError;
use datafusion::cube_ext;
use futures::ready;
use futures_timer::Delay;
use log::error;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::sync::{broadcast, Mutex, MutexGuard};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tokio_util::time::DelayQueue;

#[derive(Debug)]
enum GcTask {
    DeleteQueue(u64),
}

pub struct CacheStoreSchedulerImpl {
    event_receiver: Mutex<Receiver<MetaStoreEvent>>,
    cancel_token: CancellationToken,
    gc_queue: Mutex<DelayQueue<GcTask>>,
    gc_loop: WorkerLoop,
    cache_store: Arc<dyn CacheStore>,
}

crate::di_service!(CacheStoreSchedulerImpl, []);

impl CacheStoreSchedulerImpl {
    pub fn new(
        event_receiver: Receiver<MetaStoreEvent>,
        cache_store: Arc<dyn CacheStore>,
    ) -> CacheStoreSchedulerImpl {
        let cancel_token = CancellationToken::new();

        Self {
            event_receiver: Mutex::new(event_receiver),
            gc_queue: Mutex::new(DelayQueue::new()),
            gc_loop: WorkerLoop::new("GC"),
            cancel_token,
            cache_store,
        }
    }

    pub fn spawn_processing_loops(self: Arc<Self>) -> Vec<JoinHandle<Result<(), CubeError>>> {
        let scheduler1 = self.clone();

        vec![
            cube_ext::spawn(async move {
                scheduler1
                    .gc_loop
                    .process(
                        scheduler1.clone(),
                        async move |_| Ok(Delay::new(Duration::from_secs(30)).await),
                        async move |s, _| s.gc_loop().await,
                    )
                    .await;
                Ok(())
            }),
            cube_ext::spawn(async move {
                self.run_event_processor().await;
                Ok(())
            }),
        ]
    }

    fn collect_gc_queue(
        gc_queue: &mut MutexGuard<DelayQueue<GcTask>>,
        cx: &mut Context<'_>,
    ) -> Poll<Vec<GcTask>> {
        let mut tasks = Vec::new();

        while let Some(entry) = ready!(gc_queue.poll_expired(cx)) {
            tasks.push(entry.into_inner());
        }

        Poll::Ready(tasks)
    }

    pub async fn gc_loop(&self) -> Result<(), CubeError> {
        let to_remove = {
            let mut gc_queue = self.gc_queue.lock().await;
            futures::future::poll_fn(|cx| Self::collect_gc_queue(&mut gc_queue, cx)).await
        };
        println!("to_remove {:?}", to_remove);

        if to_remove.len() > 0 {
            let queue_results_to_remove = to_remove
                .into_iter()
                .map(|i| match i {
                    GcTask::DeleteQueue(id) => id,
                })
                .collect();

            if let Err(e) = self
                .cache_store
                .queue_results_multi_delete(queue_results_to_remove)
                .await
            {
                error!("Error while removing olds queue results: {}", e);
            };
        }

        Ok(())
    }

    async fn run_event_processor(self: Arc<Self>) {
        loop {
            let mut event_receiver = self.event_receiver.lock().await;
            let event: MetaStoreEvent = tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    return;
                }
                event = event_receiver.recv() => {
                    match event {
                        Err(broadcast::error::RecvError::Lagged(messages)) => {
                            error!("Scheduler is lagging on cache store event processing for {} messages", messages);
                            continue;
                        },
                        Err(broadcast::error::RecvError::Closed) => {
                            return;
                        },
                        Ok(event) => event,
                    }
                }
            };

            match event {
                MetaStoreEvent::AckQueueItem(event) => {
                    self.gc_queue
                        .lock()
                        .await
                        .insert(GcTask::DeleteQueue(event.id), Duration::from_secs(120));
                }
                _ => {}
            }
        }
    }

    pub fn stop_processing_loops(&self) -> Result<(), CubeError> {
        self.cancel_token.cancel();
        Ok(())
    }
}
