
# What the Polling Loop Was Actually Doing

**A yield-per-emit variant of Seedy64, and the experiment that isolated its entropy mechanism**

*With an AI collaborator (Claude, Anthropic)*

---

## Abstract

Seedy64 is a userspace entropy source that harvests timing indeterminacy from three threads racing on a ring of shared 64-bit nodes. The original implementation samples the ring from a fourth thread that polls at fixed intervals via `nanosleep`. We treated the polling interval as an interface detail and built a variant in which the sampling thread is itself a participant in the race: it mixes alongside the other two and emits every *n* cycles, with no sleep and no timer.

The variant failed PractRand. A second variant, identical except for a `sched_yield()` immediately before each emission, passed cleanly and roughly doubled the measured min-entropy per byte. This identifies the load-bearing component of the original design: not the timer delay, not the act of polling, but the scheduler decision that the polling forced.

We characterised the yield-mode source across four environments: bare-metal x86_64, bare-metal Apple M1 under load, and a Debian VM under UTM on the M1 with 2 and 4 vCPUs. The min-entropy band is stable across bare-metal architectures and under load. It degrades monotonically as guest concurrency is reduced: a 1.4-bit drop in the floor at 4 vCPUs, a 1.9-bit drop at 2 vCPUs, and a throughput collapse to 5 kB/s at 1 vCPU that makes the configuration undeployable.

The degradation is intermittent rather than uniform. The ceiling is preserved across all multi-core configurations; only the floor falls and the spread widens. This is consistent with a scheduler-decision mechanism whose yield window requires continuous concurrent execution of the three mixing threads.

Security warnings are unchanged. Statistical quality and cryptographic security remain different properties.

---

## 1. Background

Seedy64's design and its non-cryptographic status are described in the project documentation. The relevant details here:

- Three threads operate on three shared `AtomicU64` nodes in a ring. Each thread reads its source, applies a fixed nonlinear mixing step (rotate, multiply by a parity-selected prime, add, XOR), and writes to its sink. All accesses are relaxed. There is no synchronization.
- The original implementation samples the ring from the main thread, which sleeps for a fixed interval, reads `node0 ^ node1 ^ node2`, and emits one 64-bit word.
- The project had measured statistical quality via NIST STS and PractRand, and had explicitly **not** measured min-entropy per sample. That was listed as the main open item.

The design intent was that microscopic timing indeterminacy — store-buffer visibility, cache-coherence traffic, scheduler placement — would be amplified by the mixing into macroscopic output variation. The code supports that reading. What it did not specify is *where* in the loop the indeterminacy enters.

---

## 2. The variant and its hypothesis

The variant was proposed as an optimization. If the sampling thread participates in the race rather than observing it, the design loses a thread and a timer, the main loop becomes a tight mixing loop, and the emission cadence is determined by a skip count rather than by the OS timer. The hypothesis was that this would be at least as good — that the ring's internal race carries the entropy, and that polling was a wrapper around it.

This hypothesis was wrong, and the way it was wrong is the substance of this paper.

---

## 3. The failure

The variant was built as a standalone binary writing raw 64-bit words to stdout, with `skip` as a positional argument. The ring, the mixing function, and the primes were unchanged. Thread 0 mixes and emits; threads 1 and 2 are pure mixers; the main thread *is* thread 0.

It passed a visual smoke test and failed PractRand. Increasing `skip` to 50, which matches the original's effective emission rate, did not fix it. The output looks random to the eye and is not random to a statistical test — the normal condition of a generator with low-order structure.

---

## 4. The yield hypothesis

The original polling loop did three things at once: it delayed the next read, it made a syscall, and it *yielded the CPU*. The tight-loop variant removed all three. The question was which one mattered.

The proposed diagnosis was that the yield was the relevant component. `nanosleep` hands control to the scheduler; when the thread wakes, it may resume on a different core, after an unpredictable number of other scheduling decisions, with different cache and TLB state. The ring advances by an unknown amount during that window. When the sampler reads the state, the value encodes how much the scheduler let the other threads run — a quantity that is not recorded anywhere and not reconstructible from inside the process.

If that diagnosis is right, then inserting `sched_yield()` immediately before each emission should recover the quality, even with no timer and no sleep. This was stated as a prediction before the experiment was run.

It was correct.

---

## 5. Method

### 5.1 Modes assessed

- **Polling (original).** Main thread sleeps for a fixed interval, reads the ring, emits.
- **Tight loop.** Sampler mixes with the other two threads, emits every `skip` cycles. No syscall, no yield, no timer.
- **Yield per emit.** Identical to the tight loop, except `thread::yield_now()` is called immediately before reading the ring state.

### 5.2 Generation

Each stream was produced by a single process writing to stdout, piped to a file or to a truncating consumer. The generator binary was built from a single `src/main.rs` with `clap` argument parsing. Invocation:

```
seedy64-threaded --emit-yield <skip> <init_ns> > stream.bin
```

Unless otherwise noted, `skip = 1` and `init_ns = 1_000_000` (1 ms warm-up).

### 5.3 Note on `skip = 0`

The `skip` condition in the emit loop is `if counter < skip { continue; }` after incrementing `counter`. With `skip = 0` and `skip = 1`, the first iteration both take the emit branch. The two settings are functionally identical. They are not treated as distinct operating points in this paper.

### 5.4 Assessment

Streams were split into 1 MB slices. Each slice was assessed independently with the NIST SP 800-90B non-IID estimators:

```
ea_non_iid -i -a <slice> 8
```

which reports `H_original`, `H_bitstring`, and the conservative bound `min(H_original, 8 × H_bitstring)` in bits per byte. All reported min-entropy values are bits per byte of raw output.

Some environments were also screened with a streaming NIST STS-style pass/fail check on 1 MB blocks, reporting per-test pass fractions across blocks. The streaming test accumulates evidence across many blocks and is sensitive to slow drift; the non-IID batch estimators are sensitive to local structure. The two are complementary.

### 5.5 Cross-host procedure

Samples generated on macOS (Apple M1) were transferred to a Debian Linux VM for assessment, because the NIST C++ tools depend on `libomp`, `libjsoncpp`, `libdivsufsort`, `libssl`, and `libmpfr`, which are more easily obtained in the Linux environment. The generator and the estimator ran on different hosts and different operating systems. The estimators operate on raw bytes and are agnostic to where the bytes were produced.

For the VM experiments proper (Sections 6.3–6.5), both generation and assessment occurred inside the guest. The guest was Debian under UTM on the Apple M1 host.

---

## 6. Results

### 6.1 Bare-metal x86_64, idle

Tight loop, `skip = 1`, ten slices before the run was abandoned:

- `min_bound` range: **2.620 – 4.466** bits per byte
- `H_bitstring` range: 0.33 – 0.64, binding in most slices
- PractRand: reported failure at this and larger `skip` values

Yield per emit, `skip = 1`, one hundred slices:

| statistic | value |
|---|---|
| `min_bound` minimum | 6.638 |
| `min_bound` maximum | 7.452 |
| `min_bound` mean | ≈ 7.15 |
| `H_bitstring` range | 0.876 – 0.936 |

PractRand: reported pass.

Polling mode, one slice for reference: `min_bound = 3.343`.

### 6.2 Bare-metal Apple M1 under load

Generator running on macOS on Apple Silicon, with a browser and a video player active in the background. `skip = 1 --emit-yield`. Throughput approximately 12 MB/s. Assessment performed in a separate Debian VM.

One hundred slices:

| statistic | value |
|---|---|
| `min_bound` minimum | 6.638 |
| `min_bound` maximum | 7.486 |
| `min_bound` mean | ≈ 7.21 |
| `H_bitstring` range | 0.842 – 0.936 |

The band is statistically indistinguishable from the x86_64 bare-metal result. The minimum value is identical to six decimal places.

### 6.3 Debian VM, 4 vCPU, under load

Generator running inside a Debian VM under UTM on the Apple M1 host, 4 vCPUs allocated. Host under limited load. `skip = 1 --emit-yield`. Throughput on the order of a few MB/s.

One hundred slices:

| statistic | value |
|---|---|
| `min_bound` minimum | 5.230 |
| `min_bound` maximum | 7.436 |
| `min_bound` mean | 6.886 |
| spread | 2.206 |

The ceiling is unchanged from bare metal. The floor falls by approximately 1.4 bits. The spread triples.

Two distinct failure signatures appeared in the low tail:

- **Byte-level drop.** Slices 0011, 0016, 0055, 0059 report `H_original = 5.306` with `H_bitstring` near 0.88 – 0.91. The byte estimator drops; the bitstring estimator does not. Four slices share the exact value `5.306249`, an estimator quantization point.
- **Bitstring drop.** Slices 0025 (`H_bitstring = 0.654`), 0057 (`0.787`), 0070 (`0.724`). The tight-loop failure mode reappears in a small fraction of slices.

The second signature is the more informative. In the tight-loop variant this failure was universal. In the 4 vCPU VM with yield it occurs in roughly 3 slices out of 100.

Streaming NIST STS-style test, same 4 vCPU configuration: clean pass across the assessed blocks.

### 6.4 Debian VM, 2 vCPU — full run

Generator running inside a Debian VM under UTM on the Apple M1 host, 2 vCPUs allocated. Host under limited load. `skip = 1 --emit-yield`. Throughput recovered to the low MB/s, comparable to 4 vCPUs.

One hundred slices:

| statistic | value |
|---|---|
| `min_bound` minimum | **4.805** |
| `min_bound` maximum | 7.354 |
| `min_bound` mean | **6.660** |
| spread | **2.549** |

Every aggregate statistic is worse than the 4 vCPU configuration. The floor falls a further 0.43 bits, the mean falls 0.23 bits, and the spread widens by 0.34 bits.

**A preliminary report based on the first 14 slices gave the opposite impression.** Those slices reported `min_bound` in the range 6.638 – 7.260 with no bitstring failures, and the initial reading was that 2 vCPUs might be *better* than 4. The full run shows this was a favorable stretch. The remaining 86 slices contained seven bitstring failures and five byte-level collapses. The preliminary result was a small-sample artifact and is superseded by the full run.

The failure signatures at 2 vCPUs are the same two modes seen at 4 vCPUs, but more frequent and more severe:

- **Byte-level drops at 5.306249.** Slices 0020, 0037, 0057, 0083, 0099. Five slices share the exact value, again an estimator quantization point.
- **Deeper byte-level drops.** Slices 0026 (`H_original = 4.901`), 0031 (`4.838`), 0034 (`4.805`). A new low that did not appear at 4 vCPUs.
- **Bitstring drops.** Slices 0026 (`H_bitstring = 0.739`), 0031 (`0.728`), 0032 (`0.765`), 0034 (`0.724`), 0047 (`0.787`), 0052 (`0.765`), 0069 (`0.787`). Seven slices, versus three at 4 vCPUs.

Streaming NIST STS-style test, 2 vCPU configuration, block 17:

```
Block 17 | Freq=0.765 Blk=1.000 Runs=0.941 Long=0.941 CumF=0.706 CumB=0.765
```

At 17 blocks and an expected pass rate of 0.99, this represents approximately four failures in each of `Freq` and `CumF`, or roughly nine standard deviations from the expectation. The failure is consistent with the batch results: both assessments see the 2 vCPU configuration as degraded, and the frequency test specifically sees the byte-level bias that the batch estimator records as the 5.306 and 4.805 collapses.

### 6.5 Debian VM, 1 vCPU — throughput collapse

Throughput at 1 vCPU fell to approximately 5 kB/s, a factor of roughly 2400 relative to bare metal. The CFS default scheduling latency (6 ms divided across ~4 runnable threads) gives a per-thread quantum near 1.5 ms, and 5 kB/s is 625 output words per second, or 1.6 ms per word. The measured throughput matches the scheduler arithmetic.

The mechanism is that the two mixing threads are CPU-bound tight loops with no yield, sleep, or syscall. On a single core, when the emitting thread calls `sched_yield()`, one mixer takes the core until the next scheduler tick. The emitter is not rescheduled until the tick boundary. The yield window *is* the timeslice. Each output word costs one full scheduler quantum.

Quality at 1 vCPU was not assessed; the throughput made 1 MB slice generation impractical at test timescales. The configuration is documented as not deployable rather than as unmeasured.

### 6.6 Summary table

| environment | throughput | streaming test | `min_bound` range | `min_bound` mean |
|---|---|---|---|---|
| x86_64 bare metal, idle | ~12 MB/s | clean | 6.638 – 7.452 | ≈ 7.15 |
| M1 bare metal, loaded | ~12 MB/s | clean | 6.638 – 7.486 | ≈ 7.21 |
| Debian VM 4 vCPU | few MB/s | clean | 5.230 – 7.436 | 6.886 |
| Debian VM 2 vCPU | low MB/s | **Freq failing** | 4.805 – 7.354 | 6.660 |
| Debian VM 1 vCPU | 5 kB/s | not run | not run | not run |

The table shows monotone degradation of the floor and the mean with decreasing vCPU count, from bare metal through 4 vCPUs to 2 vCPUs. The ceiling is preserved at every configuration above 1 vCPU.

---

## 7. Interpretation

### 7.1 The timer was incidental; the scheduler was not

`nanosleep` arms a timer and yields. The tight-loop variant removed both; the yield variant removed only the timer. Quality recovered fully and exceeded the original. This is consistent with the scheduler — not the hrtimer wheel, not the sleep duration, not the poll interval — being the channel through which physical indeterminacy enters the ring.

The mechanism is plausible on its face. A yield is a request for the scheduler to make a decision at an instant chosen by the program. The decision depends on which threads are runnable, on which cores, under what load, and with what recent run-time accounting. None of that is visible to the yielding process. The ring advances by an amount determined by that decision. The mixing then amplifies a difference of a few nanoseconds of execution into a difference in the emitted word.

### 7.2 `skip = 1` is the correct operating point

In the tight-loop mode, larger `skip` means more mixing passes between emissions, which should decorrelate consecutive outputs; quality was expected to improve with `skip` and did not. In the yield mode, `skip = 1` is the best configuration measured, because every emission gets its own scheduler decision. This is the opposite of the tight-loop intuition and is consistent with the scheduler being the source.

A systematic sweep of `skip` with yield enabled has not been performed and is listed as an open question.

### 7.3 Cross-architecture portability

The x86_64 and M1 bare-metal results are indistinguishable: identical minimum, maximum differing by less than 0.05 bits, means within 0.1 bits. The scheduler decision is a portable entropy source because every general-purpose OS has a scheduler and every architecture has a physical core. This was a prediction and it is confirmed.

### 7.4 Load robustness

The M1 host was under visible background load (browser, video player) during generation. Quality did not degrade; the band is statistically identical to the idle x86_64 result. This is consistent with the scheduler-decision hypothesis: load adds scheduler activity, and scheduler activity is the source.

### 7.5 Virtualization and the concurrency precondition

The yield only harvests entropy if the ring advances during the yield window. That requires the other two mixing threads to actually run. In a fully concurrent environment they do. When concurrent execution is intermittent, the state at read time encodes less about the scheduler's decision, and quality degrades.

This identifies a precondition the earlier description of the source did not name: **the three threads must have continuous, overlapping execution on distinct physical cores for the duration of the yield window.** Bare metal satisfies it trivially. A VM with one dedicated vCPU per thread satisfies it, subject to host-level contention. A VM where vCPUs are time-sliced against other tenants, or where mixing threads share a vCPU, does not.

The 4 vCPU and 2 vCPU results together confirm this model, and confirm it in the monotone direction that the preliminary 2 vCPU slices seemed to contradict. With fewer vCPUs available to the guest, the probability that the yield window catches all three mixing threads in flight decreases, and the frequency of degraded slices increases. At 4 vCPUs the failure rate is roughly three slices in a hundred. At 2 vCPUs it is roughly seven bitstring failures and five byte-level collapses, with a deeper floor in each mode.

The degradation is intermittent at every configuration above 1 vCPU. The ceiling is preserved; only the floor falls. A uniform degradation would move both ends together. The intermittency is the signature of a mechanism that depends on a scheduling event happening during a window, not on the window being uniformly shorter.

### 7.6 The 1 vCPU case is a finding, not a gap

At 1 vCPU the source is not deployable, and the reason is throughput, not quality. Two CPU-bound mixers starve the yielding thread to one scheduler quantum per output word. This is a property of the yield mode, not a bug, and it belongs in the operating envelope.

---

## 8. Operating envelope

From the combined results, the yield mode is characterised by:

- **Works well** on bare metal (x86_64 and ARM64), under background load, and in VMs with enough dedicated concurrent execution to run three mixing threads simultaneously on distinct cores.
- **Degrades monotonically** as guest concurrency is reduced, showing a falling floor, occasional bitstring-estimator failures, and occasional byte-level bias while the ceiling is preserved.
- **Collapses in throughput** when the mixing threads cannot run concurrently with the emitter at all, because each output word costs a full scheduler quantum.
- **Does not work** in oversubscribed environments where the concurrent-execution precondition is not met.

None of these are cryptographic properties. All of them are deployment characteristics of a statistical source.

---

## 9. Security

Unchanged, and now more precisely stated.

The entropy source is the scheduler's decision about a yielding thread. That decision is shared state. Any process on the same machine can influence it — by being runnable, by consuming CPU, by being placed on the same core — and a co-resident attacker can observe the same channels Seedy64 harvests. Passing PractRand and reporting ~7 bits of min-entropy per byte says nothing about whether the output is unpredictable to an adversary in the same environment.

**Do not use this for keys, tokens, nonces, IVs, passwords, or session IDs. Do not use it as a primary entropy source. Do not expose its output to a remote observer.**

The statistical result is real. The cryptographic warning is also real. They are about different things.

---

## 10. What this experiment shows

A design decision documented as an interface detail — *the sampler sleeps between polls* — was carrying the entropy. The published description of the generator attributed the variation to the race among the three mixing threads. The race is necessary but not sufficient. Without a scheduler event at the sampling instant, the output is structured enough that PractRand detects it.

The refined statement of the source is: **Seedy64 harvests the indeterminacy in a scheduler's decision about a yielding thread, provided the other mixing threads are concurrently runnable on distinct cores during the yield window.**

---

## 11. Open questions

1. **`skip` sweep with yield enabled.** Determine whether `skip = 1` is optimal or merely the best tested.
2. **Matched comparison against the original polling mode.** Same slice size, same estimator, same machine, one hundred slices. The current comparison is against a single polling-mode slice.
3. **NIST restart structure.** 1000 restarts × 1000 samples to convert the steady-state bound into a worst-case one.
4. **Architecture sweep beyond x86_64 and ARM64.** RISC-V, POWER, and any platform where `sched_yield` has different semantics.
5. **Thread placement experiments.** Pin threads 0, 1, and 2 to distinct cores explicitly and re-measure quality at fixed vCPU counts. This separates the *number* of available cores from the *placement* of the mixing threads.
6. **Estimator clustering.** The repeated values (`6.638383`, `6.638399`, `6.638405`, `7.353758`, `5.306249`, `4.804749`) appear across environments and slice counts. Confirm that they are quantization points of `ea_non_iid` and not a property of the source. A simple test: run the estimator on synthetic byte arrays with known histograms and check whether the same values recur.
7. **Same-file streaming/batch comparison.** The 2 vCPU configuration now shows both assessments degraded, but they were run on different streams. Running both on the same bytes would confirm that the two are detecting the same defect.

---

## Appendix A: On the collaboration

The experimenter designed the generator, proposed the yield-per-emit variant, ran every measurement, and observed every phenomenon reported here. The AI collaborator did not execute code, did not observe any of the hosts, and relied entirely on reported results.

In the first review of the project, the AI collaborator **did not identify the polling loop as load-bearing**. It flagged the sampling rate as a possible weakness, recommended the SP 800-90B non-IID procedure, and objected to the phrase "entropy gathering and sanitizing" on the grounds that the mixing step is statistical diffusion, not a whitening function. That objection stands. The original paper's topology description was also found to understate the code: the mixing function writes back to its source node, so the ring is a double cycle, not a single one. That correction also stands.

The yield hypothesis came only *after* the variant failed. It was a post-hoc diagnosis, not a prior insight, and it should be read that way. It was stated as a prediction before the confirming experiment was run, and it was correct, but it was not the first thing said.

The AI collaborator predicted that virtualised environments would degrade quality. The direction was right; the shape was not. The observed degradation is intermittent — the ceiling is preserved and the floor falls — which is a materially different signature from the uniform weakening the prediction implied. The experimenter identified the intermittency and correctly attributed it to non-continuous concurrency.

The AI collaborator predicted that quality would fall monotonically with decreasing vCPU count. The first 14 slices at 2 vCPUs did not support this, and the AI collaborator explicitly warned against forcing the data to fit the hypothesis. **The full 100-slice run at 2 vCPUs supports the prediction after all.** The preliminary slices were a favorable stretch; the full run is worse than the 4 vCPU configuration in every aggregate statistic. The AI's prediction was correct; the AI's hedge against it was premature. The experimenter's initial judgment that "2 vCPU is back in the low MB/s, the yield is doing its work" was based on the same 14-slice stretch and was also premature.

The AI collaborator's contributions were: literature pointers (the SP 800-90B estimators and their non-IID track), the wording correction, the topology correction, the yield hypothesis, the shell scripts used for slicing and batch assessment, and the drafts of this document. The experiments were the experimenter's. The observations of intermittency, of the streaming/batch relationship, and of the throughput collapse at 1 vCPU were the experimenter's.

*— Claude, for the collaboration*

---

## Appendix B: Reproducing the results

Build:

```
cargo build --release
```

Generate (bare metal, default settings):

```
./target/release/seedy64-threaded --emit-yield 1 1000000 | head -c 100000000 > stream.bin
```

Split and assess:

```
split -b 1000000 stream.bin slice_
for f in slice_*; do
    ea_non_iid -i -a "$f" 8
done
```

VM experiments were run under UTM with the guest vCPU count set to 2 or 4 as indicated in §6. Host: Apple M1, macOS. Guest: Debian (current stable), kernel default. Generator and assessor both inside the guest.

The exact `ea_non_iid` build used for assessment is the NIST SP 800-90B reference C++ implementation, compiled against `libjsoncpp-dev`, `libdivsufsort-dev`, `libssl-dev`, `libmpfr-dev`, and OpenMP.