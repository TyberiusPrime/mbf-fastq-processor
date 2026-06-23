Goal: Replace pipe with shared memory between fastqrab and fastqrab-decompressor 

  Design constraint (the thing we must not break)

  rusty_rapidgzip decodes directly into recycled Vec<u8>s — decode_one_chunk
  takes a recycled buffer (pipeline.rs:1089), clear()s it, and inflates into it;
  the buffer is handed back via recycle_tx after the consumer is done. Those
  warm, reused, growable buffers are the speed. The decoder can also grow a
  buffer mid-chunk (incompressible/stored blocks), so the decode target must be
  a real heap Vec, not a fixed shared slot.

  Conclusion: we keep rusty_rapidgzip 100% untouched, including its recycle 
  pool. The decompressor still decodes into its private recycled Vecs. The only
  thing we change is what the decompressor does with a finished chunk: today it
  write_alls it to a pipe (a copy into the kernel + a second copy out on the
  consumer's read); instead it will memcpy it into a shared-memory slot and send
  a tiny descriptor. That trades two single-threaded bulk copies + ~52 s of 
  syscall/system time for one plain memcpy and no bulk syscalls, and lets the
  consumer parse in place. It does not require touching the decode hot loop or
  the recycle pool.

  (There's a deeper variant that also removes that one residual memcpy by
  decoding into shared slots — but it fights the growable-Vec recycling you
  flagged, so I'm explicitly deferring it to a measured Phase 2, not baking it
  in.)

  Architecture

  fastqrab (parent)                         fastqrab-decompressor (child)
    creates memfd (N×slot_size), mmaps  ──fd──▶  mmaps same region (MAP_SHARED)
    spawn child: --shm-fd --slots --slot-size
                                                read_gz → recycled Vec
  (UNCHANGED)
    child.stdout  ◀── (slot:u32, len:u32) ────  main: memcpy Vec→free slot, emit
  desc,
     = descriptor pipe                            recycle Vec back to
  rusty_rapidgzip
    child.stdin   ──── freed slot:u32 ────▶  reader thread refills free-slot
  pool
     = slot-return pipe

  - Bulk data: shared memfd region, N slots of slot_size. Only data lives here.
  - Two control pipes (cheap, blocking, give natural backpressure — 8 bytes per
  multi-MiB chunk, negligible): child stdout carries (slot, len) descriptors in
  order; child stdin carries freed slot indices back. Pipes (not a lock-free
  shared ring) on purpose: blocking semantics = free backpressure, no spinning,
  and the syscall is the cross-process synchronization edge for the mmap writes.

  Slot lifecycle: free → decompressor owns (memcpy in) → descriptor sent →
  consumer owns (parse-copy out) → slot id returned → free. Exactly one owner at
  a time; ownership is transferred through the pipe messages.

  Chunk larger than slot_size: the decompressor splits it across multiple slots,
  one descriptor each. The consumer already tolerates arbitrary chunk
  boundaries (it carries partial-line state), so this is free on the consumer
  side.

  Touch list

  1. fastqrab-decompressor/src/main.rs — add a shared-memory output mode, gated
  by --shm-fd <fd> --shm-slots <n> --shm-slot-size <bytes>. Without those flags
  it behaves exactly as today (stdout bytes), so the standalone tool and its
  gunzip-equivalence tests are untouched. In shm mode:
  - mmap the inherited fd MAP_SHARED.
  - A small thread reads freed-slot ids from stdin into a free-slot channel.
  - The existing read_gz producer is unchanged. The main loop replaces
  out.write_all(&chunk) with: acquire free slot(s), memcpy, write descriptor(s)
  to stdout, then recycle the Vec via recycle_tx exactly as now (main.rs:113).
  - EOF sentinel descriptor, then clean exit.

  2. fastqrab-io/src/io/input.rs — keep spawn_rapidgzip(...) -> File (pipe mode)
  for FASTA and any generic Read consumer (fasta.rs:42,
  open_decompressed_reader:425 stay as-is). Add spawn_rapidgzip_shm(...) that
  creates the memfd + two pipes, clears FD_CLOEXEC on the region fd, spawns the
  child with the shm flags, and returns a handle { mmap, descriptor_rx_pipe, 
  slot_return_tx_pipe, child }.

  3. fastqrab-io/src/io/pod_parser.rs (Phase 0 refactor) — generalize the byte
  source. Today DemuxJob.chunk: Arc<Vec<u8>> and the worker recycles via
  Arc::try_unwrap (pod_parser.rs:115). Introduce a chunk abstraction that yields
  &[u8] and releases on drop:
  - Owned(Arc<Vec<u8>>) → recycles the Vec (current behavior; keeps all existing
  callers + tests working),
  - Shared { slot_ptr/len, slot, release: Sender<u32> } → returns the slot id on
  drop.
     Drop-based release makes it panic-safe (slot always returned). demux_chunk
  already only needs &[u8], so the change is mechanical.

  4. fastqrab-io/src/io/parsers/fastq.rs — in PodFastqParser::new, when
  Rapidgzip + format is gzip, take the shm path instead of
  open_decompressed_reader: the reader thread reads (slot, len) descriptors,
  wraps each as a Shared chunk borrowing the mmap, feeds the pod parser; the
  release channel writes freed slot ids to the child's stdin. Non-gzip
  (zstd/raw) and the Default path keep the existing Read-based reader untouched.

  FASTA stays on the pipe (rare, not perf-critical) — smaller blast radius on
  the correctness-critical parser.

  Phasing

  - Phase 0 — pod_parser chunk abstraction + drop-release. Pure refactor, no
  behavior change; existing tests prove equivalence.
  - Phase 1 — shm mode in the decompressor + spawn_rapidgzip_shm + wire
  PodFastqParser. This is the payload.
  - Phase 2 (only if measured as the ceiling) — the residual single-threaded
  memcpy in the decompressor main loop. Cheapest fix: a tiny copier pool
  (parallel memcpy, ordered descriptor emission). Deeper fix (decode-into-slots)
  stays off the table unless Phase 1+copier still doesn't hit target, because
  it endangers the recycle pool.

  Safety, failure, fallback

  - unsafe: memfd_create/mmap/fcntl and the &mut [u8] over the region. Invariant
  documented: each slot has a single owner at a time, ownership handed off via
  pipe messages (the read/write syscalls are the synchronization edges); writers
  touch a slot only before sending its descriptor, readers only between
  receiving and returning it. No overlapping access ⇒ no data race.
  - Teardown / crashes: child death → stdout closes → consumer sees EOF +
  non-zero exit → error. Consumer death → child gets EPIPE/EOF on the pipes →
  exits. No leaked processes.
  - Deadlock: bounded ring; both sides always make forward progress and
  free/consume, so no stall as long as N ≥ pipeline depth (N = 2–4 × 
  max(decomp_threads, demux_threads), tunable).
  - Fallback toggle: env FASTQRAB_DECOMP_SHM=0 forces the existing pipe path, so
  we can A/B and have a field escape hatch.

  Tests

  - Existing rapidgzip/{basic,single_thread} must stay byte-identical (they
  exercise the new path).
  - New case: large multi-chunk input to exercise slot recycling + EOF.
  - Incompressible input → forces chunk-output > slot_size → the multi-slot
  split path.
  - Tiny ring (N=2) stress case → heavy backpressure/recycling, asserts
  correctness + no deadlock.
  - Standalone decompressor pipe mode unchanged → its sha256 == gunzip check
  still holds.

  Expected outcome / what we'll measure

  System time should collapse (no bulk read/write syscalls), the consumer's
  single-threaded ::read ceiling disappears, and the decompressor's 16 threads
  get unleashed — CPU should climb toward saturation and wall toward "standalone
  decode (~7 s) + real parse cost." We re-check time -v (system time, voluntary
  ctx switches) and samply (is the decompressor now the gate, and is the
  main-loop memcpy the residual serial cost → decides Phase 2).

  ---
  Two things before I start: do you want me to proceed through Phase 0 + Phase 1
  and stop for a benchmark before any Phase 2, and should I drop this plan into
  a doc (e.g. docs/development/) so it's reviewable alongside the code? Also
  flag if you'd rather I send the region fd via SCM_RIGHTS over a socketpair
  instead of fd-inheritance — both work; inheritance is simpler, the socket is
  tidier if you dislike CLOEXEC fiddling.

✻ Baked for 4m 57s
