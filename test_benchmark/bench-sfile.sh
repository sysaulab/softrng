#!/bin/sh
echo "Please note the following value after it stabilizes. (s-file)"
s-file /dev/zero | f-peek > /dev/null 
