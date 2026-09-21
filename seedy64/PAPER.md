# Seedy64

A nondeterministic random bit generator that harvests entropy from thread
interleaving over shared memory. It is an experimental tool for games,
simulations, and demonstrations. **It is not cryptographically secure. Do not
use it for keys, tokens, nonces, session IDs, or anything an attacker can
observe.** See the warning at the end.

## The idea

Think of DNA and environment. Neither one alone determines an organism; the
organism is what happens when the two interact. The same is true here.

The software alone is deterministic. Three threads reading and writing three
shared 64-bit words according to a fixed rule will, in a quiet machine with a
predictable scheduler, produce a predictable stream. The hardware alone is
also deterministic at the level we can see: a CPU executing a fixed program
on a fixed memory image does the same thing every time.

What is not deterministic is the overlap. The store buffer decides when a
write becomes visible. The cache coherence protocol decides which core sees
what and when. The OS scheduler decides which thread runs on which core for
how long. Each of these depends on physical details — voltage on the memory
rail, thermal state, interrupt timing — that no one controls and no one
records. When three threads race on shared state, those microscopic
uncertainties get amplified into the output.

Seedy64 does not contain a chaotic map. It does not implement a cipher. It
sets up a small ring of racing threads and lets the machine's hidden state
supply the variation. The "chaos" is a property of the whole system —
software plus hardware plus environment — not of the code by itself.

## How it works

Three threads, three shared `u64` nodes arranged in a ring:

    Thread 0:  node0 -> node1
    Thread 1:  node1 -> node2
    Thread 2:  node2 -> node0

Each thread reads its source node, applies a nonlinear mixing step (rotate,
look up a prime by a parity bit, multiply, add), and writes to its sink. All
accesses are relaxed atomic loads and stores. There is no synchronization,
no lock, no barrier. The threads genuinely race.

The main thread polls `node0 ^ node1 ^ node2` at fixed intervals and emits
one 64-bit word per poll.

Two modes are provided:

- **Streaming**: start the threads once, poll repeatedly. Fast.
- **Reset-per-output**: start the threads, take one sample, tear down.
  Slower, but each output is independent of every other — there is no
  carried state between samples.

## What we have measured

All results below are from the reset-per-output mode unless noted.

**NIST STS (full suite, 199 tests per block, 10 blocks).**
10 failures out of 1990 test runs, for a pass rate of **99.50%**. The
expected failure rate at the 1% significance threshold is 1%. All ten
failures were instances of `non_overlapping_template`, which is the test
with the largest number of instantiations per block (~148 per block, 1480
total). Within that test the failure rate was 10/1480 ≈ 0.68%, below the
threshold. No other test produced a single failure across all ten blocks.

**32-bit coupon collector.** Two to the power thirty-two 32-bit outputs.
Fraction of distinct values seen matched the theoretical curve
`1 - exp(-N/n)` to seven to nine significant figures at every stage. Final
coverage 0.9999999998, i.e. one 32-bit value unobserved out of 4.29 billion,
consistent with the expected Poisson tail.

**PractRand.** Passed to 256 GB at time of writing. Longer runs are in
progress.

## What we have not measured

**Min-entropy per sample.** This is the number that determines how much
useful randomness each output actually contains, and it has not been
measured. Until it is, any claim about how many bits per sample the source
provides is a guess. The nominal ceiling is high — three threads at GHz
speeds racing on memory is a lot of potential coin flips — but the real
number will be lower, and we do not yet know by how much. Running the
NIST SP 800-90B estimators on a few million raw samples is the next step.

**Adversarial robustness.** See below.

**Behavior across environments.** Idle, loaded, pinned, SMT on and off,
bare metal and VM. The generator is expected to degrade in virtualized
environments where the scheduler and memory subsystem are mediated by a
hypervisor. We have not yet quantified that degradation.

## What it is for

- Games and simulations where a fast, non-repeating stream is useful.
- Procedural generation.
- Demonstrations of how microscopic physical indeterminacy gets amplified
  into macroscopic output.
- Last-resort seeding of a proper DRBG when no other source is available
  and the environment is trusted.

## What it is not for

**Anything security related. Ever.**

The entropy is real, but it is **shared**. Every source of variation that
Seedy64 harvests — store-buffer timing, cache-coherence traffic, scheduler
decisions, memory-controller queue depth — is also visible to any other
process running on the same physical machine. An attacker who can run code
on the same host can, in principle, observe the same physical channels that
Seedy64 is riding and narrow the distribution of its output. This is not a
theoretical concern: it is the standard threat model for cloud co-tenancy,
shared CI runners, and any multi-tenant system.

Concretely, this means:

- **Do not generate keys, tokens, nonces, IVs, passwords, or session IDs
  with Seedy64.** Use the operating system's CSPRNG (getrandom, /dev/urandom,
  BCryptGenRandom, SecRandomCopyBytes) or a standardized DRBG seeded from it.
- **Do not use Seedy64 as a primary entropy source** in any system that has
  a validated hardware or OS source available. It is a fallback of last
  resort, not a replacement.
- **Do not expose Seedy64 output to a remote observer** and assume it is
  unpredictable just because it looks random. Statistical quality and
  cryptographic security are different properties, and Seedy64 has only the
  first.

The project warns against cryptographic use because the design cannot
support it. No amount of statistical testing changes that.

## Relationship to other generators

Seedy64 is not a PRNG in the usual sense. A PRNG has a finite state and a
deterministic transition; given the state, the future is fixed. Seedy64 has
no fixed state — the relevant state includes the hardware's hidden
micro-state, which is not accessible to the program. It is closer in spirit
to a physical entropy source than to a PRNG, even though it is implemented
entirely in software.

For parallel workloads that need reproducibility, jump-ahead, or a proven
period, use a counter-based PRNG (SplitMix64, PCG, Philox). Seedy64 is for
the cases where those properties do not matter and a small amount of
genuine physical variation is useful.

## Building and running

    cargo build --release
    ./target/release/seedy64 > stream.bin

The binary writes raw 64-bit words to stdout. 
There is no formatting.

## Status

Experimental. Interfaces may change. Test results are reported above as
they stand, and the min-entropy measurement is the main open item. If you
have a budget for it, run it.
