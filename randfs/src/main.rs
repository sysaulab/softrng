mod common;

use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use chacha20::{ChaCha20, Key, Nonce};
use clap::Parser;
use fuser::{
    Errno, FileAttr, FileHandle, FileType, Filesystem, FopenFlags, Generation, INodeNo,
    LockOwner, MountOption, OpenFlags, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    ReplyOpen, Request,
};

use crate::common::args::CommonArgs;

thread_local! {
    static THREAD_INDEX: Cell<Option<usize>> = const { Cell::new(None) };
}

#[derive(Parser)]
#[command(version, author = "Christopher Berner")]
struct Args {
    #[clap(flatten)]
    common_args: CommonArgs,

    /// Directory whose (32-byte) files seed the virtual filesystem.
    #[arg(long, value_name = "DIR")]
    ghost_dir: PathBuf,
}

/// Reported size for every virtual file.  `i64::MAX` rather than `u64::MAX`
/// because many userspace tools do signed math on `st_size`.
const MAX_SIZE: u64 = i64::MAX as u64;

/// Zero TTL so the kernel re-asks us for attributes on every lookup,
/// picking up mtime/perm changes to seed files immediately.
const TTL: Duration = Duration::ZERO;

/// Local ceiling on a single read.
const MAX_READ: u32 = 1 << 20;

// ChaCha20-IETF has a 32-bit block counter, i.e. 2^38 bytes of stream per
// (key, nonce) pair.  Split the u64 address space into 2^32-byte chunks,
// each with its own nonce = chunk index.
const CHUNK_BITS: u32 = 32;
const CHUNK_SIZE: u64 = 1u64 << CHUNK_BITS;
const CHUNK_MASK: u64 = CHUNK_SIZE - 1;

/// Synthetic inode for the per-thread read counter file.  Inode 1 is the
/// root directory; seeds start at 3.
const STATS_INO: INodeNo = INodeNo(2);
const STATS_NAME: &str = ".metadata_never_index";
const FIRST_SEED_INO: u64 = 3;

/// Fill `out` with pseudorandom bytes from the stream defined by `seed`,
/// starting at byte `offset`.  Deterministic for a given (seed, offset, len).
fn fill_random(seed: &[u8; 32], offset: u64, out: &mut [u8]) {
    let mut done = 0usize;
    let mut pos = offset;
    while done < out.len() {
        let chunk_idx = pos >> CHUNK_BITS;
        let in_chunk = pos & CHUNK_MASK;

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..].copy_from_slice(&chunk_idx.to_be_bytes());

        let key = Key::from_slice(seed);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let mut cipher = ChaCha20::new(key, nonce);
        cipher.seek(in_chunk);

        let remaining_in_chunk = (CHUNK_SIZE - in_chunk) as usize;
        let n = remaining_in_chunk.min(out.len() - done);
        cipher.apply_keystream(&mut out[done..done + n]);

        done += n;
        pos = pos
            .checked_add(n as u64)
            .expect("read offset overflowed u64");
    }
}

struct SeedEntry {
    ino: INodeNo,
    name: OsString,
    seed_path: PathBuf,
    seed: [u8; 32],
}

struct RandomFS {
    entries: Vec<SeedEntry>,
    by_name: HashMap<OsString, usize>,
    by_ino: HashMap<INodeNo, usize>,
    reads_per_thread: Vec<AtomicU64>,
    next_thread_index: AtomicUsize,
}

impl RandomFS {
    fn from_ghost_dir(ghost_dir: &Path, n_threads: usize) -> io::Result<Self> {
        let mut dirents: Vec<_> = fs::read_dir(ghost_dir)?
            .filter_map(Result::ok)
            .filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false))
            .collect();
        // Deterministic inode assignment.
        dirents.sort_by_key(|e| e.file_name());

        let mut entries = Vec::with_capacity(dirents.len());
        for (i, dirent) in dirents.into_iter().enumerate() {
            let seed_path = dirent.path();
            let seed = read_seed(&seed_path)?;
            entries.push(SeedEntry {
                ino: INodeNo(FIRST_SEED_INO + i as u64),
                name: dirent.file_name(),
                seed_path,
                seed,
            });
        }

        let mut by_name = HashMap::with_capacity(entries.len());
        let mut by_ino = HashMap::with_capacity(entries.len());
        for (i, e) in entries.iter().enumerate() {
            by_name.insert(e.name.clone(), i);
            by_ino.insert(e.ino, i);
        }

        Ok(Self {
            entries,
            by_name,
            by_ino,
            reads_per_thread: (0..n_threads).map(|_| AtomicU64::new(0)).collect(),
            next_thread_index: AtomicUsize::new(0),
        })
    }

    fn attr_for(&self, entry: &SeedEntry) -> FileAttr {
        let (atime, mtime, ctime, perm, uid, gid) = match fs::metadata(&entry.seed_path) {
            Ok(m) => (
                systime(m.atime(), m.atime_nsec()),
                systime(m.mtime(), m.mtime_nsec()),
                systime(m.ctime(), m.ctime_nsec()),
                (m.mode() & 0o7777) as u16,
                m.uid(),
                m.gid(),
            ),
            // Seed vanished since startup: fall back to safe defaults.
            Err(_) => (UNIX_EPOCH, UNIX_EPOCH, UNIX_EPOCH, 0o444, 501, 20),
        };

        FileAttr {
            ino: entry.ino,
            size: MAX_SIZE,
            blocks: MAX_SIZE / 512,
            atime,
            mtime,
            ctime,
            crtime: UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm,
            nlink: 1,
            uid,
            gid,
            rdev: 0,
            flags: 0,
            blksize: 4096,
        }
    }

    /// One line per worker thread, each with the number of reads it has
    /// served.  Size grows as counters gain digits, hence zero TTL.
    fn stats_content(&self) -> String {
        let mut s = String::with_capacity(self.reads_per_thread.len() * 8);
        for c in &self.reads_per_thread {
            s.push_str(&c.load(Ordering::Relaxed).to_string());
            s.push('\n');
        }
        s
    }

    fn stats_attr(&self) -> FileAttr {
        let len = self.stats_content().len() as u64;
        FileAttr {
            ino: STATS_INO,
            size: len,
            blocks: len.div_ceil(512),
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm: 0o444,
            nlink: 1,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
            blksize: 512,
        }
    }
}

fn read_seed(path: &Path) -> io::Result<[u8; 32]> {
    let data = fs::read(path)?;
    if data.len() != 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "seed file {} must be exactly 32 bytes, got {}",
                path.display(),
                data.len()
            ),
        ));
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&data);
    Ok(seed)
}

fn systime(secs: i64, nsecs: i64) -> SystemTime {
    if secs < 0 {
        UNIX_EPOCH
    } else {
        UNIX_EPOCH + Duration::new(secs as u64, nsecs.clamp(0, 999_999_999) as u32)
    }
}

const ROOT_ATTR: FileAttr = FileAttr {
    ino: INodeNo::ROOT,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH,
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o555,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

impl Filesystem for RandomFS {
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        if parent != INodeNo::ROOT {
            return reply.error(Errno::ENOENT);
        }
        if name.to_str() == Some(STATS_NAME) {
            // Zero TTL: the file's reported size changes as counters tick.
            reply.entry(&Duration::ZERO, &self.stats_attr(), Generation(0));
            return;
        }
        match self.by_name.get(name) {
            Some(&idx) => {
                let attr = self.attr_for(&self.entries[idx]);
                reply.entry(&TTL, &attr, Generation(0));
            }
            None => reply.error(Errno::ENOENT),
        }
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
        if ino == INodeNo::ROOT {
            return reply.attr(&TTL, &ROOT_ATTR);
        }
        if ino == STATS_INO {
            return reply.attr(&Duration::ZERO, &self.stats_attr());
        }
        match self.by_ino.get(&ino) {
            Some(&idx) => reply.attr(&TTL, &self.attr_for(&self.entries[idx])),
            None => reply.error(Errno::ENOENT),
        }
    }

    fn open(&self, _req: &Request, ino: INodeNo, flags: OpenFlags, reply: ReplyOpen) {
        if ino != STATS_INO && self.by_ino.get(&ino).is_none() {
            return reply.error(Errno::ENOENT);
        }
        // fuser 0.18: OpenFlags wraps the raw open(2) flags.
        // O_ACCMODE == 0o3; O_WRONLY == 1, O_RDWR == 2.
        let accmode = flags.0 & 0o3;
        if accmode == 1 || accmode == 2 {
            return reply.error(Errno::EACCES);
        }
        reply.opened(FileHandle(0), FopenFlags::empty());
    }

    fn read(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        size: u32,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        reply: ReplyData,
    ) {
        let thread_idx = THREAD_INDEX.with(|idx| match idx.get() {
            Some(i) => i,
            None => {
                let new_idx = self.next_thread_index.fetch_add(1, Ordering::SeqCst);
                idx.set(Some(new_idx));
                new_idx
            }
        });
        if thread_idx < self.reads_per_thread.len() {
            self.reads_per_thread[thread_idx].fetch_add(1, Ordering::Relaxed);
        }

        // Synthetic file: regenerate content, slice at [offset, offset+size).
        if ino == STATS_INO {
            let content = self.stats_content();
            let bytes = content.as_bytes();
            let start = (offset as usize).min(bytes.len());
            let end = (start + size as usize).min(bytes.len());
            return reply.data(&bytes[start..end]);
        }

        let Some(&idx) = self.by_ino.get(&ino) else {
            return reply.error(Errno::ENOENT);
        };
        let entry = &self.entries[idx];

        if offset >= MAX_SIZE {
            return reply.data(&[]);
        }

        let remaining = MAX_SIZE - offset;
        let want = (size.min(MAX_READ) as u64).min(remaining) as usize;

        let mut buf = vec![0u8; want];
        fill_random(&entry.seed, offset, &mut buf);
        reply.data(&buf);
    }

    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        if ino != INodeNo::ROOT {
            return reply.error(Errno::ENOTDIR);
        }

        let names: Vec<(INodeNo, FileType, OsString)> =
            std::iter::once((INodeNo::ROOT, FileType::Directory, OsString::from(".")))
                .chain(std::iter::once((
                    INodeNo::ROOT,
                    FileType::Directory,
                    OsString::from(".."),
                )))
                .chain(std::iter::once((
                    STATS_INO,
                    FileType::RegularFile,
                    OsString::from(STATS_NAME),
                )))
                .chain(
                    self.entries
                        .iter()
                        .map(|e| (e.ino, FileType::RegularFile, e.name.clone())),
                )
                .collect();

        for (i, (ino, kind, name)) in names.into_iter().enumerate().skip(offset as usize) {
            if reply.add(ino, (i + 1) as u64, kind, &name) {
                break;
            }
        }
        reply.ok();
    }
}

fn main() {
    let args = Args::parse();

    let mut cfg = args.common_args.config();
    cfg.mount_options
        .extend([MountOption::RO, MountOption::FSName("randfs".into())]);

    let fs = RandomFS::from_ghost_dir(&args.ghost_dir, args.common_args.n_threads)
        .unwrap_or_else(|e| {
            eprintln!(
                "failed to load ghost directory {}: {e}",
                args.ghost_dir.display()
            );
            std::process::exit(1);
        });

    fuser::mount(fs, &args.common_args.mount_point, &cfg).unwrap();
}