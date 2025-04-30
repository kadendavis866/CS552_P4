use crate::*;

#[test]
fn test_create() {
    let queue = QueueR::<Box<i32>>::new(10);
    assert!(queue.is_empty());
}

#[test]
fn test_queue_dequeue() {
    let queue = QueueR::<Box<i32>>::new(10);
    assert!(queue.enqueue(Box::new(1)).is_ok());
    assert!(!queue.is_empty());
    let item = queue.dequeue();
    assert!(item.is_some());
    assert_eq!(*item.unwrap(), 1);
    assert!(queue.is_empty());
}

#[test]
fn test_queue_dequeue_multiple() {
    let queue = QueueR::<Box<i32>>::new(10);
    assert!(queue.enqueue(Box::new(1)).is_ok());
    assert!(queue.enqueue(Box::new(2)).is_ok());
    assert!(queue.enqueue(Box::new(3)).is_ok());
    assert!(!queue.is_empty());
    let mut item = queue.dequeue();
    assert!(item.is_some());
    assert_eq!(*item.unwrap(), 1);
    assert!(!queue.is_empty());
    item = queue.dequeue();
    assert!(item.is_some());
    assert_eq!(*item.unwrap(), 2);
    assert!(!queue.is_empty());
    item = queue.dequeue();
    assert!(item.is_some());
    assert_eq!(*item.unwrap(), 3);
    assert!(queue.is_empty());
}

#[test]
fn test_queue_dequeue_shutdown() {
    let queue = QueueR::<Box<i32>>::new(10);
    assert!(queue.enqueue(Box::new(1)).is_ok());
    assert!(queue.enqueue(Box::new(2)).is_ok());
    assert!(queue.enqueue(Box::new(3)).is_ok());
    assert!(!queue.is_empty());
    queue.shutdown();
    assert!(queue.is_shutdown());
    assert!(queue.enqueue(Box::new(4)).is_err());
    let mut item = queue.dequeue();
    assert!(item.is_some());
    assert_eq!(*item.unwrap(), 1);
    item = queue.dequeue();
    assert!(item.is_some());
    assert_eq!(*item.unwrap(), 2);
    item = queue.dequeue();
    assert!(item.is_some());
    assert_eq!(*item.unwrap(), 3);
    item = queue.dequeue();
    assert!(item.is_none());
}