use std::{
    collections::BinaryHeap,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use tokio::sync::mpsc;

type BoxFuture<T> = Box<dyn Future<Output = T> + Send + Sync + 'static>;

pub struct TaskItem<T>(pub usize, pub T);

impl<T> PartialEq for TaskItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for TaskItem<T> {}

impl<T> PartialOrd for TaskItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl<T> Ord for TaskItem<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.cmp(&self.0)
    }
}

pub trait TaskAdapter<T, F>
where
    T: Send + Sync + Unpin + 'static,
    F: Future<Output = T> + Send + Sync + 'static,
    Self: Iterator<Item = F> + Sized,
{
    fn task(self, concurrency_limit: usize) -> Task<T> {
        Task::new(self, concurrency_limit)
    }
}

impl<I, F, T> TaskAdapter<T, F> for I
where
    I: Iterator<Item = F>,
    F: Future<Output = T> + Send + Sync + 'static,
    T: Send + Sync + Unpin + 'static,
{
}

pub struct Task<T> {
    futs: BinaryHeapPair<TaskItem<Pin<BoxFuture<Option<T>>>>>,
    rets: BinaryHeap<TaskItem<T>>,

    rx: mpsc::Receiver<()>,

    futs_len: usize,
    pos: usize,
}

/* fn raw_waker(ptr: *const ()) -> RawWaker {
    fn wake(_: *const ()) {
        // println!("wake {ptr:?}");
    }
    fn wake_by_ref(_: *const ()) {
        // println!("wake_by_ref {ptr:?}");
    }
    fn drop(_: *const ()) {
        // println!("drop {ptr:?}");
    }
    fn clone(ptr: *const ()) -> RawWaker {
        // println!("clone {ptr:?}");
        raw_waker(ptr)
    }

    let empty_raw_waker_vtable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    RawWaker::new(ptr, empty_raw_waker_vtable)
}

fn waker(ptr: *const ()) -> Waker {
    unsafe { Waker::from_raw(raw_waker(ptr)) }
} */

impl<T> Task<T>
where
    T: Send + Sync + 'static + Unpin,
{
    pub fn new<I, F>(it: I, limit: usize) -> Self
    where
        I: Iterator<Item = F>,
        F: Future<Output = T> + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::channel(limit);

        let mut this = Self {
            futs: BinaryHeapPair::new(BinaryHeap::new(), BinaryHeap::new()),
            rets: BinaryHeap::new(),
            rx,
            futs_len: 0,
            pos: 0,
        };

        for (ord, fut) in it.enumerate() {
            let tx = tx.clone();
            let f = async move {
                // produce task
                if tx.send(()).await.is_ok() {
                    Some(fut.await)
                } else {
                    None
                }
            };
            this.futs.not_empty().push(TaskItem(ord, Box::pin(f)));
            this.futs_len += 1;
        }

        this
    }
}

/// 서로 주고 받는 용도의 자료구조
///
/// refresh를 호출하기 전에 한쪽에 있는 데이터를 모두 다른 쪽으로 옮겨야함
struct BinaryHeapPair<T>(BinaryHeap<T>, BinaryHeap<T>, bool);

impl<T> BinaryHeapPair<T> {
    pub fn new(one: BinaryHeap<T>, two: BinaryHeap<T>) -> Self {
        let cond = one.is_empty();
        Self(one, two, cond)
    }
}

impl<T> BinaryHeapPair<T> {
    pub fn not_empty(&mut self) -> &mut BinaryHeap<T> {
        if self.2 {
            &mut self.1
        } else {
            &mut self.0
        }
    }

    pub fn empty(&mut self) -> &mut BinaryHeap<T> {
        if self.2 {
            &mut self.0
        } else {
            &mut self.1
        }
    }

    pub fn refresh(&mut self) {
        self.2 = self.0.is_empty();
    }
}

impl<T> Stream for Task<T>
where
    T: Send + Sync + Unpin + 'static,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.futs_len == 0 {
            return Poll::Ready(None);
        }

        if self.pos >= self.futs_len {
            return Poll::Ready(None);
        }

        let this = Pin::get_mut(self);

        this.futs.refresh();

        while let Some(TaskItem(ord, mut fut)) = this.futs.not_empty().pop() {
            match fut.as_mut().poll(cx) {
                Poll::Ready(Some(r)) => {
                    // consume task
                    let _r = this.rx.poll_recv(cx);

                    this.rets.push(TaskItem(ord, r));

                    cx.waker().wake_by_ref();
                }
                // if closed channel of (rx, tx), returns None
                Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    this.futs.empty().push(TaskItem(ord, fut));
                }
            }
        }

        if let Some(TaskItem(ord, _)) = this.rets.peek() {
            if ord == &this.pos {
                this.pos += 1;

                let r = this.rets.pop().unwrap();

                Poll::Ready(Some(r.1))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::{Duration, SystemTime},
    };

    use futures::{StreamExt, TryStreamExt};
    use rand::{thread_rng, Rng};

    use super::TaskAdapter;

    async fn sleep(ret: i32, duration: Duration) -> i32 {
        tokio::time::sleep(duration).await;
        ret
    }

    #[tokio::test]
    async fn base() {
        let mut rng = thread_rng();
        let it = (0..50).map(|x| sleep(x, Duration::from_millis(rng.gen_range(0..10))));

        let start = SystemTime::now();

        let r = it.task(15).collect::<Vec<_>>().await;

        let end = start.elapsed().unwrap().as_micros();

        println!("{}ms", end as f64 / 1000.0);

        assert_eq!(r, (0..50).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn empty() {
        let it = (0..0).map(|x| sleep(x, Duration::from_millis(0)));

        let r = it.task(5).collect::<Vec<_>>().await;

        assert!(r.is_empty());
    }

    #[tokio::test]
    async fn err() {
        async fn error(x: i32, dur: Duration, cnt: Arc<Mutex<i32>>) -> Result<i32, ()> {
            tokio::time::sleep(dur).await;

            {
                let mut cnt = cnt.lock().unwrap();
                *cnt += 1;
            }

            if x >= 10 {
                Err(())
            } else {
                Ok(x)
            }
        }

        let a_cnt = Arc::new(Mutex::new(0));
        let b_cnt = Arc::new(Mutex::new(0));

        let mut rng = thread_rng();

        let sorted_tasks =
            (0..15).map(|x| error(x, Duration::from_millis((x as u64) * 15), a_cnt.clone()));
        let rand_tasks = (0..15).map(|x| {
            error(
                x,
                Duration::from_millis(rng.gen_range(if x >= 14 { 1000..1001 } else { 0..10 })),
                b_cnt.clone(),
            )
        });

        let a = sorted_tasks.task(5).try_collect::<Vec<_>>().await;
        let b = rand_tasks.task(5).try_collect::<Vec<_>>().await;

        println!("a.strong_count {}", Arc::strong_count(&a_cnt));
        println!("b.strong_count {}", Arc::strong_count(&b_cnt));

        let a_cnt = Arc::try_unwrap(a_cnt).unwrap().into_inner().unwrap();
        let b_cnt = Arc::try_unwrap(b_cnt).unwrap().into_inner().unwrap();

        println!("a.cnt {}", a_cnt);
        println!("b.cnt {}", b_cnt);

        assert_eq!(a, Err(()));
        assert_eq!(a_cnt, 11);
        assert_eq!(b, Err(()));
        assert!(b_cnt <= 14, "b_cnt {b_cnt}");
    }
}
