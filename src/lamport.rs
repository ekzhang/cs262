//! Scale model of a Lamport clock system.
//!
//! The machines simulated have random, fixed clock rates determined during
//! initialization. They're simulated using threads and only communicate through
//! their [`Transceiver`], which uses channels.

use std::{thread, time::Duration};

use fastrand::Rng;
use tracing::{info, info_span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct MachineId(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Message {
    sender: MachineId,
    logical_time: u64,
}

struct Transceiver {
    id: MachineId,
    txs: Vec<flume::Sender<Message>>,
    rx: flume::Receiver<Message>,
}

fn machine_main(rng: Rng, tr: Transceiver) {
    let speed = rng.u32(1..=6); // ticks per second
    let mut clock = 0;

    let send = |clock: u64, i: usize| {
        let msg = Message {
            sender: tr.id,
            logical_time: clock,
        };
        info!(clock, ?msg, "sending message");
        tr.txs[i].send(msg).unwrap();
    };

    info!(speed, "starting machine");
    loop {
        let delay = Duration::from_secs_f64(1.0 / speed as f64);
        thread::sleep(delay);

        if let Ok(msg) = tr.rx.try_recv() {
            clock = clock.max(msg.logical_time) + 1;
            info!(clock, queued = tr.rx.len(), ?msg, "received message");
        } else {
            match rng.u32(1..=10) {
                1 => {
                    clock += 1;
                    send(clock, 0);
                }
                2 => {
                    clock += 1;
                    send(clock, 1);
                }
                3 => {
                    clock += 1;
                    for i in 0..tr.txs.len() {
                        send(clock, i);
                    }
                }
                _ => {
                    clock += 1;
                    info!(clock, "internal event");
                }
            }
        }
    }
}

pub fn run() {
    tracing_subscriber::fmt::init();

    let n = 3; // number of machines
    let cap = 100; // capacity of each message buffer
    let rng = Rng::with_seed(0x40);

    let trs = {
        let mut channels = Vec::new();
        channels.resize_with(n, || flume::bounded::<Message>(cap));

        Vec::from_iter((0..n).map(|i| Transceiver {
            id: MachineId(i as u32),
            txs: (1..n).map(|j| channels[(i + j) % n].0.clone()).collect(),
            rx: channels[i].1.clone(),
        }))
    };

    thread::scope(move |s| {
        for tr in trs {
            let machine_rng = Rng::with_seed(rng.u64(..));
            s.spawn(move || {
                info_span!("machine", id = ?tr.id).in_scope(|| {
                    machine_main(machine_rng, tr);
                });
            });
        }
    });
}
