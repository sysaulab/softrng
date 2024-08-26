# SoftRNG
![]()

Softrng is a collection of basic portable tools that can be used to explore random number generators and cryptography. It comes with basic algorithms and integrates with other software packages such as dieharder and practrand. The manual is designed to help the user ease into the principles of basic cryptography building blocks. The information database will contain extensive documentation about the algorithms, their flaws and strengths, their history...

It follows the UN*X philosophy, each tool have a very specific function.

- Generator's output are all endless and binary. 
- Filters are used to manipulate the stream(s).
- Simple syntax. Every command start with one of three letters and a dash (s- f- t-).

The tools combine together to let the user create their own solutions or observe different algorithms. It requires no knowledge of coding to get started. Commands themselves are small and focused.

## Installation

SoftRNG assumes a Unix-like environment and a C compiler. 

If you are using Windows, please install Linux first. SoftRNG has been developed and tested primarily under macOS, I tried to remain as close to standard as possible. 

Dieharder on macOS require to have libtool installed. You can install it using "[brew](https://brew.sh) install libtool". If you install dieharder after softrng, run "softrng refresh" to create the shortcuts.

1. Open a terminal.
1. Go to the folder containing this note.
1. Run "./install.sh" as root (sudo, su).

## To do

Document every available command in a text file placed in the help directory (s-system.txt...).

## Future developments

A graphical command builder could be a good teaching aid. Something online maybe? A portable GUI app that handles the piping internally?. Games where you have to solve cryptographic challenges, match a specific sequence maybe?

Anyone can help, drop a message in the discusison board with your comments or suggestions.