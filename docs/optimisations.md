# Request time optimisation

## 27.09.2026

Did some testing in debug mode, and release mode. Removed all the debugging prints.

Tested on "/" get request, and on "/post/2" get request, not logged in, with 24 comments on the page.

Requests: 10k, Concurent connections: 1k

Stats (From benchmarks/speed.txt)


BEFORE:

With Prints to /: (Debug Mode)
Time: 21:02:10, Runs: 10, Speed (Seconds): 0.8531, Date: 2026-09-27
Time: 21:05:18, Runs: 10, Speed (Seconds): 0.8348, Date: 2026-09-27


Without Prints to /: (Debug Mode)
Time: 21:05:43, Runs: 10, Speed (Seconds): 0.8171, Date: 2026-09-27


Requests sent to /posts/2 (Post with 24 comments and no image) (Debug Mode)

Time: 21:07:16, Runs: 10, Speed (Seconds): 1.9910, Date: 2026-09-27

Time: 21:08:22, Runs: 10, Speed (Seconds): 1.9867, Date: 2026-09-27



Requests sent to /posts/2 (Post with 24 comments and no image) (Release Mode)

Time: 21:15:32, Runs: 10, Speed (Seconds): 1.0132, Date: 2026-09-27

Time: 21:15:55, Runs: 10, Speed (Seconds): 0.9995, Date: 2026-09-27

Time: 21:16:10, Runs: 10, Speed (Seconds): 0.9951, Date: 2026-09-27

Time: 21:16:25, Runs: 10, Speed (Seconds): 1.0038, Date: 2026-09-27



Flamegraph:

![Flamegraph](docs/flamegraph_27_09.svg)

Conclusion: Implement thread pooling

