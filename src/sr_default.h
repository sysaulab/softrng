char _files_module_softrng[] = "\n\
s-password   srs_password\n\
f-hex        srf_hex\n\
f-bench      srf_bench\n\
f-size       srf_limitsize\n\
t-hole       srt_hole\n\
s-read       srs_fread\n\
f-write      srf_fwrite\n\
f-xor        srf_xor\n\
s-zeros      srs_zeros\n\
f-roxo64     srg_roxo64\n\
f-qxo64      srg_qxo64\n\
s-system     s-read /dev/random\n\
s-icm64      srs_icm64\n\
t-bspec32    srt_bspec32\n\
s-getent     srs_getent";

char _files_module_RNG_test[] = "\n\
t-pract      RNG_test stdin\n\
t-pract8     RNG_test stdin8\n\
t-pract16    RNG_test stdin16\n\
t-pract32    RNG_test stdin32\n\
t-pract64    RNG_test stdin64";

char _files_module_RNG_output[] = "\n\
g-jsf32      RNG_output jsf32 inf\n\
g-jsf64      RNG_output jsf64 inf\n\
g-sfc32      RNG_output sfc32 inf\n\
g-sfc64      RNG_output sfc64 inf\n\
g-xsm32      RNG_output xsm32 inf\n\
g-xsm64      RNG_output xsm64 inf\n\
g-arbee      RNG_output arbee inf\n\
g-hc256      RNG_output hc256 inf\n\
g-trivium    RNG_output trivium inf";

char _files_module_dieharder[] = "\n\
t-die               dieharder -a -g 200\n\
t-die-destruction   dieharder -a -y 2 -k -g 200\n\
t-die-resolve       dieharder -a -y 1 -g 200";

char _files_manual[] = "\n\
MANUAL - SOFTRNG version 0.1\n\
\n\
RNG made FUN and EASY\n\
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
\n\
\n\
1. INSTALL THE COMMANDS\n\
\n\
    To install the commands type this in the command line : sudo softrng setup\n\
\n\
\n\
\n\
2. RULES OF THE COMMANDS\n\
\n\
    1. The commands are chained together using pipes ( cmd1 | cmd2 )\n\
    \n\
    2. A chain cannot start with a pipe\n\
    \n\
    5. If the chain is not terminated, it will be displayed on your terminal.\n\
\n\
\n\
\n\
3. LIST OF COMMANDS\n\
\n\
T: Type of command, this tell you about the role of the command.\n\
    g: Generators, require seeding but fast\n\
    s: Seeders, initial source of entropy\n\
    f: Filters, data manipulation\n\
    t: Tests, verify how well it does\n\
\n\
I: Input, this command is expecting data, it must follow a pipe:   | cmd.\n\
O: Output, this command is providing data, it must precede a pipe: cmd |.\n\
\n\
\n\
\n\
COMMANDS INCLUDED WITH SOFTRNG\n\
\n\
The type of command tells you about \n\
\n\
   _ type\n\
 /\n\
|  COMMAND  DESCRIPTION\n\
----------  ----------------------------------------------------------------\n\
s-password  get user input and pass it to output\n\
f-bench     display live information about the stream\n\
f-hex       convert binary input to hexadecimal\n\
s-read F    read file F and send it to the output pipe\n\
f-size X    close a stream after X bytes\n\
t-void      dummy target, sink to nothing, useful when benchmarking\n\
f-write     duplicate the stream and save a copy to file F\n\
f-xor X     mix the stream with output of command chain X using XOR\n\
s-random    default system generator (/dev/random)\n\
f-qxo64     very fast, require very large seed.\n\
f-roxo64    1GB/sec, 512PT period, 8kb seed required.\n\
s-zeros     source of infinite zeros (/dev/zero)\n\
s-icm64     portable, software entropy collector ( libICM )\n\
s-getent    native entropy source ( getentropy() )\n\
t-bspec32   32 bit completeness test\n\
               _ Type (Filter, Generator, Test, Source)\n\
             /  _ Input\n\
            | /  _ Output\n\
            || /\n\
COMMAND     TIO  DESCRIPTION\n\
----------  ---  ----------------------------------------------------------------\n\
f-ask       F O  get user input and pass it to output\n\
f-bench     FIO  display live information about the stream\n\
f-hex       FI   convert binary input to hexadecimal\n\
s-read F    S O  read file F and send it to the output pipe\n\
f-size X    FIO  close a stream after X bytes\n\
f-void      TI   dummy target, sink to nothing, useful when benchmarking\n\
f-write     F O  duplicate the stream and save a copy to file F\n\
f-xor X     FIO  mix the stream with output of command chain X using XOR\n\
g-rand      S O  default system generator (/dev/random)\n\
g-qxo64 T   GIO  very fast, require very large seed. T = multi-threaded.\n\
g-roxo64    GIO  1GB/sec, 512PT period, 8kb seed required.\n\
s-zeros     G O  source of infinite zeros (/dev/zero)\n\
s-icm64     S O  portable, software entropy collector ( libICM )\n\
s-getent    S O  native entropy source ( getentropy() )\n\
t-bspec32   TI   basic 32 bit distribution test\n\
\n\
\n\
\n\
COMMANDS REQUIRING PRACTRAND\n\
\n\
COMMAND    TIO DESCRIPTION\n\
---------- --- -- -------------------------------------------------------------\n\
g-jsf32    G O generator included with practrand : jsf32\n\
g-jsf64    G O generator included with practrand : jsf64\n\
g-sfc32    G O generator included with practrand : sfc32\n\
g-sfc64    G O generator included with practrand : sfc64\n\
g-xsm32    G O generator included with practrand : xsm32\n\
g-xsm64    G O generator included with practrand : xsm64\n\
g-arbee    G O generator included with practrand : arbee\n\
g-hc256    G O generator included with practrand : hc256\n\
g-trivium  G O generator included with practrand : trivium\n\
t-pract    TI  PractRand test, standard options\n\
t-pract8   TI  PractRand test, standard options, 8 bit folds\n\
t-pract16  TI  PractRand test, standard options, 16 bit folds\n\
t-pract32  TI  PractRand test, standard options, 32 bit folds\n\
t-pract64  TI  PractRand test, standard options, 64 bit folds\n\
---------- --- ----------------------------------------------------------------\n\
\n\
\n\
\n\
COMMANDS REQUIRING DIEHARDER\n\
\n\
COMMAND    TIO DESCRIPTION\n\
---------- --- ----------------------------------------------------------------\n\
t-die      TI  Dieharder test\n\
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
