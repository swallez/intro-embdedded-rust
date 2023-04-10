#!/bin/sh

# This is MacOS-specific
device=$(ls /dev/cu.usbmodem* | head -n 1)

avrdude \
  -C .cargo/avrdude.conf \
  -V \
  -P "$device" \
  -p attiny85 \
  -c stk500v1 \
  -b 19200 \
  -U flash:w:"$1":e
