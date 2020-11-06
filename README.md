# Para
This project is a work in progress. Feel free to contribute.

## The goal
Efficient parallelization of pipelined computations on multiple cores with a simple interface.

The interface should look something like:
```rust
let data = vec!(0, 1, 2);
let mut sum = 0;
pipeline!(4, data
          => (|x| (x * 2, x) => |(x2, x)| x2 - x,
              |x| -x)
          => |x| sum += x);
```
In this example, the tasks to be done are:
- Iterate over the elements in `data`
- Clone each element and pass it to both the `|x| (x * 2, x)` and `|x| -x` closures.
- Apply the output of `|x| (x * 2, x)` to `|(x2, x)| x2 - x`
- Sum all outputs of `|(x2, x)| x2 - x` and `|x| -x` into the `sum` variable.

This constructs a graph in which each node is a closure. Data flows between the closures and gets processed.
Except for the first and the last nodes in this example (the iteration and the sum nodes),
all nodes are completely parallelizable. The `pipeline!` macro will instantiate a scheduler
which will use 4 threads to run all nodes and maximize computation throughput.

## Current state
See [the integration test](./tests/test.rs)