/// A simple producer/consumer program that uses a bounded queue.

#[cfg(test)]
mod test;

mod queue;

use crate::queue::*;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

/*
 The ID returned by std::thread::Thread.id() is not the same as the OS thread ID, also to convert
 the returned value to an int requires a nightly build of Rust. gettid is a crate which provides a
 cross-platform way to get the OS thread ID as an integer (using libc on Unix-like systems).
*/
use getopt;
use gettid::gettid;
use rand::Rng;

const MAX_C: u32 = 8; // maximum number of consumer threads
const MAX_P: u32 = 8; // maximum number of producer threads
const MAX_SLEEP: u64 = 1000000; // maximum time a thread can sleep in nanoseconds

/// Produce items at a random interval. Exit once it has produced the correct number of items or the
/// queue is shutdown.
///
/// # Arguments
/// * queue - The queue to add items to
/// * num_items - The number of items to produce
///
/// # Returns
/// the number of items successfully produced
fn producer(queue: &QueueR<Box<i32>>, num_items: u32, delay: bool) -> u32 {
    let tid = gettid();
    let mut rng = rand::rng();
    let mut num_produced = 0;
    for i in 0..num_items {
        if delay {
            let sleep_time = rng.random::<u64>() % MAX_SLEEP;
            std::thread::sleep(std::time::Duration::from_nanos(sleep_time));
        }
        if queue.enqueue(Box::new(i as _)).is_err() {
            eprintln!("ERROR: producer thread: {} failed to enqueue item", tid);
            break;
        }
        num_produced += 1;
    }
    println!("Producer thread: {} - Done producing!", tid);
    num_produced
}

/// Consume items from the queue at a random interval. Exit once the queue is empty and shutdown.
///
/// # Arguments
/// * queue - The queue to consume items from
fn consumer(queue: &QueueR<Box<i32>>, delay: bool) -> u32 {
    let tid = gettid();
    let mut rng = rand::rng();
    let mut num_consumed = 0;
    loop {
        if delay {
            let sleep_time = rng.random::<u64>() % MAX_SLEEP;
            std::thread::sleep(std::time::Duration::from_nanos(sleep_time));
        }
        match queue.dequeue() {
            Some(_) => num_consumed += 1,
            None => {
                if !queue.is_shutdown() {
                    eprintln!(
                        "ERROR: producer thread: {} got a null item when queue was not shutdown!",
                        tid
                    );
                }
                break;
            }
        }
    }
    println!("Consumer thread: {} - Done consuming!", tid);
    num_consumed
}

/// Print the usage message
///
/// # Arguments
/// * file_name - The name of the executable
fn usage(file_name: &str) {
    println!(
        "Usage: {} [-dh] [-c num_consumer] [-p num_producer] [-i num_items] [-s queue_size]",
        file_name
    );
    println!("  -d: introduce a random delay between consumer and producer");
    println!("  -h: print this help message");
}

/// The main function of the command line tool. It creates a number of producer and consumer threads
/// and runs them.
fn main() {
    let mut num_producers = 1u32;
    let mut num_consumers = 1u32;
    let mut num_items = 10u32;
    let mut queue_size = 5usize;
    let mut delay = false;

    // parse command line arguments
    let args = std::env::args().collect::<Vec<String>>();
    let mut opts = getopt::Parser::new(&args[0..], "dhc:p:i:s:");
    loop {
        match opts.next() {
            None => break,
            Some(Ok(opt)) => match opt {
                getopt::Opt('d', None) => {
                    delay = true;
                }
                getopt::Opt('h', None) => {
                    usage(&args[0]);
                    return;
                }
                getopt::Opt('c', Some(arg)) => {
                    num_consumers = arg
                        .parse::<u32>()
                        .or_else(|_| -> Result<u32, &str> {
                            eprintln!("ERROR: invalid number of consumers");
                            usage(&args[0]);
                            std::process::exit(1);
                        })
                        .unwrap();
                }
                getopt::Opt('p', Some(arg)) => {
                    num_producers = arg
                        .parse::<u32>()
                        .or_else(|_| -> Result<u32, &str> {
                            eprintln!("ERROR: invalid number of producers");
                            usage(&args[0]);
                            std::process::exit(1);
                        })
                        .unwrap();
                }
                getopt::Opt('i', Some(arg)) => {
                    num_items = arg
                        .parse::<u32>()
                        .or_else(|_| -> Result<u32, &str> {
                            eprintln!("ERROR: invalid number of items");
                            usage(&args[0]);
                            std::process::exit(1);
                        })
                        .unwrap();
                }
                getopt::Opt('s', Some(arg)) => {
                    queue_size = arg
                        .parse::<usize>()
                        .or_else(|_| -> Result<usize, &str> {
                            eprintln!("ERROR: invalid queue size");
                            usage(&args[0]);
                            std::process::exit(1);
                        })
                        .unwrap();
                }
                _ => unreachable!(),
            },
            Some(Err(_)) => {
                eprintln!("ERROR: invalid argument");
                usage(&args[0]);
                std::process::exit(1);
            }
        }
    }
    if num_consumers > MAX_C {
        num_consumers = MAX_C;
    }
    if num_producers > MAX_P {
        num_producers = MAX_P;
    }

    let per_thread = num_items / num_producers;
    println!(
        "Simulating {} producers {} consumers with {} items per thread and a queue size of {}",
        num_producers, num_consumers, per_thread, queue_size
    );

    let num_produced = Arc::new(Mutex::new(0u32));
    let num_consumed = Arc::new(Mutex::new(0u32));
    let queue = QueueR::<Box<i32>>::new(queue_size);

    let start = Instant::now();

    let mut producer_threads: Vec<JoinHandle<()>> = vec![];
    for _ in 0..num_producers {
        let queue = queue.clone();
        let num_produced = num_produced.clone();
        producer_threads.push(std::thread::spawn(move || {
            let n = producer(queue.as_ref(), per_thread, delay);
            *num_produced.lock().unwrap() += n;
        }));
    }
    println!("Started {} producer threads", num_producers);

    let mut consumer_threads: Vec<JoinHandle<()>> = vec![];
    for _ in 0..num_consumers {
        let queue = queue.clone();
        let num_consumed = num_consumed.clone();
        consumer_threads.push(std::thread::spawn(move || {
            let n = consumer(queue.as_ref(), delay);
            *num_consumed.lock().unwrap() += n;
        }));
    }
    println!("Started {} consumer threads", num_consumers);

    for thread in producer_threads {
        thread.join().expect("Failed to join producer threads");
    }

    // Once all the producers are finished we set a flag so the consumer thread can finish up. Once
    // shutdown is called the queue should drain all remaining items and be read for destruction!
    queue.shutdown();

    for thread in consumer_threads {
        thread.join().expect("Failed to join consumer threads");
    }

    let produced = num_produced.lock().unwrap().clone();
    let consumed = num_consumed.lock().unwrap().clone();
    if produced != consumed {
        eprintln!(
            "ERROR: produced {} items but consumed {} items",
            produced, consumed
        );
    }

    let elapsed = start.elapsed();
    println!(" {} {}", elapsed.as_millis(), produced);
}
