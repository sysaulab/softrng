#!/bin/sh
echo "Please note the following value after it stabilizes. (f-hex)"
f-peek < /dev/zero | f-hex > /dev/null 
