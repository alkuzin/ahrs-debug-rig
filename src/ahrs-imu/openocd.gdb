# SPDX-License-Identifier: Apache-2.0.
# Copyright (C) 2026-present ahrs-debug-rig project and contributors.

target extended-remote :3333

# Print demangled symbols.
set print asm-demangle on

# Detect unhandled exceptions, hard faults and panics.
break DefaultHandler
break HardFault
break rust_begin_unwind

monitor arm semihosting enable
load
