# SoftRNG

Softrng is a collection of basic portable tools that can be used to explore random number generators and cryptography. It comes with basic algorithms and integrates with other software packages such as dieharder and practrand. The manual is designed to help the user ease into the principles of basic cryptography building blocks. The information database will contain extensive documentation about the algorithms, their flaws and strengths, their history...

It follows the UN*X philosophy, each tool have a very specific function.

- Generator's output are all endless and binary. 
- Filters are used to manipulate the stream(s).
- Simple syntax. Every command start with one of three letters and a dash (s- f- t-).

The tools combine together to let the user create their own solutions or observe different algorithms. It requires no knowledge of coding to get started. Commands themselves are small and focused.

## Build

SoftRNG assumes a Unix-like environment. It requires a C compiler to build and the curl command to install the extra PractRand package. If you are using Windows, please install Linux first. SoftRNG has been developed and tested primarily under macOS, I tried to remain as close to standard as possible. 

If you encounter compilation errors or bugs on different platforms please me us know.

1. Run "./build.sh" to compile the commands.
2. The programs will be created in the "Release" directory

## Install
***Running this script as regular user will delete the Release folder without installing the programs properly. If this happens to you, run the build.sh script before trying again as root.***

1. Run as root "./install.sh" to install the commands. 
2. This will move the programs to /usr/local/bin and run "softrng install" to proceed with the initial setup.

The "softrng install" command will create the default support files in /etc/softrng before scanning for installed packages listed in /etc/softrng/modules and creating shell scripts that handle .

## To do

Document every available command in a text file placed in the help directory (s-system.txt...).

## Future developments

A graphical command builder could be a good teaching aid. Something online maybe? A portable GUI app that handles the piping internally?. Games where you have to solve cryptographic challenges, match a specific sequence maybe?

Anyone can help, drop a message in the discusison board with your comments or suggestions.