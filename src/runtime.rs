use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

// ==== タスク ====
pub struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    waker: Option<Waker>,
}

impl Task {
    pub fn new(fut: impl Future<Output = ()> + Send + 'static) -> Arc<Self> {
        Arc::new(Self {
            future: Mutex::new(Box::pin(fut)),
            waker: None,
        })
    }

    fn poll(self: &Arc<Self>) {
        let waker = waker(self.clone());
        let mut ctx = Context::from_waker(&waker);

        let mut fut = self.future.lock().unwrap();
        if let Poll::Pending = fut.as_mut().poll(&mut ctx) {
            // pending ならなにもしない
        }
    }
}

// ==== Executor ====
pub struct Executor {
    queue: Mutex<VecDeque<Arc<Task>>>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub fn spawn(&self, task: Arc<Task>) {
        self.queue.lock().unwrap().push_back(task);
    }

    pub fn run(&self) {
        while let Some(task) = self.queue.lock().unwrap().pop_front() {
            task.poll();
        }
    }
}

// ==== Waker ====
fn waker(task: Arc<Task>) -> Waker {
    unsafe fn clone_raw(data: *const ()) -> RawWaker {
        let arc = Arc::<Task>::from_raw(data as *const Task);
        std::mem::forget(arc.clone());
        RawWaker::new(data, &VTABLE)
    }
    unsafe fn wake_raw(data: *const ()) {
        let task = Arc::<Task>::from_raw(data as *const Task);
        // 再スケジュールしたい場合はここで Executor に入れる
        std::mem::forget(task);
    }
    unsafe fn wake_by_ref_raw(data: *const ()) {
        wake_raw(data)
    }
    unsafe fn drop_raw(_data: *const ()) {}

    static VTABLE: RawWakerVTable =
        RawWakerVTable::new(clone_raw, wake_raw, wake_by_ref_raw, drop_raw);

    let raw = RawWaker::new(Arc::into_raw(task) as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw) }
}

// ==== 使用例 ====
#[tokio::main]
async fn main() {
    let ex = Executor::new();

    let t1 = Task::new(async {
        println!("task1 start");
        println!("task1 end");
    });

    let t2 = Task::new(async {
        println!("task2 start");
        println!("task2 end");
    });

    ex.spawn(t1);
    ex.spawn(t2);

    ex.run();
}
