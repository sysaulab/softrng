#!/bin/sh
mv -f release/* /usr/local/bin/
/usr/local/bin/softrng install
rm -rf release/*
rm -rf release/
