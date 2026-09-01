# SEEDY64

`Seedy64` is a **nondeterministic random bit generator** that harvests entropy from **uncontrolled thread interleaving over shared mutable state**, accessed via **volatile loads/stores** without synchronisation. It exploits the resulting **data races** to drive a **high-dimensional chaotic mixing process** whose output is statistically robust across different memory consistency models. Beyond being a chaotic oscillator, the system is best described as a **critical, self-referential topology** that sits in a metastable regime between order and disorder, amplifying microscopic nondeterminism into macroscopic randomness.

## 1. System structure

The generator maintains three shared `u64` words, `nodes[0..2]`, initially zero. Three threads are spawned, each executing an infinite loop that reads from one node (source) and writes to another (sink), forming a directed ring:

```
Thread 0:  node0 → node1
Thread 1:  node1 → node2
Thread 2:  node2 → node0
```

All accesses use `read_volatile` / `write_volatile`. In Rust (and by analogy in C/C++), `volatile` **disables compiler optimisations**—preventing elimination, reordering, or caching in registers—but imposes **no hardware ordering or atomicity constraints**. This means the threads genuinely race: writes may be buffered, reordered, and interleaved arbitrarily according to the hardware memory model.

> **Language semantics and portability.** In Rust, `read_volatile` and `write_volatile` do not provide atomicity or synchronization. Concurrent unsynchronized access to the same memory location where at least one access is a write constitutes a data race and is formally undefined behavior. The design intentionally exploits this to access the hardware’s relaxed memory behavior, but it is not a portable, language-guaranteed entropy source. It should be viewed as an experimental study in concurrency dynamics, not as a production-ready random generator.

## 2. The mixing function

Each thread performs 64 iterations of a non-linear update on its private accumulator `acc` and on the shared nodes:

```
acc  = 1
for i in 0..63:
    val     = volatile_read(source)
    rotated = val.rotate_left(7)
    volatile_write(source, rotated)

    acc = acc.rotate_left(i)
         .wrapping_mul( PRIMES[ 2*i + (rotated & 1) ] )

    sink = volatile_read(sink) .wrapping_add( acc ^ rotated )
           volatile_write(sink, ...)
sink ^= acc   // finalisation
```

The `PRIMES` table contains 128 distinct large primes, selected to provide strong non-linearity and diffusion. The loop couples `source`, `sink`, and the accumulator in a way that resembles a **chaotic map with expanding step-dependent rotations and modular multiplications**. The interleaving across threads creates cross-coupling because each thread’s source is another thread’s sink.

## 3. Chaotic dynamics induced by races

From a dynamical systems viewpoint, the full state space is enormous: three shared nodes, three thread-local accumulators, plus the state of store buffers, caches, and scheduling. The threads form a **closed feedback loop** where:

- The exact sequence of values each thread reads is **non-deterministic** due to store-buffer forwarding, cache-coherence traffic, and OS preemption.
- Small timing differences (nanosecond granularity) cause different interleavings, which are amplified exponentially by the mixing function's sensitivity to initial conditions.
- Because there are no barriers or locks, the hardware may reorder a thread’s own writes relative to later reads, and certainly reorder writes from different threads, giving rise to an effectively **nondeterministic interleaving space** that is sampled by the main thread.

The system therefore operates in a **chaotic regime** where the Lyapunov exponent is large enough to decorrelate the state extremely quickly, and the set of reachable trajectories is vast.

> **Epistemic vs. ontic nondeterminism.** Strictly speaking, the system’s behavior is deterministic at the level of the entire hardware-software stack (assuming no quantum randomness in the CPU). However, from the program’s internal state, the exact interleaving is unpredictable and effectively random because it depends on hidden variables: OS scheduler decisions, cache-coherence traffic, and microarchitectural timing. The generator therefore harvests **epistemic randomness** — uncertainty that is irreducible from its own point of view. This is analogous to how a physical system can appear random even if the underlying laws are deterministic, because the observer lacks access to all microscopic degrees of freedom.

## 4. Fluid topology of information flow

You described the topology as “fluid.” Formally, the **information flow graph** among the nodes is time-varying. In a static view:

```
node0 ──Thread0──> node1 ──Thread1──> node2 ──Thread2──> node0
```

But because each read may pick up a value written by any thread (or even an intermediate value due to partial cache-line visibility), the *effective* graph at any instant may have edges `node2→node1` (if Thread1 reads a write from Thread2 before Thread0’s write propagates), or `node0→node2`, etc. This turns the system into a **non-autonomous dynamical network** whose coupling topology shifts at every access. Such rewiring is a known mechanism for generating **high-dimensional chaos** in complex systems.

> **Criticality.** The generator’s causal graph is a closed directed cycle with feedback. It is self-referential in the sense that each node’s output feeds back into its own future state through the other nodes. This structure is not merely cyclic — it is tuned to a critical point where:
> - perturbations neither die out (stable fixed point) nor blow up (unbounded divergence);
> - microscopic variations in thread interleaving are **amplified** into macroscopic differences in the output;
> - the system remains bounded and persistently non-repeating, indicating a metastable chaotic attractor.
>
> In the language of complex systems, this is a **critical topology**: a self-referential network poised at the edge of instability, where small fluctuations can determine large-scale outcomes. This property is what allows the system to continuously harvest entropy from the micro-environment without external reseeding.

## 5. The “only-on-change” extraction and clock jitter

The main thread polls the system state (`nodes[0] ^ nodes[1] ^ nodes[2]`) at fixed intervals (`interval_ns`). It emits an output word **only when the state differs from the previous sample**. This serves two purposes:

1. It discards intervals where the chaotic trajectory happens to be quasi-stable (no observable change in the XOR-sum), ensuring that output is drawn only from **state transitions**.
2. The sampling points are effectively **modulated by the system’s own dynamics**: the intervals between changes vary non-uniformly, introducing timing jitter that further decorrelates output from wall-clock time.

## 6. Architectural independence (ARM vs. Intel)

PractRand finds no statistical difference between outputs produced on ARM (weakly ordered) and Intel (TSO-x86) processors. This is notable because these memory models allow different hardware reorderings. The invariance indicates that the macroscopic statistical properties—namely, the bitstream’s uniformity and lack of correlation—are **independent of the fine-grained memory-ordering rules**. In dynamical terms, the system’s strange attractor has a structure that is **structurally stable** against the kinds of perturbations introduced by different memory models. The chaos is sufficiently strong to dominate over the specific microarchitectural reorderings.

## 7. Metastability and amplification

The system does not settle into a fixed point or cycle, but it also does not diverge. That is exactly **metastability** — a persistent far-from-equilibrium state that is stable on average but locally unstable. Small perturbations do not simply decay; they are fed back into the loop and magnified.

> **Amplification of microscopic fluctuations.** The three-thread cycle acts as a sensitive amplifier. A nanosecond-level preemption or cache-line delay changes the order of volatile reads/writes, which then propagates through 64 iterations of nonlinear mixing and around the feedback loop. The result is that a microscopic perturbation in the execution environment is converted into a macroscopic, fully decorrelated output word. This is the same structural principle by which quantum events in a critical physical system can be amplified into macroscopic outcomes — except here the “quantum” layer is replaced by the hidden micro-state of the memory hierarchy and scheduler.

## 8. Discussion: Relation to natural critical systems

The design resembles biological and neural systems that operate near criticality. For example, cortical networks maintain a balance between excitation and inhibition, allowing them to be sensitive to small inputs while remaining globally stable. Similarly, `Seedy64` uses three interacting feedback threads to stay in a regime where microscopic timing fluctuations are constantly amplified into macroscopic novelty. This suggests that self-referential critical topologies may be a general mechanism for converting low-level indeterminacy into high-level complexity. The generator is thus not only an engineering curiosity but also a small working model of how life and mind might exploit physical indeterminacy to generate genuine novelty and agency.

## 9. Summary

`Seedy64` is best understood as a **concurrency-driven chaotic oscillator** that operates at a **critical point** of a self-referential topology:

- **Shared mutable state** with volatile accesses creates uncontrolled data races.
- **A nonlinear mixing function** with multiple feedback loops amplifies tiny interleaving differences.
- **A circular dependency graph** with time-varying effective topology generates fluid, high-dimensional dynamics.
- **Metastability** is maintained by the balance between mixing and bounded arithmetic, preventing both collapse and divergence.
- **Architectural robustness** emerges because the chaotic regime is deep enough to absorb memory-model differences.

It is not a cryptographically verified CSPRNG, but as a study in **synthetic entropy generation from parallel non-determinism**, it elegantly illustrates how to push a memory subsystem into a statistically useful chaotic phase—and, more deeply, how a critical self-referential topology can transform microscopic unpredictability into macroscopic randomness.