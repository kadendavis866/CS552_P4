# CS 552 Operating Systems Project 4

This project contains a Rust implementation of a thread-safe bounded queue. If the queue is empty, dequeue() will block until either an enqueue operation is performed or the queue is shutdown. Similarly, enqueue() will block if there is no room in the buffer.

The queue is implemented using a circular buffer Vec. This allows for push and pop operations to be performed without shifting elements as well as fast length checking.

## Building

```bash
make
```

## Testing

```bash
make check
```

## Clean

```bash
make clean
```

## Install Dependencies

If needed, the rust build system (rustup and cargo) can be installed/updated by running the following command:

```bash
sudo make install-deps
```