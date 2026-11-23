use std::{
    future::Future,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, PoisonError,
    },
};

use deadpool_postgres::Object;
use futures::future::{BoxFuture, FutureExt, Shared};
use tokio::task::AbortHandle;

use crate::{driver::utils::ConfiguredTLS, runtime::tokio_runtime};

/// A connection checked out of the pool that cancels the queries its
/// callers abandon.
///
/// A query is abandoned when the future awaiting it is dropped before the
/// server answers, which is what happens to a Python coroutine whose task is
/// cancelled, for example by `asyncio.wait_for`. The server keeps running the
/// query, so a cancel request is sent for it. A connection that had a query
/// abandoned is closed when it is dropped instead of going back to the pool,
/// where the next caller would wait behind that query.
pub struct CancellableObject {
    // Taken in `Drop`, which decides whether it goes back to the pool.
    object: ManuallyDrop<Object>,
    tls: ConfiguredTLS,
    abandoned: AtomicBool,
    pending_cancel: Mutex<Option<PendingCancel>>,
}

/// A cancel request being sent to the server.
struct PendingCancel {
    // Tells whether the request is still being sent without polling `sent`.
    task: AbortHandle,
    sent: Shared<BoxFuture<'static, ()>>,
}

impl CancellableObject {
    #[must_use]
    pub fn new(object: Object, tls: ConfiguredTLS) -> Self {
        Self {
            object: ManuallyDrop::new(object),
            tls,
            abandoned: AtomicBool::new(false),
            pending_cancel: Mutex::new(None),
        }
    }

    /// Await `request`, and send a cancel request for it if this future is
    /// dropped before `request` completes.
    pub async fn cancel_on_drop<F: Future>(&self, request: F) -> F::Output {
        // Shrinks the window in which a cancel request meant for an abandoned
        // query stops this one instead. It does not close it: tokio-postgres
        // returns once the request is written, before the server acts on it.
        self.wait_for_pending_cancel().await;

        let guard = CancelOnDrop(self);
        let output = request.await;
        std::mem::forget(guard);
        output
    }

    async fn wait_for_pending_cancel(&self) {
        if !self.abandoned.load(Ordering::Acquire) {
            return;
        }
        let sent = self
            .pending_cancel
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .map(|pending| pending.sent.clone());
        if let Some(sent) = sent {
            sent.await;
        }
    }

    fn request_cancel(&self) {
        self.abandoned.store(true, Ordering::Release);

        let mut pending_cancel = self
            .pending_cancel
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // The server cancels whatever the connection runs when the request
        // arrives. Queries abandoned together, like the statements of an
        // `execute_many`, are covered by one request.
        if pending_cancel
            .as_ref()
            .is_some_and(|pending| !pending.task.is_finished())
        {
            return;
        }

        let token = self.object.cancel_token();
        let tls = self.tls.clone();
        let task = tokio_runtime().spawn(async move {
            // Best effort: when the request cannot be sent, the query keeps
            // running, and the connection is still closed when it is dropped.
            let _ = tls.cancel_query(token).await;
        });
        *pending_cancel = Some(PendingCancel {
            task: task.abort_handle(),
            sent: task.map(|_| ()).boxed().shared(),
        });
    }
}

impl Deref for CancellableObject {
    type Target = Object;

    fn deref(&self) -> &Object {
        &self.object
    }
}

impl DerefMut for CancellableObject {
    fn deref_mut(&mut self) -> &mut Object {
        &mut self.object
    }
}

impl Drop for CancellableObject {
    fn drop(&mut self) {
        // SAFETY: `object` is private to this type, taken only here, and
        // `drop` runs once.
        let object = unsafe { ManuallyDrop::take(&mut self.object) };
        if *self.abandoned.get_mut() {
            drop(Object::take(object));
        }
    }
}

impl std::fmt::Debug for CancellableObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CancellableObject")
            .field("object", &*self.object)
            .field("abandoned", &self.abandoned)
            .finish_non_exhaustive()
    }
}

/// Sends a cancel request when dropped; forgotten once the request completes.
struct CancelOnDrop<'a>(&'a CancellableObject);

impl Drop for CancelOnDrop<'_> {
    fn drop(&mut self) {
        self.0.request_cancel();
    }
}
