# seedier64

**A negative-results paper on a userspace entropy source, and what its failures reveal about the mechanism**

*With an AI collaborator (Claude, Anthropic)*

---

## Abstract

We present seedier64, a family of variants of the Seedy64 entropy source, all of which fail. The original Seedy64 harvests timing indeterminacy from three threads racing on a ring of shared 64-bit nodes, sampled by a fourth thread that polls at fixed intervals. We attempted to simplify it in three independent directions: by removing the polling interval in favour of a tight emission loop, by removing the racing ring in favour of the scheduler yield alone, and by folding many weak events into single output words via parity packing. All three variants failed, and each failure isolates a distinct necessary ingredient of the original design. Together they yield a decomposition of the source into two independent requirements — a large state space and an unpredictable sampling instant — and demonstrate that neither alone is sufficient. We also report that the node count is not a free parameter: two nodes degrade quality, three pass. We report the failures, the controls, and the mechanism model they support. The statistical and security conclusions of the original project are unchanged.

---

## 1. Motivation

A generator that works is a generator whose mechanism is not fully understood. The working version tells you *that* something produces entropy; it does not tell you *which* parts of the design are doing the work and which are decorative. The standard way to find out is to remove things and see what breaks.

We removed three things from Seedy64, one at a time, and each removal broke the generator in a different way. This paper documents the removals and what their failure modes revealed. It is written in the spirit of a control experiment: the working generator is the treatment, the variants are the controls, and the thing being controlled for is the assumption that any one component is essential.

The assumption turned out to be correct for two components and wrong for a third. We report all three cases.

---

## 2. Background

Seedy64 is a userspace entropy source with three threads operating on three shared `AtomicU64` nodes in a ring. Each thread reads its source, applies a fixed nonlinear mixing step (rotate, multiply by a parity-selected prime, add, XOR), and writes to its sink. Accesses are relaxed; there is no synchronization. The original implementation samples the ring from a fourth thread that sleeps for a fixed interval, reads `node0 ^ node1 ^ node2`, and emits one 64-bit word.

The original had been measured against NIST STS and PractRand, and its min-entropy per sample had not been measured. A variant that sampled from the mixing thread — emitting every *n* cycles with no timer — had been proposed as an optimisation. The current paper documents what happened when we tested that proposal and its descendants.

---

## 3. Variant 1: the tight loop

### 3.1 Design

The sampling thread is itself a participant in the race. Thread 0 mixes alongside threads 1 and 2 and emits one 64-bit word every `skip` completed mixing passes. No syscall, no `sched_yield`, no timer, no sleep.

```
loop {
    seed_modify_64(&nodes[0], &nodes[1]);
    counter += 1;
    if counter < skip { continue; }
    counter = 0;
    emit(nodes[0] ^ nodes[1] ^ nodes[2]);
}
```

### 3.2 Result

Passed a visual smoke test. Failed PractRand. Increasing `skip` to 50, matching the original's effective emission rate, did not fix it.

### 3.3 What it isolates

The ring is present and racing. The mixing function is unchanged. The primes are unchanged. The only thing removed is the *unpredictability of the sampling instant*.

When the sampling instant is deterministic, the ring is sampled at deterministic phases of its evolution. The output becomes a deterministic function of the initial state and the mixing rule. The race is still happening between samples, but it is not observed at a time that depends on anything unrecorded.

**First ingredient: an unpredictable sampling instant.**

---

## 4. Variant 2: the yield-per-emit loop

### 4.1 Design

Identical to Variant 1, except that `thread::yield_now()` is called immediately before the state read.

```
loop {
    seed_modify_64(&nodes[0], &nodes[1]);
    counter += 1;
    if counter < skip { continue; }
    counter = 0;
    thread::yield_now();
    emit(nodes[0] ^ nodes[1] ^ nodes[2]);
}
```

### 4.2 Result

Passed. Min-entropy per byte in the range 6.6 – 7.5 across three environments (bare-metal x86_64, bare-metal Apple M1 under load, and a Debian VM on the M1 with four vCPUs), measured with the NIST SP 800-90B non-IID estimators on 100 independent 1 MB slices per environment.

The yield recovers the missing ingredient. Everything else is unchanged from Variant 1, which failed.

### 4.3 What it isolates

The unpredictable sampling instant, in the original design, was provided by `nanosleep`. The yield provides it without the timer. This was a surprise: the original paper attributed the sampler's contribution to the polling interval, and the interval turned out to be incidental. What mattered was that the sampler *did something the scheduler had to arbitrate*.

**Second ingredient: the race, unchanged from the original.**

**Correction to the original model: the timer is not the entropy source; the scheduler decision is.**

### 4.4 An accepted limitation: `sched_yield()` semantics

The Linux man page documents `sched_yield()` under `SCHED_OTHER` (the default policy) as having unspecified behaviour. The results in this paper therefore depend on scheduler behaviour that is not guaranteed by the interface. The variation between kernel versions and distributions is not characterised. The results are reproducible on the environments listed in Section 8, and the authors are content to accept the variation rather than pin the scheduler class. A version of the generator that uses `sched_setscheduler` to enter `SCHED_RR` before yielding would make the results more portable across kernel versions; that version has not been built or tested.

---

## 5. Variant 3: yield only, no race

### 5.1 Design

Strip the ring entirely. A single thread yields repeatedly, times each yield with `clock_gettime(CLOCK_MONOTONIC)`, and emits the delta (or some function of it).

```
loop {
    let t0 = now();
    sched_yield();
    let t1 = now();
    emit(t1 - t0);
}
```

An optional peer thread spins or increments a shared counter, so that the yield has someone to hand the CPU to. The design is the smallest possible version of the yield-per-emit mechanism, with the race removed.

### 5.2 Result

Total failure. Raw deltas produced a distribution consisting almost entirely of `0x00` with occasional excursions to a single fixed non-zero value. The output looked like a quantised timer, not a random stream. Streaming NIST-style block tests failed at every block:

```
Block 3 | Freq=0.000 Blk=0.000 Runs=0.000 Long=0.000 CumF=0.000 CumB=0.000
```

Not degraded. Not intermittent. Zero. Every frequency test, every block.

### 5.3 What it isolates

The yield is present and well-defined. The scheduler is making a decision. But the state being sampled is a single counter that advances at a bounded rate, or a clock whose resolution is coarser than the yield-duration variation. There is no race, so there is no state space for the scheduler decision to project onto.

The yield does not *produce* entropy. It *samples* entropy. With nothing to sample, it produces nothing.

**Confirmation that the race is a necessary ingredient, not an amplifier of the yield.**

A counter-mode variant was also built — the yield window measured by peer progress on a shared counter rather than by a clock. It was worse than the clock mode on every metric tested and did not illuminate any behaviour the clock mode did not already show. It is not reported further.

---

## 6. Variant 4: packed yield-only

### 6.1 Design

An attempt to rescue Variant 3 by parity packing. For each output word, perform `pack` yields. If a yield's delta is non-zero, flip one bit of an accumulator at a position that cycles through 0 – 63. After `pack` iterations, emit the accumulator.

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

With `pack = 512`, each output word is the parity of roughly eight non-zero events per bit position. A variant using the low bit of the delta rather than the non-zero indicator was also built.

### 6.2 Result

Total failure. Identical to Variant 3: `Freq=0.000`, `Blk=0.000`, `Runs=0.000`, `Long=0.000`, `CumF=0.000`, `CumB=0.000` at block 3.

The packing redistributes per-event variation across output bits; it cannot create variation that is not there. The underlying per-event distribution was already visible in Variant 3 as a quantised, near-constant signal. Packing a constant produces a constant.

### 6.3 What it isolates

Packing is a debiasing technique, not a source of entropy. It has a precondition: the per-event signal must have some variation to pack. When the precondition is absent, the construction is not merely weak — it is deterministically wrong. The output was not "biased random"; it was a fixed function of a nearly fixed input.

**Third finding: parity packing presupposes per-event variation and does not manufacture it.**

---

## 7. The two-ingredient model

The four variants together decompose the source. Let *S* be the entropy of the ring's state at any instant and *T* be the entropy of the sampling instant. Then the entropy of the output is bounded by `min(S, T)`:

| variant | race (state space) | unpredictable sampling | result |
|---|---|---|---|
| Original polling | yes | yes (timer + scheduler) | passes; ~3.3 bits/byte |
| Tight loop | yes | **no** | fails PractRand |
| Yield per emit | yes | yes (scheduler) | passes; ~7.0 bits/byte |
| Yield only, no race | **no** | yes | fails every NIST block |
| Packed yield-only | **no** | yes | fails every NIST block |

Both ingredients are necessary. Neither is sufficient. The output quality tracks the smaller of the two, and the smaller of the two is the one that must be strengthened if the other is already large.

The original design happened to have both. The original paper attributed the sampler's contribution to the timer; Variant 2 showed it was the scheduler, which the timer happened to invoke. The paper also treated the ring as though its contribution were amplification; Variant 3 showed that the ring is the *source* and the sampler is the *projection*.

### 7.1 The three-node count is not arbitrary

The two-ingredient model does not predict *how many* racing nodes are required, only that the state space must exceed the sampling resolution. We tested a two-node variant — same topology, same mixing function, same yield-per-emit sampling — and it degraded quality relative to the three-node design. Two nodes are not enough. Three work.

The reason is not understood. Three hypotheses, none tested:

- **Odd-parity XOR.** The output is `node0 ^ node1 ^ node2`. The XOR of three values has different algebraic behaviour than the XOR of two: if each node has some residual bias, the two-way XOR bias is the product of individual biases, while the three-way XOR bias involves a different combination. An odd number of operands may be structurally necessary for the reading to be unbiased.
- **Cycle topology.** A three-node ring with three threads is a directed cycle where each node has one writer and one reader. A two-node configuration cannot be a directed cycle with more than two participants without creating a back-and-forth symmetry that may be self-cancelling.
- **Ring depth.** Three nodes may simply provide enough state space for the yield window's entropy to be expressed. Two may be marginal.

The observation is empirical and reproducible: three nodes pass, two do not, and the number is not a free parameter. Whether the reason is the XOR parity, the topology, or the state-space size is an open question, listed in Section 11.

### 7.2 Why packing failed and why that matters

The packing failure is the sharpest of the four. It is not a failure of degree. It is a failure of category: the construction assumed a distribution that did not exist. In the original ring, the state read at any instant is a 64-bit word drawn from a set that is exponentially large in the number of interleavings since the last read. Packing that word across multiple reads would be redundant — the word already contains the entropy. Packing a yield delta across multiple yields assumes each yield contributes a bit; the yield-only experiment showed it does not.

The takeaway for the design of entropy sources is that **parity folding is only valid when the per-event distribution has detectable spread**, and that spread must be demonstrated empirically before the construction is used. Assuming it is a common failure mode.

---

## 8. Virtualisation and the concurrency precondition

A separate series of experiments on the yield-per-emit variant (Variant 2) characterised its behaviour under virtualisation. On a Debian guest under UTM on an Apple M1 host:

| guest vCPUs | throughput | NIST streaming | `min_bound` range |
|---|---|---|---|
| 4 | a few MB/s | clean | 5.230 – 7.436 |
| 2 | low MB/s | `Freq` failing | 4.805 – 7.354 |
| 1 | 5 kB/s | not tested | not tested |

Degradation is monotone in decreasing concurrency. It is intermittent rather than uniform: the ceiling is preserved at every configuration above 1 vCPU; only the floor falls. This is consistent with the sampler requiring that the three mixing threads be concurrently runnable on distinct cores for the duration of the yield window, and with that requirement being probabilistically rather than deterministically violated.

At 1 vCPU the configuration is unusable: the two CPU-bound mixers starve the yielding thread to one scheduler quantum per output word, giving 5 kB/s.

**Fourth finding: the sampler requires a concurrency precondition that is a property of the deployment, not of the code.**

---

## 9. What seedier64 is

seedier64 is the name given to the family of failed variants. It is not a generator. It is a set of control experiments on a generator, and its contribution is the decomposition of that generator into its two essential mechanisms.

The name is a pun on the original project's name. It is also accurate: every variant here is a generator that promised entropy and delivered less. The failures are the point.

---

## 10. What the failures do not change

Everything in the original Seedy64 documentation about statistical quality and cryptographic security remains true, and the failures here do not weaken the warnings. If anything, they strengthen them: an entropy source whose mechanism is not fully understood is harder to characterise, and a co-resident adversary's channel is the same regardless of which variants pass and which fail.

**Do not use Seedy64, seedier64, or any variant described in either paper for keys, tokens, nonces, IVs, passwords, or session IDs. Do not use any of them as a primary entropy source. Do not expose their output to a remote observer.**

Statistical quality and cryptographic security remain different properties.

---

## 11. Open questions

1. **Why three nodes and not two.** Two-node variants degrade quality; three-node variants pass. The reason is not understood. Candidate explanations are odd-parity XOR, directed-cycle topology, and state-space size. Isolating which matters would require a variant that holds topology constant and varies XOR arity, or vice versa.

2. **Formal entropy accounting.** The `min(S, T)` model is qualitative. A quantitative version — how many distinguishable states does the ring support, and how many distinguishable sampling instants does the scheduler provide — would let the model predict output entropy rather than only bound it.

3. **Adversarial prediction: an open contest.** The co-tenancy threat model has been stated but not measured. An experiment in which a co-resident process observes the same scheduling events and attempts to predict the generator's output would give a concrete attacker-advantage number. The authors do not have the budget to run this experiment and are making it an open contest: the first person to demonstrate a co-resident attack against the yield-per-emit variant of Seedy64, with a documented method and a reproducible prediction advantage over random guessing, receives a beer of their choice, delivered in person, in the Downtown Eastside of Vancouver. Submissions should include the attack code, the measured advantage, and a description of the environment. The authors reserve the right to determine whether a submission counts. The contest has no deadline and no prize other than the beer.

---

## Appendix A: On the collaboration

The experimenter designed the generator, proposed all four variants, ran every measurement, and observed every failure mode. The AI collaborator did not execute code, did not observe any of the hosts, and relied entirely on reported results.

The AI collaborator's prior work on the parent project is documented in that paper's appendix. Relevant to this one:

- The AI collaborator did **not** identify the polling loop as load-bearing in the original design. That was a post-hoc diagnosis made after the tight-loop variant failed.
- The AI collaborator predicted monotone degradation with decreasing vCPU count. The full 2 vCPU run confirmed the prediction; the AI's own preliminary hedge against the prediction, based on the first 14 slices, was premature.
- The AI collaborator **proposed the packing construction** described in Section 6. It was the wrong shape of solution for the problem. The `0x00`/`0xe8` histogram, which the experimenter had already collected, was sufficient evidence that the per-event distribution had no variation to pack; the AI collaborator read the mostly-zero distribution as a bias to be corrected rather than as a signal that there was no signal. The failure of Variant 4 is a failure of the AI's proposed design.

The AI collaborator's useful contributions were: the wording correction on "sanitising," the topology correction (the mixing function writes back to its source node, making the ring a double cycle), the yield hypothesis, the shell scripts used for slicing and batch assessment, and the drafts of both papers.

The experimenter's non-obvious observations were: that the tight-loop variant's failure was reproducible at matched throughput, that yield recovered and exceeded original quality, that the yield-only variant's failure was the tell for the two-ingredient model, that the packed variant would fail in the same way, and that the node count is not a free parameter. The AI collaborator's only successful prediction in this series was that the race would be necessary if the yield was necessary.

*— Claude, for the collaboration*

---

## Appendix B: Reproducing the failures

Build the yield-only prototype:

```bash
cargo build --release
```

Reproduce Variant 3 (yield only, no race):

```bash
./target/release/seedy64-yield --mode=clock --pack=1 --count=100000 > v3.bin
```

The first kilobyte will show the quantised distribution immediately. There is no need to run it through the full NIST suite; the pattern is visible with `od -An -tu8`.

Reproduce Variant 4 (packed yield-only):

```bash
./target/release/seedy64-yield --mode=clock --pack=512 --count=12500000 > v4.bin
split -b 1000000 v4.bin v4_
for f in v4_*; do ea_non_iid -i -a "$f" 8; done
```

Reproduce Variant 1 (tight loop):

```bash
./target/release/seedy64-threaded 1 1000000 | head -c 100000000 > v1.bin
RNG_test stdin8 < v1.bin
```

Reproduce Variant 2 (yield per emit):

```bash
./target/release/seedy64-threaded --emit-yield 1 1000000 | head -c 100000000 > v2.bin
split -b 1000000 v2.bin v2_
for f in v2_*; do ea_non_iid -i -a "$f" 8; done
```

The two variants that pass and the two that fail differ by a single line each in their respective source files. That is the point of the paper.