# Async Patterns Reference

## Table of Contents
- [Tokio Runtime Setup](#tokio-runtime-setup)
- [Structured Concurrency](#structured-concurrency)
- [Cancellation Safety](#cancellation-safety)
- [Channel Patterns](#channel-patterns)
- [Resource Cleanup](#resource-cleanup)

## Tokio Runtime Setup

### Multi-threaded Runtime (Default)
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Uses all available cores
    run_server().await
}
```

### Current-thread Runtime (Single-threaded)
```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // Lower overhead, no Send bounds required
    run_server().await
}
```

### Custom Runtime Configuration
```rust
let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .thread_name("worker")
    .enable_all()
    .build()?;

runtime.block_on(async { /* ... */ });
```

## Structured Concurrency

### JoinSet for Task Groups
```rust
use tokio::task::JoinSet;

async fn process_batch(items: Vec<Item>) -> Vec<Result<Output, Error>> {
    let mut set = JoinSet::new();

    for item in items {
        set.spawn(async move { process(item).await });
    }

    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result.expect("task panicked"));
    }
    results
}
```

### Abort on First Error
```rust
async fn all_or_nothing(tasks: Vec<impl Future<Output = Result<T, E>>>) -> Result<Vec<T>, E> {
    let mut set = JoinSet::new();
    for task in tasks {
        set.spawn(task);
    }

    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        match result.expect("panicked") {
            Ok(value) => results.push(value),
            Err(e) => {
                set.abort_all();  // Cancel remaining tasks
                return Err(e);
            }
        }
    }
    Ok(results)
}
```

## Cancellation Safety

### Cancel-Safe Operations
```rust
// GOOD: Idempotent or atomic operations
async fn cancel_safe_read(reader: &mut TcpStream) -> io::Result<Bytes> {
    let mut buf = BytesMut::with_capacity(1024);
    reader.read_buf(&mut buf).await?;  // Cancel-safe
    Ok(buf.freeze())
}

// BAD: Partial state modification
async fn cancel_unsafe(conn: &mut Connection) -> Result<()> {
    conn.send_header().await?;   // If cancelled here...
    conn.send_body().await?;     // ...body never sent, protocol broken
    Ok(())
}
```

### Making Operations Cancel-Safe
```rust
// Wrap in a transaction-like pattern
async fn safe_transfer(from: &Account, to: &Account, amount: u64) -> Result<()> {
    // Use a lock or transaction to ensure atomicity
    let _guard = TRANSFER_LOCK.lock().await;

    // Even if cancelled, the lock ensures no partial state
    from.withdraw(amount).await?;
    to.deposit(amount).await?;
    Ok(())
}
```

## Channel Patterns

### MPSC for Work Distribution
```rust
use tokio::sync::mpsc;

async fn worker_pool(rx: &mut mpsc::Receiver<Job>) {
    while let Some(job) = rx.recv().await {
        if let Err(e) = process_job(job).await {
            tracing::error!(?e, "job failed");
        }
    }
}

// Producer
let (tx, mut rx) = mpsc::channel(100);
for _ in 0..num_workers {
    let mut rx = rx.clone();
    tokio::spawn(async move { worker_pool(&mut rx).await });
}
```

### Watch for Configuration Updates
```rust
use tokio::sync::watch;

struct ConfigManager {
    tx: watch::Sender<Config>,
}

impl ConfigManager {
    fn subscribe(&self) -> watch::Receiver<Config> {
        self.tx.subscribe()
    }

    fn update(&self, config: Config) {
        self.tx.send_replace(config);
    }
}

// Consumer
async fn config_aware_task(mut config_rx: watch::Receiver<Config>) {
    loop {
        let config = config_rx.borrow_and_update().clone();
        // Use config...

        config_rx.changed().await.ok();  // Wait for updates
    }
}
```

### Oneshot for Request-Response
```rust
use tokio::sync::oneshot;

struct Request {
    data: Vec<u8>,
    response_tx: oneshot::Sender<Response>,
}

async fn handle_request(req: Request) {
    let response = process(&req.data).await;
    let _ = req.response_tx.send(response);  // Receiver may be dropped
}
```

## Resource Cleanup

### Graceful Shutdown Pattern
```rust
use tokio::signal;
use tokio::sync::broadcast;

async fn run_with_shutdown() -> anyhow::Result<()> {
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    // Spawn workers with shutdown receivers
    for _ in 0..num_workers {
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(job) = jobs.recv() => process(job).await,
                    _ = shutdown_rx.recv() => break,
                }
            }
            cleanup().await;  // Graceful cleanup
        });
    }

    // Wait for ctrl-c
    signal::ctrl_c().await?;
    drop(shutdown_tx);  // Signal all workers

    // Wait for workers to finish (with timeout)
    tokio::time::timeout(Duration::from_secs(30), async {
        while let Some(result) = workers.join_next().await {
            result?;
        }
        Ok::<_, anyhow::Error>(())
    }).await??;

    Ok(())
}
```

### Drop Guard for Async Cleanup
```rust
struct AsyncDropGuard {
    cleanup_tx: Option<oneshot::Sender<()>>,
}

impl AsyncDropGuard {
    fn new() -> (Self, oneshot::Receiver<()>) {
        let (tx, rx) = oneshot::channel();
        (Self { cleanup_tx: Some(tx) }, rx)
    }
}

impl Drop for AsyncDropGuard {
    fn drop(&mut self) {
        if let Some(tx) = self.cleanup_tx.take() {
            let _ = tx.send(());  // Signal cleanup needed
        }
    }
}
```
