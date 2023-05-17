/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Web playground / demo
 *
 */

// sl_core::enable_global_tracing_alloc!();
sl_core::enable_global_counting_alloc!();

fn main() {
    env_logger::init();

    sl_core::trace_block! {
        "main";

        let mut rt = Executor::new(1, 10);
        rt.spawn(1);
        rt.spawn(8);
        rt.spawn(9);
        rt.spawn(42);
        rt.wait();
    }
}


//
// What is a "task"
//

enum Task {
    Stop,
    Num(usize),
}


//
// Exewcutor implementation
//

use std::{
    sync::mpsc::{
        sync_channel,
        SyncSender,
    },
    thread::JoinHandle,
};

struct Executor {
    id:      u32,
    joiner:  Option<JoinHandle<()>>,
    spawner: SyncSender<Task>,
}

impl Executor {
    pub fn new(id: u32, max_queued_tasks: usize) -> Self {
        let (spawner, task_queue) = sync_channel::<Task>(max_queued_tasks);

        let joiner = std::thread::spawn(move || {
            while let Ok(task) = task_queue.recv() {
                match task {
                    | Task::Stop => break,
                    | Task::Num(v) => println!("[Thread {id}] Task {v} complete."),
                }
            }
        });

        Self {
            id,
            joiner: Some(joiner),
            spawner,
        }
    }

    pub fn spawn(&mut self, id: usize) {
        self.spawner
            .send(Task::Num(id))
            .expect("failed to send task to thread");
    }

    pub fn wait(&mut self) {
        self.spawner
            .send(Task::Stop)
            .expect("failed to send stop signal to thread");

        match self.joiner.take() {
            | None => log::warn!("[Thread {}] Wait skipped. Thread is gone.", self.id),
            | Some(joiner) => {
                if let Err(panic) = joiner.join() {
                    log::error!("[Thread {}] Error attempting to join: {:?}", self.id, panic);
                }
            },
        }
    }
}
