#define _SSRNG_BUFLEN       128                     // buffer size in number of uint64_t (apparent size of pipe buffers in macos)
#define _SSRNG_FPS          10                      // FPS in ui
#define _SSRNG_BUFSIZE      (_SSRNG_BUFLEN*8)       // buffer size in bytes for support commands, meant to match muffer of generators
