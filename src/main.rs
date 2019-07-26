#![feature(async_await)]

use tokio;
use tokio::sync::lock::{Lock, LockGuard};

use core::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

struct Items {
    items: Vec<Lock<usize>>,
}

impl Items {
    fn new() -> Self {
        let mut items = vec![];

        for i in 0..5 {
            items.push(Lock::new(i));
        }

        Self { items }
    }
    fn get<'a>(&'a self) -> LockFuture<'a> {
        LockFuture(&self)
    }
}

struct LockFuture<'a>(&'a Items);

impl<'a> Future for LockFuture<'a> {
    type Output = LockGuard<usize>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        for mut item in self.0.items.iter().cloned() {
            if let Poll::Ready(val) = item.poll_lock(cx) {
                return Poll::Ready(val);
            }
        }

        Poll::Pending
    }
}

#[tokio::main]
async fn main() {
    let items = Arc::new(Items::new());

    let mut completed = Lock::new(0usize);

    for i in 0..1000 {
        let items = items.clone();
        let completed = completed.clone();

        rt.spawn(async move {
            let guard = items.get().await;

            println!("{:?}, {}", i, *guard);

            let mut completed = completed.clone();
            let mut completed = completed.lock().await;
            *completed += 1;
        });
    }

    while *completed.lock().await != 1000 {
        // do nothing
    }
}
