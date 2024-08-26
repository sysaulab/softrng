#include <stdint.h>

char _files_module_softrng[] = "\n\
s-system     cat /dev/random";

char _files_module_RNG_test[] = "\n\
t-pract      RNG_test stdin\n\
t-pract8     RNG_test stdin8\n\
t-pract16    RNG_test stdin16\n\
t-pract32    RNG_test stdin32\n\
t-pract64    RNG_test stdin64";

char _files_module_RNG_output[] = "\n\
s-jsf32      RNG_output jsf32 inf\n\
s-jsf64      RNG_output jsf64 inf\n\
s-sfc32      RNG_output sfc32 inf\n\
s-sfc64      RNG_output sfc64 inf\n\
s-xsm32      RNG_output xsm32 inf\n\
s-xsm64      RNG_output xsm64 inf\n\
s-arbee      RNG_output arbee inf\n\
s-hc256      RNG_output hc256 inf\n\
s-trivium    RNG_output trivium inf";

char _files_module_dieharder[] = "\n\
t-test-dieharder               dieharder -a -g 200\n\
t-test-dieharder-destruction   dieharder -a -y 2 -k -g 200\n\
t-test-dieharder-resolve       dieharder -a -y 1 -g 200";

char _files_manual[] = "\n\
# MANUAL - SOFTRNG version 0.1\n\
\n\
## RNG made FUN and EASY\n\
\n\
This program is free software; you can redistribute it and/or\n\
modify it under the terms of the Affero GNU Public Licence version 3.\n\
Other licences available upon request.\n\
\n\
Below is the list of basic commands to play with. The IO descriptor indicate\n\
if the command require an input or provide an output. By using simple shell \n\
pipe, we create combination of generator(s) entropy source(s) and all kind of\n\
other fun things to observe.\n\
\n\
-------------------------------------------------------------------------------\n\
\n\
COMMANDS\n\
\n\
    1. Commands are chained together using pipes.\n\
    2. A chain must start with a source ( s- ).\n\
    3. Any number of filters ( f- ) can be chained\n\
    4. Without a terminator ( -t ) streams are displayed unformatted.\n\
\n\
   _ Type\n\
 /\n\
|   s = Source (nothing in, something out)\n\
|   f = Filter (something in, something out)\n\
|   t = Target (something in, nothing out)\n\
|\n\
|   s-source | f-filter-1 | f-filter-2 | t-target\n\
|\n\
|  INTERNAL     DESCRIPTION\n\
|  -----------------------------------------------------------------------------\n\
s-password P    Use a password to generate a sequence.\n\
s-zeros         Portable source of infinite zeros. (/dev/zero)\n\
s-synth         Portable source of synthetic entropy.\n\
s-random        Read from the default system generator. (/dev/random)\n\
s-getent        Native quality entropy source ( getentropy() )\n\
s-file F        Read file F to output.\n\
f-peek          Display information about the stream.\n\
f-hex           Convert input to hexadecimal.\n\
f-limit N       Limit the size of a stream to N bytes\n\
f-file F        Save a copy of the the stream to file F.\n\
f-xor S         Insert stream S using XOR.\n\
f-qxo64         2gb/sec, 2mb entropy, 140eb.\n\
f-roxo64        1gb/sec, 8kb entropy, 512pt.\n\
t-hole          Bottomless pit en lost data. (portable /dev/null)\n\
t-bspec32       32 bit completeness test.\n\
| -----------------------------------------------------------------------------\n\
| PRACTRAND     DESCRIPTION\n\
| -----------------------------------------------------------------------------\n\
s-jsf32         generator included with practrand : jsf32\n\
s-jsf64         generator included with practrand : jsf64\n\
s-sfc32         generator included with practrand : sfc32\n\
s-sfc64         generator included with practrand : sfc64\n\
s-xsm32         generator included with practrand : xsm32\n\
s-xsm64         generator included with practrand : xsm64\n\
s-arbee         generator included with practrand : arbee\n\
s-hc256         generator included with practrand : hc256\n\
s-trivium       generator included with practrand : trivium\n\
t-pract         PractRand test, standard options\n\
t-pract8        PractRand test, standard options, 8 bit folds\n\
t-pract16       PractRand test, standard options, 16 bit folds\n\
t-pract32       PractRand test, standard options, 32 bit folds\n\
t-pract64       PractRand test, standard options, 64 bit folds\n\
| -----------------------------------------------------------------------------\n\
| DIEHARDER     DESCRIPTION\n\
| -----------------------------------------------------------------------------\n\
t-test-dieharder     Dieharder test, regular settings\n\
t-test-dieharder     Dieharder test, regular settings\n\
t-test-dieharder     Dieharder test, regular settings\n\
\n\
\n\
\n\
4. TUTORIAL\n\
\n\
VERY IMPORTANT\n\
At any time, if you need to stop a command,\n\
type CTRL-C in the terminal running the command.\n\
\n\
If you are not sure how this works, follow this tutorial. It will get you \n\
through the concepts one at a time with examples.\n\
\n\
The commands are to be typed in your regular shell. Once installed, the short\n\
default commands will be available to all users in all sessions. I think of\n\
it as an extension of the shell. Basic knowledge of the shell is not necessary\n\
but it is recommended.\n\
\n\
s-icm64\n\
    s-icm64 have an IO class of O. It means it do not require an stream to \n\
    function properly. The O indicates it is providing output. If the output is\n\
    not captured it will be displayed on the screen raw binary form. Executing \n\
    this command can make your terminal do\n\
    crazy things! Go ahead, give it a try!\n\
\n\
s-icm64 | f-hex\n\
    Display the stream generated by input converted in hexadecimal format.\n\
\n\
g-zeros | f-hex\n\
    Display enless zeros!\n\
\n\
g-system | f-size 1m > file.bin\n\
    Get random numbers from /dev/random, limit the size of the stream to \n\
    1000000 bytes and save the content to a file: file.bin. \n\
    \n\
    Note: If the file exist alrealy, it will be overwritten. To add data to an\n\
    existing file, replace > with >>. There is no undo with this, everything\n\
    is permanent, play with care!\n\
\n\
    Try to change the '1m' with 1M and observe the precise file size of each one.\n\
\n\
s-icm64 | f-bench | g-roxo64 | f-void\n\
    The f-bench command will display information about the data going through it.\n\
    INFO relay the information untouched with the minimal amount of overhead\n\
    portably practical. In this configuration it will display the bandwidth\n\
    used by the seeding node, not the output.\n\
    \n\
    Can you change it to make it display information about final output?\n\
\n\
s-icm64 | f-write seed.bin | g-roxo64 | f-write rand.bin | f-void\n\
    The f-output command pass data and save a copy to a file. This allow the\n\
    possibility of keeping a copy of the seed generated and the final stream.\n\
    This can help debug and understand bad test results, or satisfy\n\
    your curiosity.\n\
\n\
\n\
\n\
Enjoy softrng! And remember, every files are overwritten without warnings.\n\
Be careful.\n\
\n\
GITHUB : https://github.com/sysaulab/softrng\n\
PATREON : https://www.patreon.com/sofTRNG\n\
";
