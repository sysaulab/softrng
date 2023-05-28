/**
* @file config.h
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Compile time configuration */

#define _SSRNG_BUFLEN       1024                    // buffer size in number of uint64_t (apparent size of pipe buffers in macos)
#define _SSRNG_FPS          5                      // FPS in ui

#define _SSRNG_BUFSIZE      (_SSRNG_BUFLEN*8)       // buffer size in bytes for support commands, meant to match muffer of generators
