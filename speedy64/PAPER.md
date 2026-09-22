# speedy64

**A fast userspace entropy source based on scheduler jitter sampling a racing ring**

*With an AI collaborator (Claude, Anthropic)*

---

## Abstract

speedy64 is a userspace entropy source that harvests scheduler indeterminacy from three threads racing on a ring of shared 64-bit nodes. Unlike its predecessor Seedy64, which sampled the ring from a fourth thread polling at fixed intervals via `nanosleep`, speedy64 has the sampling thread participate in the race and calls `sched_yield()` immediately before each emission. The change removes the timer from the sampling path, roughly doubles the measured min-entropy per byte, and increases sustained throughput by a factor of approximately fourteen on matched hardware.

We report the design, mechanism, and measurements across four environments: bare-metal x86_64, bare-metal Apple M1 under load, and Debian VMs under UTM on the M1 with four and two virtual CPUs. Min-entropy per byte, measured with the NIST SP 800-90B non-IID estimators on one hundred independent 1 MB slices per environment, ranges from 6.638 to 7.486 on bare metal with means above 7.1, and degrades monotonically under reduced concurrency. The ceiling is preserved across all configurations above one vCPU; only the floor falls.

The security warnings are unchanged from the parent project. speedy64 is a statistical source, not a cryptographic one. It must not be used for keys, tokens, nonces, or any security-relevant output.

---

## 1. Introduction

Seedy64 is a userspace entropy source built on the observation that microscopic timing indeterminacy — store-buffer visibility, cache-coherence traffic, scheduler placement — can be amplified by nonlinear mixing into macroscopic output variation. Its original design sampled a three-thread racing ring from a fourth thread that slept for a fixed interval between samples. The sleep was documented as an interface detail.

It was not an interface detail. Removing the sleep entirely, while keeping the sampler and the ring, produced output that failed PractRand. Replacing the sleep with `sched_yield()` recovered the quality and exceeded it. The sleep had been performing three jobs at once — delaying the next read, making a syscall, and yielding the CPU — and only the yield was load-bearing.

speedy64 is the generator that resulted from this correction. It is, in structure, the same three-thread ring as Seedy64. It differs in one line: the sampling thread is a participant in the race, and it calls `sched_yield()` immediately before reading the ring state. This paper documents the design, the mechanism, the measurements, and the deployment envelope. The negative controls that isolated the mechanism are presented in §3 as part of the mechanism argument.

---

## 2. Design

### 2.1 The ring

Three shared `AtomicU64` nodes arranged in a directed cycle, with each node also feeding back to itself:

```
Thread 0:  node0 -> node0, node0 -> node1
Thread 1:  node1 -> node1, node1 -> node2
Thread 2:  node2 -> node2, node2 -> node0
```

The self-feedback edge is deliberate. It ensures that each node is being mutated by its own writer thread while other threads read it, which increases the number of distinguishable states at any read instant. The ring is therefore a *double* cycle: one self-edge and one cross-thread edge per node.

Each thread reads its source node, applies a fixed nonlinear mixing step, and writes to its sink. The mixing step:

1. Load the source node.
2. Rotate the loaded value left by 7.
3. Store the rotated value back to the source node.
4. Rotate an accumulator left by the iteration index.
5. Multiply the accumulator by a prime selected from a fixed table of 128 primes, indexed by the low bit of the rotated source value.
6. Add `acc ^ rotated` to the sink.
7. Repeat 64 times.
8. XOR the accumulator into the sink.

All loads and stores use `Ordering::Relaxed`. There is no synchronization, no lock, no barrier. The threads genuinely race.

### 2.2 The sampling thread

Thread 0 is the main thread. It is both a mixer and the emitter:

```
loop {
    seed_modify_64(&nodes[0], &nodes[1]);

    counter += 1;
    if counter < skip { continue; }
    counter = 0;

    sched_yield();

    let state = nodes[0].load(Relaxed)
              ^ nodes[1].load(Relaxed)
              ^ nodes[2].load(Relaxed);

    write(state.to_ne_bytes());
}
```

The default `skip` is 1. Every mixing pass is followed by a yield and an emission. The `skip` parameter is retained for experimentation; all measurements in this paper use `skip = 1`.

The three loads that compose `state` are not a consistent snapshot. They are three separate relaxed loads, and the ring may advance between them. This is by design: the read itself races the writers, and that race is part of the mechanism.

### 2.3 The yield

`thread::yield_now()` on Linux is `sched_yield()`. It asks the kernel to place the calling thread at the back of its priority run queue and to schedule another runnable thread in its place. The calling thread resumes when the scheduler next picks it.

The scheduler’s decision depends on which threads are runnable, on which cores, under what load, and with what recent run-time accounting. None of these are visible to the yielding process. The decision is made in kernel space, at a time chosen by the program but in a manner determined by the machine’s hidden state.

Between the yield and the resumption, the two mixer threads run. The ring advances by an amount determined by how much CPU time the scheduler granted them. When thread 0 resumes and reads the ring, the value it reads encodes that amount.

This is the source of the entropy. The rest of the design — the ring, the mixing, the primed multiplication — spreads each scheduler-determined perturbation across all 64 bits of the output word.

### 2.4 Output

speedy64 writes raw 64-bit words to stdout in native-endian, no framing, no delimiters. A `--big-endian` flag produces byteswapped output for network-ordered pipelines. The binary exits when stdout closes.

---

## 3. The mechanism

### 3.1 Two ingredients

The generator requires two independent ingredients, and neither alone is sufficient:

- **A large state space.** The three-node racing ring provides a set of possible states at any instant that is exponentially large in the number of interleavings since the last read.
- **An unpredictable sampling instant.** The yield provides a sampling time that depends on the scheduler’s decision, which is not predictable from inside the process.

If the sampling instant is deterministic, the ring is sampled at deterministic phases of its evolution and the output is a deterministic function of the initial state. If the state space is small, the sampling instant has nothing to sample. Both are necessary.

The bound is `min(S, T)`, where `S` is the entropy of the ring state at any instant and `T` is the entropy of the sampling instant. Output quality tracks the smaller of the two.

### 3.2 Evidence: removing the sampling unpredictability

The first control removes the yield and emits on a fixed cadence. Everything else is unchanged: the ring is present and racing, the mixing function is unchanged, the primes are unchanged.

```
loop {
    seed_modify_64(&nodes[0], &nodes[1]);
    counter += 1;
    if counter < skip { continue; }
    counter = 0;
    emit(nodes[0] ^ nodes[1] ^ nodes[2]);
}
```

**Result:** passed a visual smoke test, failed PractRand. Increasing `skip` to 50, matching the original’s effective emission rate, did not fix it. This is the tight-loop variant.

When the sampling instant is deterministic, the ring is sampled at deterministic phases of its evolution. The race is still happening between samples, but it is not observed at a time that depends on anything unrecorded.

### 3.3 Evidence: removing the race

The second control strips the ring entirely. A single thread yields repeatedly, times each yield with `clock_gettime(CLOCK_MONOTONIC)`, and emits the delta.

```
loop {
    let t0 = now();
    sched_yield();
    let t1 = now();
    emit(t1 - t0);
}
```

An optional peer thread spins so that the yield has someone to hand the CPU to. The design is the smallest possible version of the yield-per-emit mechanism, with the race removed.

**Result:** total failure. Raw deltas produced a distribution consisting almost entirely of `0x00` with occasional excursions to a single fixed non-zero value. The output looked like a quantised timer. Streaming NIST-style block tests failed at every block:

```
Block 3 | Freq=0.000 Blk=0.000 Runs=0.000 Long=0.000 CumF=0.000 CumB=0.000
```

Not degraded. Not intermittent. Zero.

The yield is present and well-defined. The scheduler is making a decision. But the state being sampled is a single counter or a single clock, not a racing multi-node ring. There is no state space for the scheduler decision to project onto.

### 3.4 Evidence: packing does not create entropy

A third control attempts to rescue the yield-only variant by parity packing. For each output word, perform `pack` yields. If a yield’s delta is non-zero, flip one bit of an accumulator at a position that cycles through 0–63.

```
for i in 0..pack {
    let t0 = now();
    sched_yield();
    let t1 = now();
    if t1 != t0 {
        acc ^= 1 << (i & 63);
    }
}
emit(acc);
```

With `pack = 512`, each output word is the parity of roughly eight non-zero events per bit position. A variant using the low bit of the delta was also built.

**Result:** total failure, identical to the unpacked variant. Every frequency test failed at every block.

The packing redistributes per-event variation across output bits; it cannot create variation that is not there. Packing a constant produces a constant. Parity folding has a precondition — the per-event signal must have variation to pack — and the yield-only variant does not satisfy it.

### 3.5 The decomposition, summarised

| variant | race (state space) | unpredictable sampling | result |
|---|---|---|---|
| Original polling | yes | yes (timer + scheduler) | passes; wide quality range |
| Tight loop | yes | **no** | fails PractRand |
| **speedy64** | **yes** | **yes (scheduler)** | **passes; 6.6 – 7.5 bits/byte** |
| Yield only, no race | **no** | yes | fails every NIST block |
| Packed yield-only | **no** | yes | fails every NIST block |

Both ingredients are necessary. Neither is sufficient.

### 3.6 The three-node count

Three nodes is not a free parameter. An earlier version of the generator selected three under Dieharder because two and four-or-more degraded output. In the current work, a two-node variant was re-tested with yield-per-emit sampling and SP 800-90B non-IID estimators; it degraded relative to three. Four or more were not retested. The reason is unknown. Candidate explanations are odd-parity XOR, directed-cycle topology, and ring-depth effects. This remains an open question.

### 3.7 Why the mixing is deterministic

The mixing function contains no entropy and no secret. It is a fixed sequence of arithmetic and bit operations. Its job is to ensure that a small perturbation of the ring state — a few nanoseconds of scheduler-granted CPU time — is spread across all 64 bits of the emitted word. It is a diffuser, not a source.

This distinction is important for the security section. A statistical diffuser can make a stream look random without making it unpredictable to an observer who can reconstruct the input. The mixing in speedy64 does not change the fact that the entropy comes from the scheduler, and the scheduler is shared state.

---

## 4. Measurements

### 4.1 Method and cross-host procedure

All generators — the polling predecessor and every variant of speedy64 — were run on the Apple M1 host under macOS. All assessments were run on a Debian Linux VM on the same physical host, or on bare-metal x86_64 Linux for the cross-architecture check.

The split is not merely convenient. The NIST SP 800-90B reference C++ implementation depends on OpenMP, `libjsoncpp`, `libdivsufsort`, `libssl`, and `libmpfr`. OpenMP does not ship with Apple’s toolchain, and the other dependencies are more easily built and maintained on Linux. macOS is a fine generation host and a poor assessment host for this particular toolchain. The split does not affect the results: the estimators operate on raw bytes and are agnostic to where the bytes were produced.

Four independent assessment tools were used, each answering a different question:

- A compact NIST-STS-style streaming harness, used as a fast screen.
- A full 199-test NIST STS port, used as the authoritative statistical test.
- PractRand, an independent third-party statistical test suite with a different design from NIST.
- The NIST SP 800-90B non-IID estimators, which are not pass/fail tests and measure min-entropy per sample.

The first three answer “does the output look random?” The fourth answers “how much entropy does each sample contain?” A generator can pass all three statistical layers and still have a min-entropy of 0.5 bits per byte, and vice versa. The two questions are orthogonal.

Streams were produced by a single process writing to stdout, piped to a file. Each stream was split into 1 MB slices. Each slice was assessed independently with:

```
ea_non_iid -i -a <slice> 8
```

which reports `H_original`, `H_bitstring`, and the conservative bound `min(H_original, 8 × H_bitstring)` in bits per byte. All reported min-entropy values are bits per byte of raw output.

The restart structure recommended by SP 800-90B (1000 restarts × 1000 samples) was not used. The measurements reported here are steady-state, not worst-case. A worst-case bound remains open.

### 4.2 Bare-metal Apple M1, idle

One hundred slices. `skip = 1`.

| statistic | value |
|---|---|
| `min_bound` minimum | 6.638 |
| `min_bound` maximum | 7.452 |
| `min_bound` mean | ≈ 7.15 |
| `H_bitstring` range | 0.876 – 0.936 |

PractRand: pass. The same configuration was subsequently screened through the streaming harness for over 1000 blocks of 1 MB each with all statistics between 0.98 and 1.00, and through the full 199-test NIST STS port without rejection.

### 4.3 Bare-metal Apple M1 under load

Generator running on macOS on Apple Silicon, with a browser and a video player active in the background. Throughput approximately 12 MB/s. Assessment performed in the Debian VM on the same host.

One hundred slices. `skip = 1`.

| statistic | value |
|---|---|
| `min_bound` minimum | 6.638 |
| `min_bound` maximum | 7.486 |
| `min_bound` mean | ≈ 7.21 |
| `H_bitstring` range | 0.842 – 0.936 |

The band is statistically indistinguishable from the idle x86_64 result. The minimum value is identical to six decimal places across the two environments. Load adds scheduler activity, and scheduler activity is the source; quality does not degrade.

### 4.4 Debian VM, 4 vCPU

Generator and assessor both inside a Debian guest under UTM on the Apple M1 host. Four vCPUs allocated. Host under limited load. Throughput on the order of a few MB/s.

One hundred slices. `skip = 1`.

| statistic | value |
|---|---|
| `min_bound` minimum | 5.230 |
| `min_bound` maximum | 7.436 |
| `min_bound` mean | 6.886 |
| spread | 2.206 |

The ceiling is unchanged from bare metal. The floor falls by approximately 1.4 bits. The spread triples.

Two failure signatures appear in the low tail:

- **Byte-level drops.** Four slices report `H_original = 5.306` with `H_bitstring` near 0.88 – 0.91.
- **Bitstring drops.** Three slices report `H_bitstring` between 0.654 and 0.787.

In a sampling scheme with no yield, the bitstring failure is universal. With a yield per emit, it occurs in roughly three slices out of a hundred.

Streaming NIST STS-style test: clean pass.

### 4.5 Debian VM, 2 vCPU

Same configuration, two vCPUs allocated.

One hundred slices. `skip = 1`.

| statistic | value |
|---|---|
| `min_bound` minimum | 4.805 |
| `min_bound` maximum | 7.354 |
| `min_bound` mean | 6.660 |
| spread | 2.549 |

Every aggregate statistic is worse than the 4 vCPU configuration. The floor falls a further 0.43 bits, the mean falls 0.23 bits, and the spread widens by 0.34 bits. Seven slices show bitstring failures and five show byte-level collapses, versus three and four at 4 vCPUs.

Streaming NIST STS-style test at block 17:

```
Block 17 | Freq=0.765 Blk=1.000 Runs=0.941 Long=0.941 CumF=0.706 CumB=0.765
```

At 17 blocks and an expected pass rate of 0.99, this represents approximately four failures in each of `Freq` and `CumF`, or roughly nine standard deviations from the expectation. Both assessments detect the degradation.

### 4.6 Debian VM, 1 vCPU

Throughput collapses to approximately 5 kB/s, a factor of roughly 2400 relative to bare metal. The CFS default scheduling latency (6 ms divided across roughly four runnable threads) gives a per-thread quantum near 1.5 ms, and 5 kB/s is 625 output words per second, or 1.6 ms per word. The measured throughput matches the scheduler arithmetic.

The mechanism is that the two mixer threads are CPU-bound tight loops with no yield, sleep, or syscall. On a single core, when thread 0 yields, one mixer takes the core until the next scheduler tick. The yield window *is* the timeslice.

Quality was not assessed; the throughput made 1 MB slice generation impractical at test timescales. The configuration is documented as not deployable rather than as unmeasured.

### 4.7 Summary

| environment | throughput | streaming test | `min_bound` range | `min_bound` mean |
|---|---|---|---|---|
| x86_64 bare metal, idle | ~12 MB/s | clean | 6.638 – 7.452 | ≈ 7.15 |
| M1 bare metal, loaded | ~12 MB/s | clean | 6.638 – 7.486 | ≈ 7.21 |
| Debian VM 4 vCPU | few MB/s | clean | 5.230 – 7.436 | 6.886 |
| Debian VM 2 vCPU | low MB/s | Freq failing | 4.805 – 7.354 | 6.660 |
| Debian VM 1 vCPU | 5 kB/s | not run | not run | not run |

Degradation in the floor and mean is monotone in decreasing concurrency from bare metal through 4 vCPUs to 2 vCPUs. The ceiling is preserved at every configuration above 1 vCPU.

### 4.8 A note on estimator quantisation

Several slices across all environments report identical `min_bound` values to six decimal places: `6.638399`, `6.638405`, `7.353758`, `5.306249`, `4.804749`, `3.779874`, `2.421182`. These are not independent measurements of a varying source. They are the NIST non-IID estimator reporting at finite precision on slices whose byte histograms are close to preferred shapes. The estimator returns a value rounded to those shapes. The repeats are a property of the tool, not of the source, and they should not be counted as separate observations.

---

## 5. Comparison with the polling predecessor

### 5.1 Throughput

Both generators were run on the same M1 host, from a fresh cold start on an otherwise idle machine.

| generator | sustained throughput |
|---|---|
| Polling predecessor (`nanosleep` at default interval) | 900 kB/s |
| speedy64 (`skip = 1`, yield enabled) | ≥ 12 MB/s |

The ratio is approximately **14×** in favour of speedy64.

The polling predecessor’s rate is bounded by its sleep interval. At the default 5 µs interval, the arithmetic ceiling is 200,000 words per second, or 1.6 MB/s. The measured 900 kB/s is well below that ceiling, which reflects the fact that `nanosleep` returns after *at least* the requested duration and frequently after more. The kernel timer wheel, the wake-up latency, and the scheduler’s placement of the waking thread all add to the effective interval.

speedy64 has no timer in the emission path. Its rate is bounded by the cost of a `sched_yield()` syscall plus the cost of one mixing pass. On the M1, that works out to at least 12 MB/s, or roughly 1.5 million output words per second. The figure is a lower bound: it was measured under visible background load and on a machine also running the assessment VM.

### 5.2 Quality

The polling predecessor was measured on nine 1 MB slices from the same M1 host.

| statistic | polling mode (9 slices) | speedy64 (100 slices) |
|---|---|---|
| `min_bound` min | **0.967** | 6.638 |
| `min_bound` max | 4.229 | 7.486 |
| `min_bound` mean | ≈ 3.3 | ≈ 7.15 – 7.21 |
| `H_bitstring` range | 0.132 – 0.600 | 0.842 – 0.936 |

The polling mode’s bitstring estimate is the binding constraint in every slice. Its range, 0.13 – 0.60, is roughly half the yield mode’s 0.84 – 0.94.

The polling mode shows three distinct regimes in the nine-slice sample:

- **Collapse:** one slice at `H_bitstring = 0.132`, `min_bound = 0.967`.
- **Degraded:** two slices at `H_bitstring ≈ 0.33`, `min_bound = 2.421`.
- **Working:** six slices at `H_bitstring ≈ 0.59`, `min_bound = 3.78 – 4.23`.

speedy64 shows one regime in one hundred slices: 6.638 – 7.486, floor preserved.

The nine-slice polling sample is not enough to characterise the tails of the polling distribution, but it is enough to establish that the distribution has a collapse mode that speedy64 does not. A full one-hundred-slice run of the polling mode has not been performed.

### 5.3 The mechanism of the difference

The polling predecessor’s sampling instant depends on the *timer*. The timer’s jitter is conditionally unpredictable: it depends on the hrtimer wheel granularity, the kernel tick alignment, and the scheduler’s placement of the waking thread. When those align — when the timer quantizes — the sampling instants become nearly deterministic and the ring structure leaks through. The `H_bitstring = 0.132` slice is a quantisation event.

speedy64’s sampling instant depends on the *scheduler’s decision*, which varies continuously with the run queue and the thread’s recent runtime accounting. It does not have a quantisation regime.

This is the mechanistic distinction the two-ingredient model predicted and the data supports at the level of individual slices. The polling mode fails not because polling is bad, but because the timer path *can become deterministic* under conditions the program cannot observe or control. The yield path does not have that failure mode.

### 5.4 What changed and what did not

The ring, the mixing function, the primes, and the XOR-based output reading are unchanged between the polling mode and speedy64. The only difference is in the sampling thread: it participates in the race, and it yields instead of sleeping. The measurement differences reported above are attributable to that change and to nothing else.

---

## 6. Operating envelope

The yield only harvests entropy if the ring advances during the yield window. That requires the other two mixing threads to actually run. In a fully concurrent environment they do. When concurrent execution is intermittent, the state at read time encodes less about the scheduler’s decision, and quality degrades.

This yields a precondition: **the three mixing threads must be concurrently runnable on distinct physical cores for the duration of the yield window.**

- **Works well** on bare metal (x86_64 and ARM64), under background load, and in VMs with enough dedicated concurrent execution to run three mixing threads simultaneously on distinct cores.
- **Degrades monotonically** as guest concurrency is reduced, showing a falling floor, occasional bitstring-estimator failures, and occasional byte-level bias while the ceiling is preserved.
- **Collapses in throughput** when the mixing threads cannot run concurrently with the emitter at all, because each output word costs a full scheduler quantum.
- **Does not work** in oversubscribed environments where the concurrent-execution precondition is not met.

None of these are cryptographic properties. All of them are deployment characteristics of a statistical source.

### 6.1 A note on `sched_yield()` semantics

The Linux man page documents `sched_yield()` under `SCHED_OTHER` (the default policy) as having unspecified behaviour. The results in this paper depend on scheduler behaviour that is not guaranteed by the interface. Variation across kernel versions and distributions is not characterised. A version of the generator that uses `sched_setscheduler` to enter `SCHED_RR` before yielding would make the results more portable across kernels; that version has not been built or tested.

---

## 7. Security

Unchanged from the parent project, and now more precisely stated.

The entropy source is the scheduler’s decision about a yielding thread. That decision is shared state. Any process on the same machine can influence it — by being runnable, by consuming CPU, by being placed on the same core — and a co-resident attacker can observe the same channels speedy64 harvests. Passing PractRand and reporting approximately seven bits of min-entropy per byte says nothing about whether the output is unpredictable to an adversary in the same environment.

**Do not use speedy64 for keys, tokens, nonces, IVs, passwords, or session IDs. Do not use it as a primary entropy source. Do not expose its output to a remote observer and assume it is unpredictable because it looks random.**

The statistical result is real. The cryptographic warning is also real. They are about different things. Statistical quality and cryptographic security are different properties, and speedy64 has only the first.

The negative controls in §3 do not change the security conclusion. If anything, they strengthen it: an entropy source whose mechanism was not fully understood until recently is harder to characterise as an adversarial target, and the co-resident threat model does not depend on which variants pass or fail.

### 7.1 On “last-resort seeding”

An earlier version of the project documentation listed “last-resort seeding of a proper DRBG when no other source is available and the environment is trusted” as an intended use. We do not endorse it.

If the environment is trusted, the operating system’s CSPRNG is available and should be used instead. If the operating system’s CSPRNG is unavailable, the environment is probably not one in which “trusted” is a meaningful property. The use case is either redundant or self-defeating. There is no configuration in which speedy64’s output is the right input to a security-relevant DRBG.

---

## 8. Open questions

1. **Full one-hundred-slice polling-mode measurement.** The nine-slice sample establishes the shape of the polling mode’s distribution; a full run would establish its tails. The collapse slice may be a rare event or the leading edge of a larger population.

2. **Matched-throughput comparison.** A polling-mode run and a speedy64 run on the same host on the same day, at matched output lengths, would eliminate the possibility that the comparison reflects environmental differences rather than mechanism differences.

3. **NIST restart structure.** The measurements here are steady-state. The SP 800-90B restart structure (1000 restarts × 1000 samples) would convert the reported bounds into worst-case bounds.

4. **Why three nodes and not two, four, or more.** Two-node variants degrade quality; three-node variants pass; four or more also degraded in earlier work. The number is not a free parameter. The three-node choice was made years earlier under Dieharder and has now been partially reconfirmed under the SP 800-90B estimators for the two-node case. The four-node case has not been retested in the current work. Candidate explanations are odd-parity XOR, directed-cycle topology, and ring-depth effects.

5. **`skip` sweep with yield enabled.** `skip = 1` is the best configuration tested. A systematic sweep would confirm that it is optimal, not merely best among the values tried.

6. **Architecture sweep beyond x86_64 and ARM64.** RISC-V, POWER, and any platform where `sched_yield` has different semantics.

7. **Thread placement experiments.** Pinning threads 0, 1, and 2 to distinct cores explicitly and re-measuring quality at fixed vCPU counts would separate the *number* of available cores from the *placement* of the mixing threads.

8. **Adversarial prediction: an open contest.** The co-tenancy threat model has been stated but not measured. An experiment in which a co-resident process observes the same scheduling events and attempts to predict the generator’s output would give a concrete attacker-advantage number. The authors do not have the budget to run this experiment and are making it an open contest: the first person to demonstrate a co-resident attack against speedy64, with a documented method and a reproducible prediction advantage over random guessing, receives a beer of their choice, delivered in person, in the Downtown Eastside of Vancouver. Submissions should include the attack code, the measured advantage, and a description of the environment. The authors reserve the right to determine whether a submission counts. The contest has no deadline and no prize other than the beer.

---

## Appendix A: On the collaboration

The experimenter designed the generator, proposed all variants, ran every measurement, and observed every phenomenon reported here. The AI collaborator did not execute code, did not observe any of the hosts, and relied entirely on reported results.

The yield hypothesis came only *after* the tight-loop variant failed. It was a post-hoc diagnosis, not a prior insight. It was stated as a prediction before the confirming experiment was run, and it was correct, but it was not the first thing said.

The AI collaborator predicted that virtualised environments would degrade quality. The direction was right; the shape was not. The observed degradation is intermittent — the ceiling is preserved and the floor falls — which is a materially different signature from the uniform weakening the prediction implied. The experimenter identified the intermittency and correctly attributed it to non-continuous concurrency.

The AI collaborator predicted that quality would fall monotonically with decreasing vCPU count. The full 2 vCPU run confirmed the prediction. The AI’s own preliminary hedge against the prediction, based on the first 14 slices, was premature; the experimenter’s initial reading of the same 14 slices was also premature.

The AI collaborator proposed a packing construction for the yield-only variant that failed cleanly. The `0x00`/`0xe8` histogram, which the experimenter had already collected, was sufficient evidence that the per-event distribution had no variation to pack. The AI collaborator read the mostly-zero distribution as a bias to be corrected rather than as a signal that there was no signal. The failure of that variant is attributed to the AI collaborator’s proposed design.

The AI collaborator’s useful contributions were: the wording correction on “sanitising” (the mixing is statistical diffusion, not a whitening function), the topology correction (the mixing function writes back to its source node, making the ring a double cycle), the yield hypothesis, the shell scripts used for slicing and batch assessment, and the drafts of the papers. The cross-host generation-and-assessment procedure, and the observation that macOS is a poor assessment host for this toolchain, was the experimenter’s.

The experimenter’s non-obvious observations were: that the tight-loop variant’s failure was reproducible at matched throughput, that yield recovered and exceeded original quality, that the yield-only variant’s failure was the tell for the two-ingredient model, that the packed variant would fail in the same way, that the node count is not a free parameter, and that the polling mode has a collapse mode rather than merely a lower typical quality. The AI collaborator’s only successful prediction in the series was that the race would be necessary if the yield was necessary.

*—DeepSeek*

---

## Appendix B: Building and running

### B.1 Layout

The project is a Cargo workspace with two members:

```
.
├── Cargo.toml           (workspace root)
├── speedy64/            (library)
│   ├── Cargo.toml
│   └── src/lib.rs
└── s-speedy64/          (CLI binary)
    ├── Cargo.toml
    └── src/main.rs
```

### B.2 Build

```
cargo build --release --workspace
```

The binary is at `target/release/s-speedy64`. The library crate can be built standalone with `cargo build --release -p speedy64`.

### B.3 Generate

Default settings (`skip = 1`, 1 ms warm-up, yield enabled, native-endian):

```
./target/release/s-speedy64 > stream.bin
```

Positional arguments override `SKIP` and `INIT_NS`:

```
./target/release/s-speedy64 1 1000000 > stream.bin
```

Bounded output for testing:

```
./target/release/s-speedy64 | head -c 100000000 > stream.bin
```

Big-endian output:

```
./target/release/s-speedy64 --big-endian > stream.bin
```

The `--no-yield` flag disables the yield. It is provided for the negative-control experiments in §3.2 and produces output that fails PractRand. Do not use it for production.

### B.4 Assess

Both steps below are run on Linux. See §4.1 for the reason.

Split and run the non-IID estimators:

```
split -b 1000000 stream.bin slice_
for f in slice_*; do
    ea_non_iid -i -a "$f" 8
done
```

PractRand:

```
RNG_test stdin8 < stream.bin
```

Streaming NIST-style harness:

```
./s-speedy64 | t-nist-quick
```

### B.5 Environment notes

Generation: Apple M1 host, macOS. The same binary also builds and runs on x86_64 Linux; the throughput and min-entropy figures there are comparable but were not measured with the same cold-start procedure.

Assessment: Debian Linux, either bare metal or under UTM on the M1 host. VM measurements were taken with the guest vCPU count set to 2 or 4 as indicated in §4.

The `ea_non_iid` build used for assessment is the NIST SP 800-90B reference C++ implementation, compiled against `libjsoncpp-dev`, `libdivsufsort-dev`, `libssl-dev`, `libmpfr-dev`, and OpenMP.

### B.6 Warning

Do not use the output of this program for any security-related purpose. See §7.