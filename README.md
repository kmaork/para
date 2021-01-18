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
See [the integration tests](./tests/test.rs)

## TODO
### Features
- Performance regression tests
- CI/CD
- Add benchmark that can be optimized using work-stealing
- Make it easier to pass producers to schedule() (why not consume? maybe if we want to stop before the end?)
- Improve macro. Better support fanout.
- Support adding priority for nodes
- Support marking nodes as running on external HW (e.g. GPU)
- Support stateless producers? Rayon-style splittable iterators?
- Implement Fanout for any iterable of consumers 
### Optimization
- When running short tasks, threads spend significant time synchronizing pushing and popping from the task queue.
  The usual solution for that would be to implement work stealing.
- Stateful consumers are held inside a mutex, so threads might spend time blocking.
  A possible improvement is to modify the `consume` interface to return a boolean and use `try_lock` when executing functions in a mutex.
  This might be less problematic once we implement work stealing. Also, we might wanna use a refcell instead of a mutex.
- Tasks contain a pointer to a node, which is wasteful as it is has 64 bits while we usually have around 3 bits of nodes.
  We could use an id instead of reference, or collect tasks per-node. Collecting tasks per node might allow us to further optimize
  running of stateful tasks.
- We want all cores to always work. There could be a situation in which there are many tasks, but all are for a specific
  Stateful node, so they can't be parallelized. For that reason, stateful nodes should always be prioritized.
- Smartly manage the balance between having many ready tasks and memory usage, by knowing which nodes produce more work
  and which nodes consume more work, and prioritizing them according to current workload.
### WIP
- Support fanout in macro
- Support multiple producers in macro