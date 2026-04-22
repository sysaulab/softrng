use std::ptr::{read_volatile, write_volatile};
use std::thread;
use std::time::Duration;

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

// Wrapper to make raw pointers Send (safe because threads are joined before pointed data goes out of scope)
#[repr(transparent)]
#[derive(Clone, Copy)]
struct SendPtr<T>(*mut T);
unsafe impl<T> Send for SendPtr<T> {}

unsafe fn seed_modify_64(input: *mut u64, output: *mut u64) {
    let mut acc: u64 = 1;
    let mut x = 0;
    while x < 64 {
        let val = read_volatile(input);
        let rotated = val.rotate_left(7);   // safer and clearer
        write_volatile(input, rotated);

        acc = acc.rotate_left(x as u32);    // FIXED: no more UB

        acc = acc.wrapping_mul(PRIMES[(2 * x) + (rotated & 1) as usize]);

        let out_val = read_volatile(output);
        write_volatile(output, out_val.wrapping_add(acc ^ rotated));

        x += 1;
    }
    let out_val = read_volatile(output);
    write_volatile(output, out_val ^ acc);
}

struct SeedThread {
    source: SendPtr<u64>,
    sink: SendPtr<u64>,
    run: SendPtr<i32>,
}

fn seed_thread_main(state: SeedThread) {
    unsafe {
        while read_volatile(state.run.0) != 0 {
            seed_modify_64(state.source.0, state.sink.0);
        }
    }
}

pub struct Seedy64 {
    nodes: [u64; 3],
    init_duration: Duration,   // in nanoseconds internally
    interval_duration: Duration,
}

impl Seedy64 {
    pub fn new() -> Self {
        Self::with_timings(1_000_000, 10_000)
    }

    pub fn with_timings(init_ns: u64, interval_ns: u64) -> Self {
        Seedy64 {
            nodes: [0; 3],
            init_duration: Duration::from_nanos(init_ns),
            interval_duration: Duration::from_nanos(interval_ns),
        }
    }

    unsafe fn read_state(&self) -> u64 {
        let n0 = read_volatile(&self.nodes[0]);
        let n1 = read_volatile(&self.nodes[1]);
        let n2 = read_volatile(&self.nodes[2]);
        n0 ^ n1 ^ n2
    }

    pub fn fill_buffer(&mut self, buffer: &mut [u8]) {
        let bytes = buffer.len();
        let blocks = bytes / 8;
        let mut i = 0;

        let mut run_flags = [1i32; 3];
        let run_ptrs: [SendPtr<i32>; 3] = [
            SendPtr(&mut run_flags[0] as *mut i32),
            SendPtr(&mut run_flags[1] as *mut i32),
            SendPtr(&mut run_flags[2] as *mut i32),
        ];

        let node_ptrs: [SendPtr<u64>; 3] = [
            SendPtr(&mut self.nodes[0] as *mut u64),
            SendPtr(&mut self.nodes[1] as *mut u64),
            SendPtr(&mut self.nodes[2] as *mut u64),
        ];

        let configs = [
            (node_ptrs[0], node_ptrs[1], run_ptrs[0]),
            (node_ptrs[1], node_ptrs[2], run_ptrs[1]),
            (node_ptrs[2], node_ptrs[0], run_ptrs[2]),
        ];

        let handles: Vec<_> = configs
            .into_iter()
            .map(|(src, snk, run)| {
                let state = SeedThread {
                    source: src,
                    sink: snk,
                    run,
                };
                thread::spawn(move || seed_thread_main(state))
            })
            .collect();

        thread::sleep(self.init_duration);

        let mut last_pick = unsafe { self.read_state() };

        while i < 8 * blocks {
            thread::sleep(self.interval_duration);
            let next_pick = unsafe { self.read_state() };
            if next_pick != last_pick {
                buffer[i..i + 8].copy_from_slice(&next_pick.to_ne_bytes());
                last_pick = next_pick;
                i += 8;
            }
        }

        let mut j = 0;
        while i < bytes {
            thread::sleep(self.interval_duration);
            let next_pick = unsafe { self.read_state() };
            if next_pick != last_pick {
                buffer[i] = next_pick.to_ne_bytes()[j];
                i += 1;
                j += 1;
                last_pick = next_pick;
            }
        }

        for flag in &mut run_flags {
            unsafe { write_volatile(flag, 0) };
        }

        for handle in handles {
            let _ = handle.join();
        }
    }
}

impl Default for Seedy64 {
    fn default() -> Self {
        Self::new()
    }
}
