# simpple-vm

**simpple-vm** is a Rust-based implementation of a minimal Virtual Machine Monitor (VMM) designed for Apple Silicon processors, utilizing Apple's Hypervisor framework as the underlying virtualization backend.

## RoadMaps

- [x] Run simple arm64 binaries

Our next step is to run the ARM64 U-Boot binary contained in [QEMU debian package](http://ftp.debian.org/debian/pool/main/u/u-boot/u-boot-qemu_2025.01-3_all.deb).

To successfully run U-Boot, we need to implement the following features:

- [x] Simple MMIO support
- [x] Emulate PL011 UART
- [ ] Emulate GIC
- [ ] Emulate PL031 RTC
- [ ] Emulate PL061 GPIO
- [ ] Emulate PL041 AACI
- [ ] Run U-Boot

At the same time, we need to enrich features for this VMM:

- [x] Debugger to show registers and recent instructions
- [ ] Single step debugger

## Status

Right now, it can print out:

```
U-Boot 2025.01-3 (Apr 08 2025 - 23:07:41 +0000)

DRAM:  1 GiB
Core:  11 devices, 5 uclasses, devicetree: board
Flash: 0 Bytes
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff000: invalid read from 0x3ffff000
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff040: invalid read from 0x3ffff040
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff048: invalid read from 0x3ffff048
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff050: invalid read from 0x3ffff050
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff058: invalid read from 0x3ffff058
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff060: invalid read from 0x3ffff060
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff068: invalid read from 0x3ffff068
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff070: invalid read from 0x3ffff070
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff071: invalid read from 0x3ffff071
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff072: invalid read from 0x3ffff072
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff040: invalid read from 0x3ffff040
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff041: invalid read from 0x3ffff041
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff042: invalid read from 0x3ffff042
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff043: invalid read from 0x3ffff043
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff044: invalid read from 0x3ffff044
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff045: invalid read from 0x3ffff045
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff046: invalid read from 0x3ffff046
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff047: invalid read from 0x3ffff047
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff048: invalid read from 0x3ffff048
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff049: invalid read from 0x3ffff049
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff04a: invalid read from 0x3ffff04a
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff04b: invalid read from 0x3ffff04b
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff04c: invalid read from 0x3ffff04c
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff04d: invalid read from 0x3ffff04d
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff04e: invalid read from 0x3ffff04e
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff04f: invalid read from 0x3ffff04f
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff050: invalid read from 0x3ffff050
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff051: invalid read from 0x3ffff051
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff052: invalid read from 0x3ffff052
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff053: invalid read from 0x3ffff053
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff054: invalid read from 0x3ffff054
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff055: invalid read from 0x3ffff055
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff056: invalid read from 0x3ffff056
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff057: invalid read from 0x3ffff057
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff058: invalid read from 0x3ffff058
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff059: invalid read from 0x3ffff059
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff05a: invalid read from 0x3ffff05a
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff05b: invalid read from 0x3ffff05b
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff05c: invalid read from 0x3ffff05c
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff05d: invalid read from 0x3ffff05d
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff05e: invalid read from 0x3ffff05e
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff05f: invalid read from 0x3ffff05f
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff06f: invalid read from 0x3ffff06f
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff070: invalid read from 0x3ffff070
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff071: invalid read from 0x3ffff071
[2025-08-01T17:54:31Z ERROR simpple_vm] Unmapped memory access at address 0x000000003ffff072: invalid read from 0x3ffff072
"Synchronous Abort" handler, esr 0x96000004, far 0x58fffe2110fffe00
elr: 00000000000a1b70 lr : 000000000006e9ac (reloc)
elr: 000000007f758b70 lr : 000000007f7259ac
x0 : 58fffe2110fffe00 x1 : 000000007f7779e0
x2 : 0000000000000010 x3 : 0000000000000007
x4 : 0000000000000000 x5 : 58fffe2110fffe00
x6 : 000000003fffd000 x7 : 0000000000000004
x8 : 000000003fffe000 x9 : 0000000000000001
x10: 000000007e6779c0 x11: 000000003fffdfff
x12: 000000007f7dc890 x13: 0000000000000018
x14: fffffffffffff000 x15: 000000007e6779c0
x16: 000000007f702298 x17: 0000000000000000
x18: 000000007e676df0 x19: 0000000000000000
x20: 000000007f6b7f68 x21: 000000007f7779e0
x22: 0000000000000000 x23: 000000007e677a40
x24: 000000007f7a1308 x25: 000000003fffd000
x26: 000000007e677f90 x27: 000000003fffe000
x28: 000000007f7a1308 x29: 000000007e676890

Code: eb04005f 54000061 52800000 14000006 (386468a3)
Resetting CPU ...

resetting ...
System reset not supported on this platform
### ERROR ### Please RESET the board ###
^C
```

We have many unmapped memory accesses because some devices are not simulated yet.
