use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use clap::Parser;

const PRIMES: [u64; 128] = [
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

/// Seedy64 threaded-emit prototype.
///
/// Three threads race on a ring of shared u64 nodes. Thread 0 is the main
/// thread; it mixes and emits one 64-bit word every <SKIP> full mixing passes.
/// Threads 1 and 2 are pure mixers. Output is raw 64-bit words on stdout,
/// native-endian, no framing, no delimiters.
#[derive(Parser, Debug)]
#[command(
    name = "seedy64-threaded",
    version,
    about = "Timing-jitter harvesting and statistical mixing in one pass.",
    long_about = None,
    after_help = "EXAMPLES:\n  \
        seedy64-threaded 3 1000000 > stream.bin\n  \
        seedy64-threaded 3 1000000 | head -c 1000000 > slice.bin\n\n\
        The first positional argument is SKIP, the second is INIT_NS."
)]
struct Args {
    /// Number of full mixing passes between emitted words.
    ///
    /// Small values (1-4) produce more output but consecutive words are more
    /// correlated. Large values (64-256) produce less output but each word is
    /// more decorrelated. The default of 3 is a reasonable starting point.
    #[arg(value_name = "SKIP", default_value_t = 3)]
    skip: u64,

    /// Warm-up duration in nanoseconds, before the first emit.
    ///
    /// The mixing threads run unconditionally during this window so the ring
    /// is populated with timing-jitter-laden state rather than the initial
    /// zeros. 1 ms is enough on bare metal; raise it on loaded or virtualized
    /// hosts.
    #[arg(value_name = "INIT_NS", default_value_t = 1_000_000)]
    init_ns: u64,

    /// Write byteswapped (big-endian) words instead of native-endian.
    ///
    /// Useful when the output is consumed by a network-ordered pipeline or
    /// by a big-endian analysis tool.
    #[arg(long, default_value_t = false)]
    big_endian: bool,

    /// Yield to the scheduler immediately before each emit.
    ///
    /// Forces a scheduler decision at every emission instant, introducing
    /// variable delay and possible core migration. Intended to recover the
    /// jitter that the original polling loop obtained from nanosleep.
    #[arg(long, default_value_t = false)]
    emit_yield: bool,
}

#[inline]
fn seed_modify_64(input: &AtomicU64, output: &AtomicU64) {
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

#[inline]
fn spawn_mixer(
    src: Arc<AtomicU64>,
    snk: Arc<AtomicU64>,
    run: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while run.load(Ordering::Acquire) {
            seed_modify_64(&src, &snk);
        }
    })
}

fn main() {
    let args = Args::parse();

    let nodes = [
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
    ];
    let run = Arc::new(AtomicBool::new(true));

    let h1 = spawn_mixer(Arc::clone(&nodes[1]), Arc::clone(&nodes[2]), Arc::clone(&run));
    let h2 = spawn_mixer(Arc::clone(&nodes[2]), Arc::clone(&nodes[0]), Arc::clone(&run));

    thread::sleep(Duration::from_nanos(args.init_ns));

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut counter: u64 = 0;
    let mut buf = [0u8; 8];

    'emit: loop {
        seed_modify_64(&nodes[0], &nodes[1]);

        counter += 1;
        if counter < args.skip {
            continue;
        }
        counter = 0;

        if args.emit_yield {
            thread::yield_now();
        }

        let state = nodes[0].load(Ordering::Relaxed)
            ^ nodes[1].load(Ordering::Relaxed)
            ^ nodes[2].load(Ordering::Relaxed);

        buf = if args.big_endian {
            state.to_be_bytes()
        } else {
            state.to_ne_bytes()
        };

        if out.write_all(&buf).is_err() {
            break 'emit;
        }
    }

    run.store(false, Ordering::Release);
    let _ = h1.join();
    let _ = h2.join();
}