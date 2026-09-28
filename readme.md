# Crab Blog


## Motivation

I've always wanted to have a blog, and to make it. But most blog development is boring... Add some JS, some React, some php/python and boom you got a blog.

Thats too easy. So I set out to do something more difficult.


## Features

This is an HTTP server, I wrote all the HTTP parsing code myself, from the raw TCP connection.

The website is mostly server side rendered, I tried to avoid JS and do all of the computations in rust.

That being said some things like the show password button for the auth does use JS.


## My journey to optimise it (Updated to 28.09.2026)

As of today, I have managed to improve the throughput to 14,200 requests per second on a post with 24 comments from multiple users.

You can check benchmark/speed.txt for all the logs from my testing script.


## Summary
 
| Setup (release, `/post/2`) | Time for 10k requests | Throughput     |
|----------------------------|-----------------------|----------------|
| Thread per connection      | 1.00 s                | ~10,000 req/s  |
| Thread pool, 2 workers     | 0.71 s                | ~14,200 req/s  |



Replacing thread-per-connection with a thread pool gave **~42% more throughput**.

The best worker count on this machine turned out to be **2, not 24** — see [Why two workers is fastest](#why-two-workers-is-fastest).

At least while running Apache Bench (ab command in Linux)

I will have to investigate more with using a multi threaded testing tool like wrk.


## Setup
- **Machine:** 12 Core, 24-thread CPU, AMD Ryzen 9 3900X
- **Load:** 10,000 requests, 1,000 concurrent connections; each measurement is an average of 10 runs
- **Routes:** `/` and `/post/2` (a post with 24 comments, no image), not logged in
- **Caveat:** the benchmark client runs on the same machine as the server and competes with it for CPU, so absolute numbers are conservative
- **Raw logs:** `benchmarks/speed.txt`




## 1. Before — thread per connection (Until 27.09.2026)
 
### Debug vs release
 
| Route     | Build   | Time (10k requests) |
|-----------|---------|---------------------|
| `/` with debug prints    | Debug   | 0.83 – 0.85 s |
| `/` without debug prints | Debug   | 0.82 s        |
| `/post/2`                | Debug   | 1.99 s        |
| `/post/2`                | Release | 1.00 s        |


Removing the debug prints helped slightly (~2–3%). Building in release mode halved the time for `/post/2`.
Debug builds skip optimisations and also distort *where* time is spent, so all comparisons from here on use release builds.



### Profiling
 
![Flamegraph before the thread pool](/docs/assets/flamegraph_27_09.svg)
 
Profiled with `cargo flamegraph` (perf, frame pointers enabled). The widest stacks were thread creation:


- the main thread creating a thread per connection: `thread::spawn` → `pthread_create` → `clone3` — **44.3%** of samples

- each new thread setting itself up before running: `make_handler` → `get_stack` → `mmap` — **15.9%**

- actual request handling, `handle_connection`: **17.4%**

So we are spending 60.2% of the time creating and setting up threads.

**Decision:** replace thread-per-connection with a thread pool.



## 2. Thread pool (From 28.09.2026 - 1AM Onwards)
 
A fixed number of worker threads is started once, at startup. The accept loop sends each incoming `TcpStream` into a single `mpsc` channel. 

The channel's receiver is shared by all workers through `Arc<Mutex<Receiver<TcpStream>>>`. 

Thanks to the .recv call, workers sit at idle when there is no work to do. 

So all the worker does is wait, lock the receiver, takes the next connection, releases the lock, and handles the request. 

Shared state (`db`, `attempts`) is handed to each worker once, when it is spawned. There are also protected by a mutex.
 
### Tuning the worker count (release, `/post/2`)
 
| Workers | Avg time (10k requests) | Throughput     |
|---------|-------------------------|----------------|
| 1       | 0.93 s                  | ~10,700 req/s  |
| **2**   | **0.71 s**              | **~14,200 req/s** |
| 4       | 0.77 s                  | ~13,000 req/s  |
| 8       | 0.84 s                  | ~12,000 req/s  |
| 12      | 0.84 s                  | ~12,000 req/s  |
| 16      | 0.84 s                  | ~11,900 req/s  |
| 20      | 0.86 s                  | ~11,600 req/s  |
| 30      | 0.87 s                  | ~11,500 req/s  |

Even a single pooled worker (0.93 s) beats thread-per-connection (1.00 s), since no thread is created per request.

This is because the server simply doesn't need so many threads, so it gets slowed down by the overhead of creating the threads, and also the kernel scheduling overhead.

### Why two workers is fastest
 
**My Hypothesis:** every worker shares one SQLite connection behind a `Mutex`, so part of every request can only run on one thread at a time.

- With one worker, querying and rendering happen one after the other.
- With **two**, one worker can query while the other renders and writes its response, so the lock is kept busy with little waiting.
- With more than that, it seems like the threads wait too much for the connection to free up, add the kernel overhead on top of that and you get increasingly slower.

**To confirm:** I can run the same worker sweep against a route that never touches the database (e.g. `/style.css`), and measure the time spent waiting to acquire the database lock. If the database-free route keeps scaling with more workers, the lock is the ceiling.
 
**About the flamegraph:** it only records time spent running on CPU. A thread waiting on a lock isn't running, so it produces no samples. That's probably why the profile predicted a bigger win than the pool delivered.

![Flamegraph after the thread pool](/docs/assets/post_thread_pool_27.09.2026.svg)


## Next steps
- Confirm the lock hypothesis, and implement a pool for the locks.
- Test with a multi threaded testing tool
- Re-measure after each change, in release mode
- Deploy the blog and stop obsessing over performance haha :))






