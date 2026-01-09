#[allow(unused_imports)]
use std::sync::mpsc;

#[cfg(not(target_arch = "wasm32"))]
use std::thread;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::collections::VecDeque;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

/// A cross-platform background task executor
/// On native: uses real threads (true parallelism)
/// On WASM: uses spawn_local to defer work to next event loop tick (non-blocking for current frame)
pub struct BackgroundTask<T, R> {
    #[cfg(not(target_arch = "wasm32"))]
    receiver: Option<mpsc::Receiver<(T, Result<R, String>)>>,
    #[cfg(target_arch = "wasm32")]
    completed_results: Rc<RefCell<VecDeque<(T, Result<R, String>)>>>,
}

impl<T: Send + 'static, R: Send + 'static> BackgroundTask<T, R> {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            receiver: None,
            #[cfg(target_arch = "wasm32")]
            completed_results: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    /// Execute a task in the background
    /// `task_id`: Identifier for the task (used when retrieving results)
    /// `work`: The work function to execute
    pub fn execute<F>(&mut self, task_id: T, work: F)
    where
        F: FnOnce() -> R + Send + 'static,
        T: Clone + Send + 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // On native: spawn a real thread for true parallelism
            // For now, support one task at a time (sufficient for audio loading)
            // TODO: Add multi-task support with Arc<Mutex<mpsc::Sender>> in future if needed
            let (sender, receiver) = mpsc::channel();
            let task_id_clone = task_id.clone();

            thread::spawn(move || {
                let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {
                    Ok(r) => Ok(r),
                    Err(_) => Err("Task panicked".to_string()),
                };
                let _ = sender.send((task_id_clone, result));
            });

            // Replace receiver (supports one active task at a time)
            let _old_receiver = std::mem::replace(&mut self.receiver, Some(receiver));
        }

        #[cfg(target_arch = "wasm32")]
        {
            // On WASM: use spawn_local to defer work to next event loop tick
            // This doesn't block the current frame, though it still runs on main thread
            let task_id_clone = task_id.clone();
            let results = Rc::clone(&self.completed_results);

            // Wrap the synchronous work in an async block
            spawn_local(async move {
                // Execute the work function in the next event loop tick
                // Note: This still runs on main thread but defers execution, so current frame won't block
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
                let final_result = match result {
                    Ok(r) => Ok(r),
                    Err(_) => Err("Task panicked".to_string()),
                };
                results
                    .borrow_mut()
                    .push_back((task_id_clone, final_result));
            });
        }
    }

    /// Check if any tasks have completed and return their results
    /// This should be called each frame in the main update loop
    pub fn try_recv(&mut self) -> Option<(T, Result<R, String>)> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(ref receiver) = self.receiver {
                receiver.try_recv().ok()
            } else {
                None
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.completed_results.borrow_mut().pop_front()
        }
    }
}

impl<T: Send + 'static, R: Send + 'static> Default for BackgroundTask<T, R> {
    fn default() -> Self {
        Self::new()
    }
}
