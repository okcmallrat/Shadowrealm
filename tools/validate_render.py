"""Compare all native map crops with a separate Python plane renderer.
Uses original ROM M68000 decompression for tagged definitions, via Unicorn 2.1.4.
This is not a comparison to a running console or emulator screenshot.
"""
import argparse
import hashlib
from pathlib import Path
import struct
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
    p.error('reference checks require the supplied ROM revision')
def word(offset):
    return struct.unpack_from('>H', b, offset)[0]
def long(offset):
    return struct.unpack_from('>I', b, offset)[0]
def decode(source):
    cpu = Uc(UC_ARCH_M68K, UC_MODE_BIG_ENDIAN)
    cpu.ctl_set_cpu_model(UC_CPU_M68K_M68000)
    cpu.mem_map(0, 0x200000)
    cpu.mem_write(0, b)
    cpu.mem_map(0x400000, 0x20000)
    cpu.mem_map(0xff0000, 0x10000)
    for reg, value in [(UC_M68K_REG_SR, 0x2700), (UC_M68K_REG_A7, 0xffff00),
                       (UC_M68K_REG_A3, source), (UC_M68K_REG_A1, 0x400000)]:
        cpu.reg_write(reg, value)
    cpu.emu_start(0x1610, 0x1670, count=2_000_000)
    if cpu.reg_read(UC_M68K_REG_PC) != 0x1670:
        raise RuntimeError('original decompressor did not terminate')
    return bytes(cpu.mem_read(0x400000, word(source)+1))
binary = Path(__file__).resolve().parents[1] / 'target/release/shadowrealm'
def reference(index, origin_x, origin_y):
    desc = long(0x3232+index*4)
    setup = long(desc+0x34)
    palette = []
    for i in range(64):
        color = word(long(setup+4)+i*2)
        palette.append(bytes(((color >> shift) & 7)*255//7 for shift in (1, 5, 9)))
    # $566c2 replaces the last 48 target colors after setup finishes.
    for i in range(16, 64):
        color = word(0x18ac56+(i-16)*2)
        palette[i] = bytes(((color >> shift) & 7)*255//7 for shift in (1, 5, 9))
    regs = long(setup)
    backdrop = next((word(regs+8+i*2) & 63 for i in range(15)
                     if word(regs+8+i*2) >> 8 == 0x87), 0)
    vram = bytearray(65536)
    cursor = setup+16
    while long(cursor):
        source, dest, length = struct.unpack_from('>III', b, cursor)
        cursor += 12
        payload = decode(source & ~1) if source & 1 else b[source:source+length]
        vram[dest:dest+len(payload)] = payload
    first = long(desc+0x18)
    if first & 1:
        definition_data = decode(first & ~1)
        definitions = [definition_data, definition_data]
    else:
        definitions = [b[long(desc+offset):] for offset in (0x18, 0x20)]
    stride = word(desc+0x24)
    grids = [long(desc+offset) for offset in (0x14, 0x1c)]
    output = bytearray()
    tiles = {}
    for y in range(origin_y, origin_y+224):
        for x in range(origin_x, origin_x+320):
            best = (-1, palette[backdrop])
            for layer in range(2):
                block = word(grids[layer]+(y//32)*stride+(x//32)*2)
                offset = block*32+((y//8 % 4)*4+(x//8 % 4))*2
                attribute = struct.unpack_from('>H', definitions[layer], offset)[0]
                attribute ^= 0x8000 if layer == 1 else 0
                tile_id = attribute & 2047
                if tile_id not in tiles:
                    raw = vram[tile_id*32:tile_id*32+32]
                    tiles[tile_id] = [v for byte in raw for v in (byte >> 4, byte & 15)]
                tx, ty = x % 8, y % 8
                if attribute & 2048:
                    tx = 7-tx
                if attribute & 4096:
                    ty = 7-ty
                color = tiles[tile_id][ty*8+tx]
                rank = (2 if attribute & 0x8000 else 0)+layer
                if color and rank > best[0]:
                    best = (rank, palette[((attribute >> 13) & 3)*16+color])
            output.extend(best[1])
    return b'P6\n320 224\n255\n'+output
with tempfile.TemporaryDirectory() as folder:
    for index in range(56):
        # Off-tile origins exercise pixel cropping and flips, including compressed maps.
        desc = long(0x3232+index*4)
        x, y = (0, 0) if word(desc+0x24) == 20 else (7, 11)
        output = Path(folder) / f'{index}.ppm'
        subprocess.run([str(binary), 'render-map', str(a.rom.resolve()), str(index),
                        str(x), str(y), str(output)], check=True, capture_output=True)
        expected = reference(index, x, y)
        if output.read_bytes() != expected:
            raise RuntimeError(f'map {index}: pixels differ')
print('PASS: all 56 map crops, 4,014,080 pixels, match independent renderer with original M68000 decompression')
