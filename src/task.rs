use std::future::Future;

pub async fn task<T>(
    futs: impl IntoIterator<Item = impl Future<Output = T> + Send + Sync + 'static>,
    concurreny_limit: usize,
) -> Vec<T>
where
    T: Send + Sync + 'static,
{
    task_inner(futs, |r| Ok::<T, ()>(r), concurreny_limit)
        .await
        .unwrap()
        .collect()
}

pub async fn result_task<T, E>(
    futs: impl IntoIterator<Item = impl Future<Output = Result<T, E>> + Send + Sync + 'static>,
    concurreny_limit: usize,
) -> Result<Vec<T>, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    task_inner(futs, |r| r.map(Ok), concurreny_limit)
        .await
        .map(|xs| xs.map(|x| x.map_err(|_| ()).unwrap()).collect())
}

pub async fn option_task<T>(
    futs: impl IntoIterator<Item = impl Future<Output = Option<T>> + Send + Sync + 'static>,
    concurreny_limit: usize,
) -> Option<Vec<T>>
where
    T: Send + Sync + 'static,
{
    task_inner(futs, |r| r.ok_or(()).map(Some), concurreny_limit)
        .await
        .map(|xs| xs.map(|x| x.unwrap()).collect())
        .ok()
}

async fn task_inner<T, E, F>(
    futs: impl IntoIterator<Item = impl Future<Output = T> + Send + Sync + 'static>,
    f: F,
    concurreny_limit: usize,
) -> Result<impl Iterator<Item = T>, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + 'static,
    F: Fn(T) -> Result<T, E>,
    F: Send + Sync + 'static,
{
    let (task_producer, mut task_consumer) = tokio::sync::mpsc::channel(concurreny_limit);
    let (tx, mut rx) = tokio::sync::mpsc::channel(5);
    let mut threads = Vec::new();
    let mut res = Vec::new();

    let mut task_len = 0;
    let mut received_len = 0;

    for (ord, fut) in futs.into_iter().enumerate() {
        let task_producer = task_producer.clone();
        let tx = tx.clone();
        // maybe not error returned
        let thread = tokio::spawn(async move {
            /* if task_producer.send(()).await.is_err() {
                return false;
            } */
            task_producer.send(()).await.unwrap();
            let r: T = fut.await;
            tx.send((ord, r)).await.map_err(|_| "SendError(T)").unwrap();
        });
        threads.push(thread);
        task_len += 1;
    }

    if task_len > 0 {
        while let Some((ord, r)) = rx.recv().await {
            received_len += 1;

            task_consumer.recv().await;

            match f(r) {
                Ok(r) => res.push((ord, r)),
                Err(err) => {
                    for thread in threads {
                        thread.abort();

                        // consume thread
                        thread.await.ok();
                    }
                    return Err(err);
                }
            }

            if received_len >= task_len {
                break;
            }
        }

        for thread in threads {
            // consume thread
            let _ok = thread.await.unwrap();
        }

        res.sort_by(|a, b| a.0.cmp(&b.0));
    }

    Ok(res.into_iter().map(|(_, r)| r))
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::{Duration, SystemTime},
    };

    use rand::{thread_rng, Rng};

    use super::{result_task, task};

    async fn sleep(ret: i32, duration: Duration) -> i32 {
        tokio::time::sleep(duration).await;
        ret
    }

    #[tokio::test]
    async fn base() {
        let mut rng = thread_rng();
        let tasks = (0..50).map(|x| sleep(x, Duration::from_millis(rng.gen_range(0..10))));

        let start = SystemTime::now();

        let r = task(tasks, 25).await;

        let end = start.elapsed().unwrap().as_micros();

        println!("{}ms", end as f64 / 1000.0);

        assert_eq!(r, (0..50).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn empty() {
        let tasks = (0..0).map(|x| sleep(x, Duration::from_millis(0)));

        let r = task(tasks, 5).await;

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

        let cnt = Arc::new(Mutex::new(0));
        let tasks = (0..15).map(|x| error(x, Duration::from_millis((x as u64) * 10), cnt.clone()));

        let r = result_task(tasks, 5).await;

        let cnt = Arc::try_unwrap(cnt).unwrap().into_inner().unwrap();

        assert_eq!(r, Err(()));
        assert_eq!(cnt, 11);
    }
}
