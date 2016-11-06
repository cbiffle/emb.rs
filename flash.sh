#!/bin/sh -e

ROOT="$(dirname "$0")"

openocd -f board/stm32f4discovery.cfg \
  -c "init" \
  -c "reset init" \
  -c "flash write_image erase target/thumbv7em-none-eabihf/release/emb1" \
  -c "reset halt" \
  -c "resume" \
  -c "shutdown"
