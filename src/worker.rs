use crossbeam::channel::{unbounded, Sender};
use log::error;
use std::collections::BTreeMap;
use std::thread;
use tokio::runtime::Runtime;

use crate::events::{Message, ParsedEvent, SequencedEvent};
use crate::output::{process_outputs, OutputChannel};
use crate::parser::EventParser;

fn broadcast_event(
    outputs: &mut Vec<Box<dyn OutputChannel>>,
    rt: &Runtime,
    mut event: ParsedEvent,
) {
    rt.block_on(async {
        if let Err(err) = process_outputs(outputs, &mut event).await {
            error!("Error while processing output: {}", err);
        }
    });
}

pub fn spawn_work_threads(mut outputs: Vec<Box<dyn OutputChannel>>, worker_count: usize) 
    -> Sender<SequencedEvent>
{
    let (work_tx, work_rx) = unbounded::<SequencedEvent>();
    let (result_tx, result_rx) = unbounded::<(usize, Option<ParsedEvent>)>();

    // Spawn parser workers
    for _ in 0..worker_count {
        let tx = result_tx.clone();
        let rx = work_rx.clone();
        let parser = EventParser::new().expect("Failed to create event parser for worker thread");

        thread::spawn(move || {
            for msg in rx {
                let seq_id = msg.sequence_id();

                match msg.get_event() {
                    Message::Server(event) => {
                        let result = parser.parse_server_event(event);
                        tx.send((seq_id, result)).unwrap_or_else(|err| {
                            error!("Failed to send parsed event to output worker: {}", err);
                        });
                    },
                    Message::System(mut event) => {
                        let mut result = ParsedEvent::new(
                            -1, 
                            event.time, 
                            event.category
                        );

                        result.data = std::mem::take(&mut event.data);
                        tx.send((seq_id, Some(result))).unwrap_or_else(|err| {
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

        for (i, result) in &result_rx {
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