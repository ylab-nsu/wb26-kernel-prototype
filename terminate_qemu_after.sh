#!/bin/bash
# Wait in background until remote gdb connection is closed. Then kill qemu and exit.
# Finds gdb and qemu by port they are using (default 1234).
# May kill other programs if they are using same port.
# Suitable to use as pre-launch task for an IDE debugger

PORT=${1:-1234}

terminate_qemu_after() {
    sleep 1
    while lsof -Pi :"$PORT" -sTCP:ESTABLISHED -t >/dev/null 2>&1; do
        sleep 0.2
    done
    cleanup
}

cleanup() {
    lsof -ti:"$PORT" | xargs -r kill -TERM
}

trap cleanup INT TERM HUP
terminate_qemu_after &
