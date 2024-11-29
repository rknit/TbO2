#! /usr/bin/bash

set -e

cd ../msbasic

./make.sh

cp ./tmp/tbo2.bin ../asm/tbo2.bin

cd ../asm
