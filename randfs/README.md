
# randfs

A tiny FUSE filesystem that serves **infinite, deterministic, seekable
pseudorandom data** from a directory of 32-byte seed files.

Built as a test harness: hex editors, backup tools, `dd`, `rsync`, and
anything else that does `lseek` + `read` on a huge file can be pointed at
it to exercise their edge-case handling.

## Why

Real files are boring. They end. They fit in RAM. They don't live at
offset `2^63`. Testing a hex editor's scrollbar, a backup tool's sparse
detection, or a `pread` loop's chunking logic needs a file that:

- is enormous (larger than any sane test machine's disk),
- is reproducible (same bytes at the same offset, every time),
- supports random access without generating everything before the offset,
- has metadata (mtime, perms, uid/gid) you can mutate at will.

`randfs` gives you all four. Each seed file becomes one virtual file
whose bytes are derived from the seed via ChaCha20.

## Features

- **Seekable PRNG.** `read(offset, size)` is O(size), not O(offset).
  Backed by ChaCha20 with a per-2^32-byte-chunk nonce, so the full
  `u64` address space is addressable without counter overflow.
- **Deterministic.** Same seed, same offset, same bytes. Forever.
- **Metadata mirroring.** Each virtual file reports the seed file's
  mtime, ctime, perms, uid, and gid. `st_size` and `st_blocks` are
  overridden to `i64::MAX` (the file looks dense and bottomless).
- **Live attribute updates.** Zero TTL means `stat` re-reads the seed
  file every time, so `touch` / `chmod` on the ghost dir are reflected
  immediately.
- **Read-only.** Mounted `RO`; `open(O_WRONLY)` returns `EACCES`.
- **Per-thread read counters.** Kept for observing FUSE worker-thread
  dispatch under load. Pair with `--n-threads N` and (on Linux)
  `--clone-fd` to see how the kernel spreads requests.

## Build

```sh
cargo build --release
```

Requires a working FUSE setup:

- **Linux:** `fusermount3` (or `fusermount`), and you must be in the
  `fuse` group or run as root.
- **macOS:** [macFUSE](https://osxfuse.github.io/) installed.

## Usage

Create a ghost directory with one or more 32-byte seed files:

```sh
mkdir -p /tmp/ghost /tmp/rand
head -c 32 /dev/urandom > /tmp/ghost/alpha.bin
head -c 32 /dev/urandom > /tmp/ghost/beta.bin
```

Mount (this blocks):

```sh
./target/release/randfs /tmp/rand --ghost-dir /tmp/ghost
```

In another terminal:

```sh
ls -la /tmp/rand
stat /tmp/rand/alpha.bin       # size = 9223372036854775807
xxd /tmp/rand/alpha.bin | head # pseudorandom bytes
```

Unmount:

```sh
umount /tmp/rand              # Linux
diskutil unmount /tmp/rand    # macOS
```

## CLI

```
randfs [OPTIONS] <MOUNT_POINT> --ghost-dir <DIR>

Arguments:
  <MOUNT_POINT>              Where to mount the filesystem

Options:
      --ghost-dir <DIR>      Directory of 32-byte seed files
      --auto-unmount         Automatically unmount on process exit
      --allow-root           Allow the root user to access the filesystem
      --n-threads <N>        Number of worker threads [default: 1]
      --clone-fd             Use FUSE_DEV_IOC_CLONE so each worker thread
                             gets its own fd (Linux 4.5+)
  -h, --help                 Print help
  -V, --version              Print version
```

Each regular file (or symlink-to-file) in `--ghost-dir` becomes one
virtual file at the mount root, named after the seed file. Subdirectories
are ignored.

## How it works

### The PRNG

ChaCha20 in IETF mode (96-bit nonce, 32-bit block counter) gives you a
pseudorandom stream you can jump into at any byte offset via
`StreamCipherSeek::seek`. One catch: the 32-bit counter only covers
`2^38` bytes (~256 GiB) before it wraps.

`randfs` handles this by splitting the `u64` address space into
`2^32`-byte chunks. Chunk `N` uses the same key (the seed) and a nonce
derived from `N`. Within a chunk, the byte offset is always `< 2^32`,
which fits the counter comfortably.

```
offset 0x1_0000_0000  →  chunk 1, in-chunk offset 0x0000_0000
offset 0x2_FFFF_FFFF  →  chunk 2, in-chunk offset 0xFFFF_FFFF
```

Reads that straddle a chunk boundary are split and stitched together
transparently in `fill_random`.

### Inode layout

| Inode | Meaning                          |
|------:|----------------------------------|
|     1 | Root directory                   |
|  2..N | One per seed file, sorted by name |

Inodes are assigned by `readdir` order after sorting by filename, so
they're stable across mounts regardless of the underlying filesystem's
directory ordering.

### Attribute overrides

`attr_for` stats the seed file and copies its `mode`, `uid`, `gid`,
`atime`, `mtime`, and `ctime` into the returned `FileAttr`, then
overwrites:

- `size = i64::MAX` (not `u64::MAX` — signed arithmetic in userspace
  tools breaks above `i64::MAX`)
- `blocks = i64::MAX / 512` (looks dense, so sparse-aware tools still
  read us)

If the seed file is deleted while mounted, the attribute fallback is
`(UNIX_EPOCH, 0o444, uid=501, gid=20)`. Reads still succeed because the
seed is cached in memory at startup.

## Edge cases handled

| Case | Behavior |
|---|---|
| `read` at `offset >= i64::MAX` | Empty reply |
| `read` near `i64::MAX` | Capped to remaining bytes |
| Read straddling a `2^32`-byte PRNG chunk | Split and stitched correctly |
| Pathological `size` from the kernel | Capped at 1 MiB per call |
| Zero-length read | Empty reply |
| `getattr` on the root inode | Returns `ROOT_ATTR` |
| `lookup` with non-root parent | `ENOENT` |
| `readdir` on a non-directory | `ENOTDIR` |
| `open` with `O_WRONLY` / `O_RDWR` | `EACCES` |
| Seed file has wrong size | Refuse to mount with a clear error |
| Seed file deleted after mount | Attrs fall back; reads still work |
| Seed file symlinked | Followed (`metadata()` not `file_type()`) |
| `readdir` past the end | Empty iteration, `ok` |
| Seed dir missing | Exit before mounting with an error |

## Testing recipes

**Determinism across offsets:**

```sh
A=$(dd if=/tmp/rand/alpha.bin bs=1 skip=1000000 count=64 2>/dev/null | md5)
B=$(dd if=/tmp/rand/alpha.bin bs=1 skip=1000000 count=64 2>/dev/null | md5)
[ "$A" = "$B" ] && echo OK
```

**Chunk-boundary continuity:**

```sh
# 128 bytes starting 64 before the 2^32 boundary
dd if=/tmp/rand/alpha.bin bs=1 skip=4294967232 count=128 2>/dev/null \
    | dd bs=1 skip=64 count=64 2>/dev/null | md5
# 64 bytes starting at the boundary
dd if=/tmp/rand/alpha.bin bs=1 skip=4294967296 count=64 2>/dev/null | md5
# These must match.
```

**Near-EOF behavior:**

```sh
dd if=/tmp/rand/alpha.bin bs=1 skip=9223372036854775744 count=64 2>/dev/null | wc -c
# 64
dd if=/tmp/rand/alpha.bin bs=1 skip=9223372036854775807 count=64 2>/dev/null | wc -c
# 0
```

**Live attribute changes:**

```sh
touch /tmp/ghost/alpha.bin
stat /tmp/rand/alpha.bin          # mtime updates immediately
chmod 600 /tmp/ghost/alpha.bin
stat /tmp/rand/alpha.bin          # perm updates immediately
```

**Thread dispatch diagnostics (Linux):**

```sh
./target/release/randfs /tmp/rand --ghost-dir /tmp/ghost \
    --n-threads 8 --clone-fd
# Then hammer the file from multiple processes and watch the
# per-thread counters diverge — see notes below.
```

## Non-goals

- **Writing.** The filesystem is read-only by design.
- **Real file content.** Bytes are derived from the seed; the seed
  file's actual contents are only used as key material.
- **Performance.** The per-thread counters and the `Vec` rebuild in
  `readdir` are fine for a test harness and wasteful in production.
- **Stability.** No promises about mount behavior under memory
  pressure, sudden `SIGKILL`, or exotic kernels.

## Credits

The FUSE boilerplate (`common/args.rs`, the `CommonArgs` config
plumbing, and the original stub) is adapted from the
[`fuser` examples](https://github.com/cberner/fuser).

## License

See `LICENSE` in the repository root.
