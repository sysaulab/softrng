#!/bin/sh
echo "Please note the following value after it stabilizes. (cat)"
cat /dev/zero | f-peek > /dev/null
