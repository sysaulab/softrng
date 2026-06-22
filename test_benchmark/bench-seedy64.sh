#!/bin/sh
echo "Please note the following value after it stabilizes. (s-seedy)"
s-seedy64 | f-peek > /dev/null
