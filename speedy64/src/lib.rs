//! speedy64 — a userspace entropy source.
//!
//! Three threads race on a ring of shared `u64` nodes. Thread 0 mixes and
//! emits one 64-bit word every `skip` passes, yielding to the scheduler
//! immediately before each emission. The scheduler's decision at the
//! sampling instant is the entropy source; the ring provides the state
//! space that the decision projects onto.
//!
//! # Security
//!
//! **This is not a cryptographic random number generator.** The entropy it
//! harvests is shared with every other process on the same machine. Do not
//! use the output for keys, tokens, nonces, IVs, passwords, or session IDs.
//! Do not use it as a primary entropy source. Do not expose its output to a
//! remote observer.
//!
//! Statistical quality and cryptographic security are different properties.
//! This crate has only the first.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// The prime table used by the mixing step.
///
/// Exposed for reproducibility and experimentation. Changing these changes
/// the output stream and invalidates any measurement made with the
/// published values.
pub const PRIMES: [u64; 128] = [
    2985113428254948959, 2133371973447287789, 9566925003104080411, 6083414080430737261,
    3191658566037706763, 6913367267901031997, 3051762490125913007, 4496279044463191703,
    2879325911889076339, 1685450054146532617, 3665492224534333199, 5457405118009752311,
    9603674174905975319, 5115570615370213399, 2085406057152991381, 3944123521248288751,
    7068318273059084621, 9757209530458778617, 4605875306346355289, 6173263078309968247,
    7762991333463966857, 9020426977113888977, 7479023155365493211, 1106308364738794853,
    6407388572493574733, 2575377960084637447, 3570804859728957167, 9744215084471400349,
    5786617550671126231, 3349473746601511027, 4095923646587284861, 9337574612835542873,
    3860411135543820653, 6490253556739218821, 8764900392399904901, 3339372363887497583,
    6577701767928311429, 9172672799009443051, 2788564470716716463, 7776210437768060567,
    9071396938031495159, 5357072389526583433, 6633238451589839561, 6234713344370965483,
    1756087432259377561, 3210111471103586737, 2408295824875360559, 8027800137025286071,
    1345382896972678691, 9079595888451204607, 6556450002823963531, 7301373103704944939,
    1213337252585919101, 8574942160927332881, 8566185364500201241, 6910045784078487233,
    2298581249625702883, 2596136803677419831, 8990581036371057601, 3046889204803159003,
    6864701819024214703, 9142639517683050829, 3257525145999184439, 5916350576516314903,
    1991008946812235681, 5221054210441722691, 4658950185140640559, 7616154792906075869,
    3789735489826889479, 6040491638497194979, 4531551599015040559, 8347022247974757701,
    5867558032219360441, 4346307676337591453, 1905346606445378533, 7219759323757378847,
    8561163821217102983, 6227762971465314193, 1778354250598502561, 5435722899178511549,
    7859180386742822047, 3610133528213982527, 4577787558750808879, 3974497005134013109,
    3241288686894948329, 3206546850542044487, 2621128575668534623, 6945339563322422683,
    2086055779082649859, 9237096783908108657, 1916568609042647407, 8362781431934621017,
    6122543372115730003, 6200399818195771541, 1259180073496439959, 4347405785030416513,
    4385862544209720253, 5836562284002027449, 3121678407354608831, 4319136453194505203,
    9085838656424873461, 5448516463713100871, 4520015808866767003, 2892328915328607301,
    3897908110284010777, 6252018905800065119, 3278710975885668139, 2623408458547857527,
    3055342605008846819, 5255646576194862401, 5611094629836787733, 3296996236992951511,
    3283558997806116761, 2823396471863459119, 9572308265712226037, 8290430615658140401,
    2621246478142939039, 1300291691995934101, 1143863624227711889, 6789671898597272039,
    1593529243588418053, 6573565795099723367, 7154224095543629129, 2355509713379653547,
    7691022839137400143, 2727794217173953619, 8541344794240933387, 6490328854741497349,
];

/// Runtime configuration for a [`Generator`].
///
/// The defaults are the values measured in the published characterisation:
/// `skip = 1`, one millisecond of warm-up, yield enabled, native-endian.
#[derive(Clone, Debug)]
pub struct Config {
    /// Number of full mixing passes between emitted words.
    ///
    /// The published measurements use `skip = 1`: one mixing pass, one
    /// yield, one emission. Larger values reduce the emission rate without
    /// providing the scheduler event that the smaller value does.
    pub skip: u64,

    /// Warm-up duration before the first emission.
    ///
    /// The mixing threads run unconditionally during this window, so the
    /// ring is populated with scheduler-laden state rather than the zero
    /// initialisation. One millisecond is sufficient on bare metal; raise
    /// it on loaded or virtualised hosts.
    pub init_duration: Duration,

    /// Yield to the scheduler immediately before each emission.
    ///
    /// This is the entropy mechanism. Disabling it produces output that
    /// fails PractRand. See the paper for why.
    pub emit_yield: bool,

    /// Emit byteswapped (big-endian) words instead of native-endian.
    pub big_endian: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            skip: 1,
            init_duration: Duration::from_millis(1),
            emit_yield: true,
            big_endian: false,
        }
    }
}

/// One mixing pass on the ring.
///
/// Reads the source node, rotates and writes it back to itself (the
/// self-feedback edge), and mixes the rotated value into the sink through
/// a prime-selected nonlinear step. All operations are relaxed atomics.
#[inline]
pub fn mix(input: &AtomicU64, output: &AtomicU64) {
    let mut acc: u64 = 1;
    let mut x = 0;
    while x < 64 {
        let val = input.load(Ordering::Relaxed);
        let rotated = val.rotate_left(7);
        input.store(rotated, Ordering::Relaxed);

        acc = acc.rotate_left(x as u32);
        acc = acc.wrapping_mul(PRIMES[(2 * x) + (rotated & 1) as usize]);

        let out_val = output.load(Ordering::Relaxed);
        output.store(out_val.wrapping_add(acc ^ rotated), Ordering::Relaxed);

        x += 1;
    }
    let out_val = output.load(Ordering::Relaxed);
    output.store(out_val ^ acc, Ordering::Relaxed);
}

/// A running speedy64 generator.
///
/// The ring state persists across calls to [`stream_to`](Self::stream_to)
/// and [`fill`](Self::fill). To obtain statistically independent output
/// (reset-per-output mode), construct a fresh `Generator` for each sample.
/// Construction is cheap; the mixer threads are not spawned until the first
/// call to `stream_to` or `fill`.
///
/// Dropping the value stops the mixer threads and joins them.
pub struct Generator {
    nodes: [Arc<AtomicU64>; 3],
    run: Arc<AtomicBool>,
    config: Config,
    handles: Vec<JoinHandle<()>>,
    started: bool,
}

impl Generator {
    /// Construct a generator with the given configuration.
    ///
    /// No threads are spawned until the first call to
    /// [`stream_to`](Self::stream_to) or [`fill`](Self::fill).
    pub fn new(config: Config) -> Self {
        Self {
            nodes: [
                Arc::new(AtomicU64::new(0)),
                Arc::new(AtomicU64::new(0)),
                Arc::new(AtomicU64::new(0)),
            ],
            run: Arc::new(AtomicBool::new(true)),
            config,
            handles: Vec::new(),
            started: false,
        }
    }

    /// Construct a generator with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(Config::default())
    }

    fn ensure_started(&mut self) {
        if self.started {
            return;
        }
        self.run.store(true, Ordering::Release);

        let h1 = spawn_mixer(
            Arc::clone(&self.nodes[1]),
            Arc::clone(&self.nodes[2]),
            Arc::clone(&self.run),
        );
        let h2 = spawn_mixer(
            Arc::clone(&self.nodes[2]),
            Arc::clone(&self.nodes[0]),
            Arc::clone(&self.run),
        );
        self.handles.push(h1);
        self.handles.push(h2);

        thread::sleep(self.config.init_duration);
        self.started = true;
    }

    /// Write 64-bit words to `w` until the writer returns an error.
    ///
    /// Returns the number of words written. A writer error (including
    /// `BrokenPipe` from a closed stdout) is treated as end-of-stream, not
    /// as a failure; it is the intended shutdown signal.
    ///
    /// The ring state persists across calls. To reset, drop the generator
    /// and construct a new one.
    pub fn stream_to<W: Write>(&mut self, w: &mut W) -> io::Result<u64> {
        self.ensure_started();

        let mut counter: u64 = 0;
        let mut written: u64 = 0;
        let mut buf = [0u8; 8];

        loop {
            mix(&self.nodes[0], &self.nodes[1]);

            counter += 1;
            if counter < self.config.skip.max(1) {
                continue;
            }
            counter = 0;

            if self.config.emit_yield {
                thread::yield_now();
            }

            let state = self.nodes[0].load(Ordering::Relaxed)
                ^ self.nodes[1].load(Ordering::Relaxed)
                ^ self.nodes[2].load(Ordering::Relaxed);

            buf = if self.config.big_endian {
                state.to_be_bytes()
            } else {
                state.to_ne_bytes()
            };

            if w.write_all(&buf).is_err() {
                break;
            }
            written += 1;
        }

        Ok(written)
    }

    /// Fill `buf` completely with output.
    ///
    /// `buf.len()` should be a multiple of 8 for clean word boundaries; any
    /// trailing bytes are filled from the last word's low bytes. Returns
    /// the number of complete 64-bit words written.
    pub fn fill(&mut self, buf: &mut [u8]) -> io::Result<u64> {
        let mut cursor: &mut [u8] = buf;
        self.stream_to(&mut cursor)
    }

    /// Stop the mixer threads and join them.
    ///
    /// Called automatically on drop. Calling it explicitly is optional;
    /// useful if you want to release the threads before the generator
    /// value goes out of scope.
    pub fn stop(&mut self) {
        if !self.started {
            return;
        }
        self.run.store(false, Ordering::Release);
        for h in self.handles.drain(..) {
            let _ = h.join();
        }
        self.started = false;
    }

    /// Returns the configuration this generator was constructed with.
    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl Drop for Generator {
    fn drop(&mut self) {
        self.stop();
    }
}

#[inline]
fn spawn_mixer(
    src: Arc<AtomicU64>,
    snk: Arc<AtomicU64>,
    run: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while run.load(Ordering::Acquire) {
            mix(&src, &snk);
        }
    })
}