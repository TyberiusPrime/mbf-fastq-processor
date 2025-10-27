status: open
# Work engine implementation

We're currently using a thread-per-step.

Write an alternative implementation that has just one thread,
that moves a block through all the steps, and benchmark it vs
the original.

-- 
I tried async, that was essentially the same performance.
Bonus is that it might work WASM - though tokio and wasm is a topic in itself.


