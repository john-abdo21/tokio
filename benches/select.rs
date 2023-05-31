//! Benchmark implementation details of the `select!` macro.
//!
//! This is to help us improve perf for this macro.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::Duration;
use tokio::runtime::{self, Runtime};

use bencher::{benchmark_group, benchmark_main, Bencher};

const NUM_SELECT: usize = 10;
const NUM_POLLS: u32 = 10;

static mut RENDEZVOUS: u32 = 0;
static mut WAKER: Option<Waker> = None;

fn select_some(b: &mut Bencher) {
    let rt = rt();

    rt.spawn(async {
        loop {
            unsafe {
                if let Some(waker) = WAKER.take() {
                    waker.wake();
                }
            }
            tokio::task::yield_now().await;
        }
    });

    b.iter(|| {
        rt.block_on(async {
            for _ in 0..NUM_SELECT {
                unsafe { RENDEZVOUS = 0 };
                tokio::select! {
                    x = XWakeFuture { id: 0 } => { bencher::black_box(x); }
                    x = XWakeFuture { id: 1 } => { bencher::black_box(x); }
                }
            }
        });
    });
}

fn select_many(b: &mut Bencher) {
    let rt = rt();

    rt.spawn(async {
        loop {
            unsafe {
                if let Some(waker) = WAKER.take() {
                    waker.wake();
                }
            }
            tokio::task::yield_now().await;
        }
    });

    b.iter(|| {
        rt.block_on(async {
            for _ in 0..NUM_SELECT {
                unsafe { RENDEZVOUS = 0 };

                tokio::select! {
                    x = XWakeFuture { id: 0 } => { bencher::black_box(x); }
                    x = XWakeFuture { id: 1 } => { bencher::black_box(x); }
                    x = XWakeFuture { id: 2 } => { bencher::black_box(x); }
                    x = XWakeFuture { id: 3 } => { bencher::black_box(x); }
                    x = XWakeFuture { id: 4 } => { bencher::black_box(x); }
                    x = XWakeFuture { id: 5 } => { bencher::black_box(x); }
                    x = XWakeFuture { id: 6 } => { bencher::black_box(x); }
                    x = XWakeFuture { id: 7 } => { bencher::black_box(x); }
                }
            }
        });
    });
}

struct XWakeFuture {
    id: u32,
}

impl Future for XWakeFuture {
    type Output = (u32, u32);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            std::thread::sleep(Duration::from_micros(50));

            if self.id == 0 {
                if RENDEZVOUS > NUM_POLLS {
                    return Poll::Ready((RENDEZVOUS, self.id));
                } else {
                    RENDEZVOUS += 1;
                }
            }

            WAKER = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

fn rt() -> Runtime {
    runtime::Builder::new_current_thread().build().unwrap()
}

benchmark_group!(scheduler, select_some, select_many);

benchmark_main!(scheduler);
