# Where does novelty come from?

A note on determinism, indeterminacy, and what a small concurrency toy
might be trying to say.

## The question

If the universe is deterministic, nothing new ever happens. Every event is
the inevitable consequence of the state before it. "Novelty" would then be
an illusion — a word we use for outcomes we did not bother to predict.

If the universe is not deterministic, then somewhere there is a source of
genuine unpredictability, and everything downstream inherits it. Where is
that source, and how does it reach us?

Most of the time we do not have to answer this question. We act as if the
future is open, and it mostly works. But the question comes back whenever
we build something that is supposed to be unpredictable — a random number
generator, a cryptographic key, a simulation that must not repeat itself.
At that point we have to decide what kind of unpredictability we are
actually asking for.

## Two kinds of unpredictability

It helps to separate two things that are often conflated.

**Epistemic unpredictability** is unpredictability from a particular point
of view. A deterministic system can be epistemically unpredictable if the
observer lacks access to enough of its state. The weather is the classic
example: the equations are known, the initial conditions are not, and the
sensitivity to those conditions is so extreme that prediction beyond a
couple of weeks is hopeless. Nothing about the weather is metaphysically
open; it is just closed to us.

**Ontic unpredictability** is unpredictability in principle. If quantum
mechanics is correct, the outcome of a radioactive decay is not determined
by anything in the prior state of the universe, even in principle. Nothing
about the future is fixed at that moment.

Most practical systems that we call "random" are epistemically
unpredictable. The OS entropy pool is seeded from physical events that no
one records. A PRNG seeded from that pool is unpredictable because the seed
is unknown. Chaos-based generators are unpredictable because the
trajectory diverges faster than we can measure it.

Seedy64 is epistemically unpredictable. Its output depends on the exact
interleaving of three threads racing on shared memory, and that interleaving
depends on physical details — voltage, thermal state, interrupt timing —
that are not recorded anywhere. Whether those details are themselves
deterministic at a deeper level is not a question the generator can answer.
From its own point of view, they are not accessible, and that is enough to
make the output unpredictable to any observer who is not also reading the
same physical channels.

## DNA and environment

The relationship between software and hardware in Seedy64 is like the
relationship between DNA and environment in a living thing.

DNA alone does not determine an organism. Two identical genomes raised in
different environments produce different phenotypes. The genome constrains
the space of possible outcomes, but the actual outcome is the result of an
interaction that includes the environment in a way that cannot be factored
out.

Environment alone does not determine an organism either. The same soil,
the same sunlight, the same temperature produce different organisms from
different genomes.

The organism is what happens when both are present and interacting. It is
not reducible to either one. It is, in a specific sense, novel — not
because it violates any physical law, but because it is not contained in
either input considered by itself.

Seedy64 has the same structure. The program alone is deterministic. The
hardware alone, considered as a fixed machine executing a fixed program on
a fixed memory image, is also deterministic. But the program does not
determine its own execution: it races on shared state without
synchronization, and the exact order of operations depends on the machine.
And the machine does not determine the program's behavior either: the same
machine running a different program produces different output. The output
is what happens when both are present and interacting.

Whether this counts as genuine novelty depends on what you think novelty
is. If you are an ontologist, probably not: the universe is one big
deterministic (or quantum-random) system, and Seedy64 is just a corner of
it doing what it does. If you are an epistemologist, yes: the output is not
predictable from the inputs that the program has access to, and in that
sense it is new.

The honest position is probably that both are true and the distinction is
about what we mean by "new." Seedy64 is a small, working example of how
the question can be posed sharply, which is more than most philosophical
discussions of novelty manage.

## Determinism, indeterminacy, and the amplification question

There is a further question hiding behind the DNA analogy: even granting
that the environment contains microscopic indeterminacy, how does that
indeterminacy reach the macroscopic output?

In a chaotic system, the answer is exponential divergence. A difference
of one part in 10^15 in the initial conditions grows by a factor of e
every Lyapunov time, and after a few dozen Lyapunov times it has become a
difference of order unity. The microscopic becomes macroscopic through
amplification.

Seedy64 does not implement a chaotic map in the mathematical sense. But it
does have an amplification mechanism. A single store-buffer decision — one
write becoming visible to another core a few nanoseconds earlier or
later — changes the value that one thread reads next, which changes what
it writes, which changes what the next thread reads, and so on around the
ring. Over 64 iterations of nonlinear mixing per thread, the effect of
that single timing difference is spread across the entire output word. The
amplification is not the exponential divergence of a smooth dynamical
system; it is the combinatorial explosion of a discrete feedback loop.

Both mechanisms have the same effect. A difference too small to observe
directly becomes a difference in the output that is large enough to be
visible. This is the point at which microscopic indeterminacy — epistemic
or ontic — becomes macroscopic novelty.

## What Seedy64 demonstrates

Three things, if the empirical results hold up:

1. **A software system can harvest real physical indeterminacy without
   access to any dedicated hardware source.** The variation is already
   present in the timing of ordinary memory operations. It does not need
   to be added.

2. **The output can be statistically clean without being cryptographically
   secure.** These are different properties, and the project is careful
   not to confuse them. Seedy64 passes the NIST STS suite and multi-
   hundred-GB PractRand runs. That says nothing about whether its output
   is unpredictable to a co-resident adversary, which it is not, because
   the adversary can observe the same channels.

3. **The interesting property is a property of the whole system, not of
   the code.** Two identical binaries on two identical machines with two
   identical starting states will produce different streams, because the
   machines are not identical in the ways that matter. This is not a bug
   to be fixed; it is the point.

## Open questions

- **Is the novelty real or epistemic?** The generator cannot answer this.
  Whatever your metaphysics, the empirical behavior is the same.

- **Does the amplification survive across different hardware?** The
  results reported in the README suggest yes, at least for ARM and x86.
  Whether it survives on a fully synchronous chip with no memory jitter —
  a hypothetical one — is unknown.

- **What does this mean for the concept of a "random number generator"?**
  A traditional PRNG is a deterministic function from a seed to a stream.
  Seedy64 is a function from an unmeasured physical environment to a
  stream. It is not clear that the same word should apply to both.

- **If Seedy64 counts as novel, what else does?** Every physical process
  is, in this sense, novel. That may be a reductio, or it may be the
  correct answer. The philosophical literature has not settled it.

These are not questions the project is trying to answer. They are
questions the project makes it possible to ask clearly. That is enough.

## A closing caveat

It is tempting, when thinking about novelty and indeterminacy, to conclude
that a system which produces unpredictable output must be a good source
for cryptography. This is wrong, and the project says so explicitly
elsewhere. Statistical novelty and cryptographic security are different
properties, and Seedy64 has only the first. A generator can be novel in
every philosophical sense and still be trivially predictable to an
adversary who shares the same physical environment.

The philosophy here is genuine. The security warning is genuine too. They
are not in tension; they are about different things.