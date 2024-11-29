#! /usr/bin/bash

set -e

ca65 ./bios.s

ld65 -C ./bios.cfg ./bios.o -o bios.bin -Ln bios.sym

rm ./bios.o
