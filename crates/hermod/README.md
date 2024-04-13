<!-- cargo-rdme start -->

# hermod

Intra-process communication utility crate.

## Event Emitter
<sub> Requires `events` feature </sub>

The event emitter can be used to emit and register listeners
from anywhere on your codebase.

### Features

 - **Emit from anywhere**: You can emit events from anywhere using
   an immutable reference - using an `Arc`, for example.

 - **Async callbacks**: Hermod was made to be used asynchronously,
   so the callbacks you register are async.

### Drawbacks

 - Registering listeners requires a mutable reference. You
   must put the emitter in a lock (`Arc<Mutex<...>>`) or
   register them all in one place.

## Queue
<sub> Requires `queue` feature </sub>

The queue can be used from anywhere to send messages to a
single-threaded callback.

### Features

 - **Send from anywhere**: You can send messages from anywhere
   using an `Arc`. You can even put it in a `static ref` (from
   `lazy_static`)

 - **Persistant data**: You can persist some data between calls.
   Because the queue is single-threaded, we can just use a mutable
   reference with no overhead.

<!-- cargo-rdme end -->
