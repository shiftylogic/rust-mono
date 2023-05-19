/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Web playground / demo
 *
 */

// sl_core::enable_global_tracing_alloc!();
// sl_core::enable_global_counting_alloc!();

fn main() {
    env_logger::init();

    let nums = make_nums();

    bench_baseline(&nums);
    bench_xorer::<ChannelAsyncXor>("Async:\t\t\t", &nums);
    bench_xorer::<ChannelSyncXor<32768>>(format!("Sync (32768)):\t\t").as_str(), &nums);
    bench_xorer::<MultiXor<ChannelAsyncXor, 2>>(format!("Multi Async (2):\t").as_str(), &nums);
    bench_xorer::<MultiXor<ChannelAsyncXor, 4>>(format!("Multi Async (4):\t").as_str(), &nums);
    bench_xorer::<MultiXor<ChannelSyncXor<32768>, 2>>(
        format!("Multi Sync (32768, 2):\t").as_str(),
        &nums,
    );
    bench_xorer::<MultiXor<ChannelSyncXor<32768>, 4>>(
        format!("Multi Sync (32768, 4):\t").as_str(),
        &nums,
    );
}

fn bench_baseline(nums: &[u64; 2048]) {
    let mut res = 0;

    let now = std::time::Instant::now();
    for _ in 0..100_001 {
        for n in nums {
            res = res ^ *n;
        }
    }

    let elapsed = now.elapsed();
    println!("Baseline:\t\t{res} in {:?}", elapsed);
}

fn bench_xorer<T: Xorer + Default>(tag: &str, nums: &[u64; 2048]) {
    let mut xorer = T::default();

    let now = std::time::Instant::now();
    for _ in 0..100_001 {
        for n in nums {
            xorer.xor(*n);
        }
    }

    let elapsed = now.elapsed();
    println!("{tag}{} in {:?}", xorer.finalize(), elapsed);
}

fn make_nums() -> [u64; 2048] {
    let rng = fastrand::Rng::with_seed(8374);
    let nums: [u64; 2048] = [0; 2048];
    nums.map(|_| rng.u64(..))
}


trait Xorer {
    fn xor(&mut self, v: u64);
    fn finalize(&mut self) -> u64;
}


//
// Baseline XOR
//


//
// Channel pump (async)
//
struct ChannelAsyncXor {
    joiner: Option<std::thread::JoinHandle<u64>>,
    tx:     Option<std::sync::mpsc::Sender<u64>>,
}

impl Default for ChannelAsyncXor {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let joiner = std::thread::spawn(move || -> u64 {
            let mut res = 0;
            while let Ok(v) = rx.recv() {
                res = res ^ v;
            }
            res
        });

        Self {
            joiner: Some(joiner),
            tx:     Some(tx),
        }
    }
}

impl Xorer for ChannelAsyncXor {
    fn xor(&mut self, v: u64) {
        if let Some(tx) = self.tx.as_mut() {
            tx.send(v).expect("send failed on channel");
        }
    }

    fn finalize(&mut self) -> u64 {
        if let Some(tx) = self.tx.take() {
            drop(tx);
        }

        if let Some(joiner) = self.joiner.take() {
            return joiner.join().expect("failed to join channel thread");
        }

        panic!("finalizing a second time");
    }
}


//
// Channel pump (sync)
//
struct ChannelSyncXor<const N: usize> {
    joiner: Option<std::thread::JoinHandle<u64>>,
    tx:     Option<std::sync::mpsc::SyncSender<u64>>,
}

impl<const N: usize> Default for ChannelSyncXor<N> {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel(N);

        let joiner = std::thread::spawn(move || -> u64 {
            let mut res = 0;
            while let Ok(v) = rx.recv() {
                res = res ^ v;
            }
            res
        });

        Self {
            joiner: Some(joiner),
            tx:     Some(tx),
        }
    }
}

impl<const N: usize> Xorer for ChannelSyncXor<N> {
    #[inline]
    fn xor(&mut self, v: u64) {
        if let Some(tx) = self.tx.as_mut() {
            tx.send(v).expect("send failed on channel");
        }
    }

    fn finalize(&mut self) -> u64 {
        if let Some(tx) = self.tx.take() {
            drop(tx);
        }

        if let Some(joiner) = self.joiner.take() {
            return joiner.join().expect("failed to join channel thread");
        }

        panic!("finalizing a second time");
    }
}


//
// Multi-threaded wrapper
//
struct MultiXor<T: Xorer + Default, const N: usize> {
    xorers: [T; N],
    index:  usize,
}

impl<T: Xorer + Default, const N: usize> Default for MultiXor<T, N> {
    fn default() -> Self {
        Self {
            xorers: [0; N].map(|_| T::default()),
            index:  0,
        }
    }
}

impl<T: Xorer + Default, const N: usize> Xorer for MultiXor<T, N> {
    #[inline]
    fn xor(&mut self, v: u64) {
        self.xorers[self.index].xor(v);
        self.index = (self.index + 1) % N;
    }

    fn finalize(&mut self) -> u64 {
        let mut res = 0;
        for xorer in &mut self.xorers {
            res = res ^ xorer.finalize();
        }
        res
    }
}
