"""Compare extracted maps with ROM bytes and the original M68000 decompressor.
Requires unicorn==2.1.4 for validation only (not needed by the Rust program).
"""
import argparse
import hashlib
from pathlib import Path
import subprocess
import tempfile
from unicorn import Uc, UC_ARCH_M68K, UC_MODE_BIG_ENDIAN
from unicorn.m68k_const import (UC_CPU_M68K_M68000, UC_M68K_REG_SR,
    UC_M68K_REG_A7, UC_M68K_REG_A3, UC_M68K_REG_A1, UC_M68K_REG_PC)
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('rom', type=Path)
a = p.parse_args()
b = a.rom.read_bytes()
if hashlib.sha256(b).hexdigest() != '92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e':
    p.error('validation requires the supplied ROM revision')
binary = Path(__file__).resolve().parents[1] / 'target/release/shadowrealm'
def u32(offset):
    return int.from_bytes(b[offset:offset+4], 'big')
compressed = 0
with tempfile.TemporaryDirectory() as temp:
    for index in range(56):
        folder = Path(temp) / str(index)
        result = subprocess.run([str(binary), 'map', str(a.rom.resolve()), str(index), str(folder)],
                                capture_output=True, text=True, check=True)
        ptr = u32(0x3232+index*4)
        if folder.joinpath('descriptor.bin').read_bytes() != b[ptr:ptr+96]:
            raise RuntimeError(f'map {index}: descriptor differs')
        for layer, offset in [(0, 0x14), (1, 0x1c)]:
            grid = u32(ptr+offset)
            if folder.joinpath(f'layer{layer}-copy.bin').read_bytes() != b[grid:grid+0x1c20]:
                raise RuntimeError(f'map {index}: grid copy differs')
        tiles = u32(ptr+0x18)
        if tiles & 1:
            source = tiles & ~1
            length = int.from_bytes(b[source:source+2], 'big')+1
            cpu = Uc(UC_ARCH_M68K, UC_MODE_BIG_ENDIAN)
            cpu.ctl_set_cpu_model(UC_CPU_M68K_M68000)
            cpu.mem_map(0, 0x200000)
            cpu.mem_write(0, b)
            cpu.mem_map(0x400000, 0x20000)
            cpu.mem_map(0xff0000, 0x10000)
            cpu.reg_write(UC_M68K_REG_SR, 0x2700)
            cpu.reg_write(UC_M68K_REG_A7, 0xffff00)
            cpu.reg_write(UC_M68K_REG_A3, source)
            cpu.reg_write(UC_M68K_REG_A1, 0x400000)
            # Execute the real ROM routine through its exit before register restore.
            cpu.emu_start(0x1610, 0x1670, count=2_000_000)
            if cpu.reg_read(UC_M68K_REG_PC) != 0x1670:
                raise RuntimeError(f'map {index}: original routine did not finish')
            if cpu.reg_read(UC_M68K_REG_A1) != 0x400000+length:
                raise RuntimeError(f'map {index}: original output length differs')
            expected = bytes(cpu.mem_read(0x400000, length))
            if folder.joinpath('shared-tile-definitions.bin').read_bytes() != expected:
                raise RuntimeError(f'map {index}: native decompression differs from M68000 execution')
            consumed = cpu.reg_read(UC_M68K_REG_A3)-source
            if f'from {consumed} source bytes' not in result.stdout:
                raise RuntimeError(f'map {index}: source consumption differs')
            compressed += 1
print(f'PASS: 56 descriptors, 112 grid copies, {compressed} compressed streams compared with original M68000 execution')
