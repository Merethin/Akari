use crossbeam::channel::{unbounded, Sender};
use log::error;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;

use crate::events::{Event, Message};
use crate::output::{process_outputs, OutputChannels};
use crate::patterns::generate_happenings;
use crate::parser::handle_server_message;

fn broadcast_event(
    outputs: &mut OutputChannels,
    rt: &Runtime,
    mut event: Event,
) {
    rt.block_on(async {
        if let Err(err) = process_outputs(outputs, &mut event).await {
            error!("Error while processing output: {}", err);
        }
    });
}

pub fn spawn_work_threads(mut outputs: OutputChannels, worker_count: usize) 
    -> Sender<(usize, Message)>
{
    let (work_tx, work_rx) = unbounded::<(usize, Message)>();
    let (result_tx, result_rx) = unbounded::<(usize, Option<Event>)>();

    let happenings = Arc::new(generate_happenings().expect("Failed to generate happening list"));

    // Spawn parser workers
    for _ in 0..worker_count {
        let tx = result_tx.clone();
        let rx = work_rx.clone();
        let h = happenings.clone();

        thread::spawn(move || {
            for msg in rx {
                match msg.1 {
                    Message::Server(event) => {
                        let result = handle_server_message(event, &h);
                        tx.send((msg.0, result)).unwrap_or_else(|err| {
                            error!("Failed to send parsed event to output worker: {}", err);
                        });
                    },
                    Message::System(mut event) => {
                        let mut result = Event::new(
                            -1, 
                            event.time, 
                            event.line.as_str(), 
                            event.category
                        );

                        result.data = std::mem::take(&mut event.data);
                        tx.send((msg.0, Some(result))).unwrap_or_else(|err| {
                            error!("Failed to send parsed event to output worker: {}", err);
                        });
                    }
                }
            }
        });
    }

    // Spawn output worker
    thread::spawn(move || {
        let mut next_sequence_id = 0;
        let mut buffer = BTreeMap::new();
        let rt = Runtime::new().expect("Failed to initialize Tokio runtime for output worker thread");

        for (i, result) in result_rx.iter() {
            buffer.insert(i, result);

            while let Some(maybe_event) = buffer.remove(&next_sequence_id) {
                if let Some(event) = maybe_event {
                    broadcast_event(&mut outputs, &rt, event);
                }

                next_sequence_id += 1;
            }
        }
    });

    work_tx
}