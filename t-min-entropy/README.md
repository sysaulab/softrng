# `t-min-entropy` — a streaming min-entropy assessor

A thin wrapper around the NIST SP 800-90B estimators (`ea_non_iid`, `ea_iid`) that slices a stream into fixed-size blocks, runs the reference C++ implementation on each slice, and reports the conservative per-slice bound. This README is written from measured behaviour of the tool on real data, not from the SP 800-90B specification alone.

---

## What this tool actually measures

`t-min-entropy` reports `min(H_original, 8 × H_bitstring)` in **bits per byte**, one value per slice. That number is a **lower bound on min-entropy under the assumption implied by the chosen estimator** — non-IID for `ea_non_iid`, IID for `ea_iid`. It is not the true min-entropy of the source, and it is not a pass/fail verdict. It is the most pessimistic estimate the estimator can produce given the slice.

The non-IID estimators are built adversarially. They include prediction estimators (MultiMCW, LZ78Y, Lag, MultiMMC) that will find *any* exploitable structure in the data, including finite-sample structure that is not present in the source. This makes the tool excellent at detecting a source that is genuinely broken, and poor at distinguishing between two sources that are both good.

The IID estimators are less conservative but require the source to pass a bank of ten IID-validation tests before a number is reported. They have headroom above the non-IID ceiling and are the correct instrument when the non-IID track has saturated.

**Both tracks have a ceiling and a floor that belong to the tool, not to the source.** Understanding where those limits sit is the difference between a useful measurement and a misleading one.

---

## The reference envelope: `/dev/urandom`

Always run `/dev/urandom` through the identical pipeline as a control. The reference numbers below were measured on the same assessor, with the same helper invocation, on 1 MB slices.

### `ea_non_iid`, 100 × 1 MB

| statistic | 1 MB slices | 10 MB slices |
|---|---|---|
| `min_bound` typical | ~7.10 | ~7.47 |
| `min_bound` maximum | 7.486 | ~7.54 |
| `min_bound` minimum | 6.116 | — |
| spread (max − min) | 1.370 | narrower |

### `ea_iid`, 100 × 1 MB

| statistic | value |
|---|---|
| `min_bound` typical | ~7.88 |
| `min_bound` maximum | ~7.89 |
| `min_bound` minimum | ~7.86 |
| spread (max − min) | ~0.03 |

Four things to internalise from these tables:

1. **The non-IID ceiling is the estimator's.** A source that is uniform by construction does not score higher than ~7.47 typical / ~7.54 max on 1–10 MB slices. You cannot exceed this by generating better bytes. You can only fall below it.

2. **The IID ceiling is also the estimator's.** On 1 MB slices, uniform data reports ~7.88. The gap from 7.88 to 8.0 is the MCV estimator's finite-sample bias at n = 10⁶, not a property of the source. The IID track buys you ~0.4 bits of headroom over the non-IID track, but it does not reach 8.0 either.

3. **The floors are the estimators'.** The 6.116 non-IID minimum on urandom is the low tail of that estimator's distribution on 1 MB slices. The ~7.86 IID minimum is the low tail of the IID estimator's distribution. Any source whose true min-entropy is above these floors will produce slices inside the corresponding band, and the tool cannot tell you where inside the band the source actually sits.

4. **Both tracks share quantized values.** The same six-decimal values recur across unrelated runs and unrelated sources — `7.881334`, `7.878561`, `7.878907`, `7.872686`, `7.869585`, `7.874756`, `7.885156`, `7.885504` on the IID track; `6.638399`, `6.638405`, `7.353758`, `5.306249`, `4.804749` on the non-IID track. These are the estimator reporting at finite precision on preferred histogram shapes. Do not count repeats as separate observations, and do not interpret a difference of a few units in the sixth decimal as a real difference in the source. The tool rounds; the source does not.

If your source's numbers overlap the urandom envelope on either track, the correct conclusion is **"the tool cannot distinguish this source from uniform at this slice size"** — not "this source equals urandom in min-entropy," and not "this source has 7.88 bits/byte."

---

## Reading the output

```
[42/100] slice_0042 (1000000 bytes) ... H_orig=7.893880 H_bit=0.997632 min=7.893880
```

- `H_orig` — MCV estimate on byte symbols, in bits/byte.
- `H_bit` — MCV estimate on bit symbols, in bits/bit.
- `min` — `min(H_orig, 8 × H_bit)`. This is the reported bound. The binding constraint is whichever term is smaller.

On short slices with a high-quality source, `H_orig` is almost always the binding constraint, because `8 × H_bit` sits near 7.98 and byte-level MCV sits near 7.88. When `H_bit` drops and becomes binding, that is the signature of a bit-level structure the prediction estimators or the bit-level MCV found. It is worth investigating.

### Slice size and slice count

- **1 MB slices** give the widest estimator envelope and the least useful discrimination at the top. Good for detecting collapse.
- **10 MB slices** raise the non-IID maximum slightly (~+0.05 bits) and narrow the spread, but do not raise the typical value. On the IID track, the ceiling rises from ~7.88 to ~7.98 because MCV bias shrinks as O(1/√n).
- **100 MB slices** push the IID ceiling to ~7.998. This is where a real defect of a few hundredths of a bit becomes visible.
- **100 slices** is the minimum for a usable maximum and a rough shape. Fewer slices cannot characterise the tail. More slices mostly repeat the same envelope.

If your source's worst slice is within the urandom floor on either track, the tool is not telling you the source is bad — it is telling you the slice is short. Increase slice length before concluding anything about a source that is not collapsing.

---

## When to switch tracks

Use `ea_non_iid` first. It is the more conservative bound and it detects collapse cleanly.

Switch to `ea_iid` when:
- The non-IID typical is at the non-IID ceiling (~7.4–7.5 on 1 MB) and you want to know whether the source is genuinely at the top or is being masked by the estimator's saturation.
- You need a number that can discriminate between two good sources.
- The source passes the IID gate.

Do not use `ea_iid` as a replacement for `ea_non_iid`. They answer different questions. The non-IID bound is the one that holds under adversarial interpretation; the IID number is the one that tells you the source is well-behaved at the resolution measured.

### The IID gate is a filter, not a proof

`ea_iid` runs ten independent IID-validation tests before it will report a number. Failing any of them causes the tool to refuse to report, rather than to report a misleading value.

**Passing the gate is not the same as being IID.** The gate is a bank of ten statistical tests at finite sample size. A source with structure below the tests' detection threshold will pass and still have structure. Failing the gate is conclusive evidence of non-IIDness; passing it is a strong but not absolute statement that the source is statistically indistinguishable from IID at the resolution measured.

This caveat applies equally to any single-slice `ea_iid` result. A gate pass on 100 slices of 1 MB each is 100 gate passes at 1 MB resolution, not a proof of IIDness at all resolutions.

---

## Interpreting your source

### A source is collapsing if...

- Any slice reports `H_bitstring` below ~0.5 with `H_orig` high. This is bit-level structure the prediction estimators found and byte-level MCV did not. It is the signature of a source whose byte histogram looks fine but whose bit-to-bit dependencies are not.
- The minimum `min_bound` over 100 slices is below ~5.0 on the non-IID track. This is outside the urandom envelope and is a real signal.
- The IID gate refuses to run on a source that is not expected to be non-IID. This is a real structural signal.
- The streaming statistical tests (PractRand, NIST STS) reject the same stream. Min-entropy and statistical pass/fail are orthogonal, but a source that fails both is not a source.

### A source is at the tool's ceiling if...

- The typical non-IID `min_bound` is above ~7.4 and the maximum is at ~7.5, **and** the typical IID `min_bound` is above ~7.85 and the maximum is at ~7.89.
- The spread is narrow on both tracks (max − min < 1.0 non-IID, < 0.05 IID).
- The IID gate passes on every slice.

At the ceiling, the tool has no more to say at that slice size. Stop running larger slice counts and either increase slice length or switch instruments:
- **PractRand / TestU01 Crush** for structural defects (linearity, lattice, correlation).
- **A pooled bias bound** for a direct measure of distance from uniform: pool all slices, compute the maximum byte-histogram deviation ε, report `−log₂(1/256 + ε)`. This converges toward 8.0 on uniform data and gives a real number for a high-quality source, which neither estimator can do at saturation.
- **The SP 800-90B restart structure** (1000 × 1000 samples per configuration) for a worst-case rather than steady-state bound. Expect ~3 hours per configuration on a single machine.

### A source is indistinguishable from uniform if...

- Its envelope overlaps urandom's on the typical and the maximum, on both tracks.
- Its minimum is above the urandom floor on both tracks.
- The IID gate passes on every slice.

This is the strongest claim the tool can support. It is **not** a claim that the source is cryptographically secure, and it is not a claim about unpredictability to an adversary. It is a claim that the estimators cannot see a difference between this source and uniform on the slices measured, at the resolution measured.

---

## What the tool cannot do

- **It cannot distinguish two good sources.** A source at the ceiling and a source above the ceiling produce the same number.
- **It cannot give a true min-entropy.** It gives a conservative lower bound under the estimator's assumptions.
- **It cannot prove IIDness.** The IID gate is a filter. Passing it is necessary but not sufficient.
- **It cannot replace statistical tests.** A source can pass min-entropy and fail PractRand; a source can pass PractRand and have a low non-IID bound. Run both.
- **It cannot assess cryptographic security.** Min-entropy is a statistical property. Unpredictability against a co-resident adversary is a different property, measured by different means.
- **It cannot give a worst-case bound from steady-state runs.** The restart structure is the only path to a worst-case number, and it is not run by default.

---

## Practical recipe

```bash
# 1. Generate a stream long enough for the slice size you intend to use.
./your-generator | head -c 100000000 > stream.bin

# 2. Slice it.
split -b 1000000 stream.bin slice_

# 3. Run the non-IID estimators on each slice.
for f in slice_*; do ea_non_iid -i -a "$f" 8; done

# 4. If the typical is at the non-IID ceiling, re-run with ea_iid.
for f in slice_*; do ea_iid -i -a "$f" 8; done

# 5. Run the same pipeline on /dev/urandom as a control, on whichever
#    track you used for the source.
head -c 100000000 /dev/urandom > urandom.bin
split -b 1000000 urandom.bin uslice_
for f in uslice_*; do ea_non_iid -i -a "$f" 8; done
# (and ea_iid if applicable)

# 6. Compare envelopes. If your source's minimum is below the urandom
#    floor on either track, increase slice length before drawing
#    conclusions. If your source's typical is at the urandom ceiling on
#    both tracks, the tool cannot distinguish it from uniform at this
#    resolution; switch to a pooled bias bound or to structural tests.
```

Report the typical value, the maximum, the minimum, and the spread, on both tracks if both were run. A single number is not a measurement; the shape of the 100-slice distribution is.

---

## A worked example: `qxo64`

The `qxo64` filter was assessed on 100 × 1 MB slices with a seed from a trusted chaotic source. Results on the two tracks, with `/dev/urandom` controls on the same pipeline:

| track | qxo64 typical | qxo64 range | urandom typical | urandom range |
|---|---|---|---|---|
| `ea_non_iid` | ~7.1 | 6.638 – 7.28 | ~7.10 | 6.116 – 7.486 |
| `ea_iid` | ~7.88 | 7.855 – 7.894 | ~7.88 | 7.864 – 7.887 |

**Reading the non-IID result.** The non-IID track saturates at ~7.47 on 1 MB slices. qxo64's typical of ~7.1 sits inside the urandom envelope; the apparent deficit is the estimator's low tail, not a property of the filter. The non-IID track cannot distinguish qxo64 from uniform at this slice size and slice count. The one thing it can say is that qxo64 is not collapsing: its minimum is above the urandom floor and its spread is narrower than urandom's.

**Reading the IID result.** The IID track has headroom to ~7.88 on 1 MB slices, and qxo64 sits at its ceiling. The qxo64 and urandom distributions are statistically identical: mean difference 0.002 bits, ranges overlapping, shared quantized values across the two datasets. The IID gate passed on every slice. qxo64's byte-level output is indistinguishable from uniform at 1 MB resolution by the SP 800-90B IID estimators.

**What this does and does not support.** qxo64 is at the tool's ceiling on both tracks at 1 MB resolution. The correct conclusion is the narrow one: the estimators see no exploitable structure in qxo64's output at this slice size and slice count. This is not a claim about cryptographic security. It is not a claim that qxo64 is better than urandom — only that this instrument, at this resolution, cannot tell them apart. It is not a claim that qxo64 is IID; it is a claim that the IID gate did not reject it.

**Where a real defect would show.** A source bias of a few hundredths of a bit would be invisible on 1 MB slices but visible on 100 MB slices, where the IID ceiling rises to ~7.998. If qxo64 stays within ~0.005 of urandom at 100 MB, the filter is as good as this instrument can resolve. If it does not, the gap is the first measurable defect the filter has shown.

---

## Reference

- NIST SP 800-90B, *Recommendation for the Entropy Sources Used for Random Bit Generation*, §5 and §6.
- The reference C++ implementation: `ea_non_iid`, `ea_iid`, `ea_restart`. Build against `libjsoncpp-dev`, `libdivsufsort-dev`, `libssl-dev`, `libmpfr-dev`, and OpenMP.
