#!/bin/sh
echo "Please note the following value after it stabilizes. (f-limit)"
f-peek < /dev/zero | f-limit 1T > /dev/null 
