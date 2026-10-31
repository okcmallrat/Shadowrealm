"""Compare the bounded Space Needle entry with original M68000 routines.

Requires the supplied ROM and Unicorn 2.1.4. Uses an explicit no-mission fixture,
not a complete new-game/quest VM or a full-game emulator. The original dialogue
UI, random NPC timer, AI and subsequent conversations are outside this check.
"""
import argparse
import csv
import hashlib
from pathlib import Path
import struct
import subprocess
import tempfile
from unicorn import Uc, UC_ARCH_M68K, UC_MODE_BIG_ENDIAN
from unicorn.m68k_const import *

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('rom', type=Path)
a = p.parse_args()
rom = a.rom.read_bytes()
if hashlib.sha256(rom).hexdigest() != '92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e':
    p.error('requires the supplied ROM revision')
root = Path(__file__).resolve().parents[1]

def long(address):
    return struct.unpack_from('>I', rom, address)[0]

def cpu():
    c = Uc(UC_ARCH_M68K, UC_MODE_BIG_ENDIAN)
    c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
    for base, size in [(0, 0x200000), (0x400000, 0x20000), (0xff0000, 0x10000), (0xc00000, 0x1000)]:
        c.mem_map(base, size)
    c.mem_write(0, rom)
    c.reg_write(UC_M68K_REG_SR, 0x2700)
    c.reg_write(UC_M68K_REG_A7, 0xffff00)
    c.mem_write(0xffff00, struct.pack('>I', 0x400000))
    return c

def putw(c, address, value):
    c.mem_write(address, struct.pack('>H', value))

def getw(c, address):
    return struct.unpack('>H', c.mem_read(address, 2))[0]

def execute(c, start, stop=0x400000):
    c.emu_start(start, stop, count=2_000_000)
    if c.reg_read(UC_M68K_REG_PC) != stop:
        raise RuntimeError(f'original routine {start:x} did not reach {stop:x}')

with tempfile.TemporaryDirectory() as temp:
    output = Path(temp) / 'interior'
    subprocess.run([str(root/'target/release/shadowrealm'), 'interior-data', str(a.rom.resolve()), str(output)], check=True)
    row = {key: int(value) for key, value in next(csv.DictReader((output/'plan.csv').open())).items()}
    c = cpu()
    c.mem_write(0xff0100, rom[0x7291a:0x72a1a])
    for address in [0xfffbec, 0xfffc0c]:
        putw(c, address, 0xffff)
    putw(c, 0xfffc30, 1)
    c.mem_write(0xfffc04, struct.pack('>I', 0xff0100))
    c.mem_write(0xfffbf8, bytes([row['source_map'], row['trigger'] & 31]))
    execute(c, 0x1f5ca, 0x1f246)
    assert c.reg_read(UC_M68K_REG_A0) == row['node'], 'exposition node differs'
    assert c.reg_read(UC_M68K_REG_A5) == 0x1220b4, 'portrait context differs'
    assert long(row['node']) == row['text'] and long(row['node']+4) == 0xffffffff
    text = rom[row['text']:].split(b'\xff', 1)[0].replace(b'\x80', b'\n')
    assert text == (output/'text.txt').read_bytes(), 'exposition text differs'
    assert bytes(c.mem_read(0xfff0d6, 1)) == b'\xff', 'entry permission differs'
    # Resume the post-dialogue continuation after acknowledgement, excluding UI.
    c.reg_write(UC_M68K_REG_A7, 0xffff00)
    c.reg_write(UC_M68K_REG_D6, 0)
    execute(c, 0x1fed8)
    assert c.reg_read(UC_M68K_REG_D6) & 0x8000 == 0, 'original denies entry'

    c = cpu()
    c.reg_write(UC_M68K_REG_A3, 0xc1064)
    c.reg_write(UC_M68K_REG_A1, 0x400000)
    execute(c, 0x1610, 0x1670)
    font = (output/'font.bin').read_bytes()
    assert bytes(c.mem_read(0x400000, len(font))) == font, 'font decompression differs'

    c = cpu()
    saved = (output/'saved-descriptor.bin').read_bytes()
    putw(c, 0xffddc4, row['source_map'])
    c.mem_write(0xffddc6, saved)
    putw(c, 0xff0100, row['player_x'])
    putw(c, 0xff0104, row['player_y'])
    c.mem_write(0xffef10, bytes([row['trigger']])*4)
    c.reg_write(UC_M68K_REG_A0, 0xff0100)
    execute(c, 0xbc36)
    returned = [row[f'return_{axis}{slot}'] for slot in range(3) for axis in ['x', 'y']]
    assert [getw(c, 0xffdf44+i*2) for i in range(6)] == returned, 'return formation differs'
    assert [getw(c, 0xffdf50+i*2) for i in range(2)] == [row['saved_camera_x'], row['saved_camera_y']]
    c.mem_write(0xfffbf9, bytes([row['trigger'] & 31]))
    execute(c, 0x167ae, 0x1683a)
    assert getw(c, 0xffddc4) == row['target_map'], 'location route differs'
    assert bytes(c.mem_read(0xffde26, 98)) == struct.pack('>H', row['source_map'])+saved, 'saved descriptor differs'
    target = long(0x3232+row['target_map']*4)
    c.mem_write(0xffddc6, rom[target:target+96])
    execute(c, 0x2b34, 0x2b76)
    assert [getw(c, 0xffddc6+i*2) for i in range(2)] == [row['camera_x'], row['camera_y']]
    assert [getw(c, 0xff0100+slot*256+offset) for slot in range(3) for offset in [0, 4]] == [row[f'{axis}{slot}'] for slot in range(3) for axis in ['x', 'y']], 'interior spawn differs'
    # Execute bounded restore blocks; exclude load/VDP/subsystem calls between them.
    execute(c, 0x2f5c, 0x2f72)
    execute(c, 0x2f82, 0x2f96)
    execute(c, 0x2fa4, 0x2fc2)
    assert bytes(c.mem_read(0xffddc6, 96)) == saved, 'restored descriptor differs'
    assert getw(c, 0xffddc4) == row['source_map']
    assert [getw(c, 0xff0100+slot*256+offset) for slot in range(3) for offset in [0, 4]] == returned, 'restored party differs'

    c = cpu()
    putw(c, 0xffdd40, 3)
    c.reg_write(UC_M68K_REG_A1, 0xff0400)
    c.reg_write(UC_M68K_REG_A2, 0x920)
    c.reg_write(UC_M68K_REG_A3, long(0x1701c))
    c.reg_write(UC_M68K_REG_A5, 0x17020)
    c.reg_write(UC_M68K_REG_D1, 0x49)
    c.reg_write(UC_M68K_REG_D4, long(0x17018) & 0x7fffffff)
    execute(c, 0x44bc, 0x45c6)
    execute(c, 0x45f4, 0x4616)
    expected = bytearray(c.mem_read(0xff0400, 256))
    # Native idle presentation selects facing and rate; NPC dispatcher/random
    # timers are intentionally unimplemented, not asserted as game-equivalent.
    struct.pack_into('>H', expected, 0x16, 8)
    struct.pack_into('>H', expected, 0xca, 30)
    native = (output/'npc.bin').read_bytes()
    assert expected == native, f'NPC record differs at {[hex(i) for i in range(256) if expected[i] != native[i]]}'

print('PASS: original entry gate/node/text/font, destination/spawn, saved exterior/return formation, and fixed NPC record match the bounded native path')
