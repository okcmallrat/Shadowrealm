"""Check the small JSON conversation pilot against original M68000 execution.

Only the unlinked Space Needle/Trent contact and zero-opinion dialogue scope are
implemented. Original rendering, AI, targeting and hiring are outside this test.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile
from unicorn import Uc, UC_ARCH_M68K, UC_MODE_BIG_ENDIAN, UC_HOOK_CODE
from unicorn.m68k_const import *

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('rom', type=Path)
a = p.parse_args()
rom = a.rom.read_bytes()
if hashlib.sha256(rom).hexdigest() != '92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e':
    p.error('requires the supplied ROM revision')
root = Path(__file__).resolve().parents[1]

def long(p):
    return struct.unpack_from('>I', rom, p)[0]

def cpu():
    c = Uc(UC_ARCH_M68K, UC_MODE_BIG_ENDIAN)
    c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
    for base, size in [(0, 0x200000), (0xff0000, 0x10000), (0x400000, 0x1000)]:
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

def getl(c, address):
    return struct.unpack('>I', c.mem_read(address, 4))[0]

def execute(c, start, stop):
    c.emu_start(start, stop, count=1_000_000)
    assert c.reg_read(UC_M68K_REG_PC) == stop, f'{start:x} did not reach {stop:x}'

with tempfile.TemporaryDirectory() as temp:
    output = Path(temp)/'dialogue'
    subprocess.run([str(root/'target/release/shadowrealm'), 'dialogue-data', str(a.rom.resolve()), str(output)], check=True)
    data = json.loads((output/'dialogue.json').read_text())
    nodes = {n['node']: n for n in data['nodes']}
    for n in nodes.values():
        assert long(n['node']) == n['text_address']
        assert rom[n['text_address']:].split(b'\xff', 1)[0].replace(b'\x80', b'\n').decode('ascii') == n['text']
        for i, choice in enumerate(n['choices']):
            assert long(n['node']+4+i*4) == choice['text_address']
            assert long(n['node']+16+i*4) == choice['destination']
            assert rom[choice['text_address']:].split(b'\xff', 1)[0].replace(b'\x80', b'\n').decode('ascii') == choice['text']
    transitions = 0
    for edge in data['edges']:
        if edge['next'] is None:
            assert not nodes[edge['node']]['choices'][edge['slot']]['available']
            assert edge['runner_known'] == 0xa000 and edge['contacts'] == 0x10000000
            continue
        c = cpu()
        c.reg_write(UC_M68K_REG_A0, edge['node']+16)
        c.mem_write(0xffdd22, bytes([[0x40, 0x10, 0x20][edge['slot']], 0]))
        c.mem_write(0xfff0ae, b'\x02')
        # Optional-choice gates are initialized by $1f452/$1f4ae in the original UI.
        # All selected edges here refer to real non-null option slots.
        putw(c, 0xfffc2a, 0xa000)
        c.mem_write(0xfffc2c, struct.pack('>I', 0x10000000))
        execute(c, 0x1f522, 0x1f3c4)
        assert c.reg_read(UC_M68K_REG_A0) == edge['next'], f'branch mismatch: {edge}'
        assert getw(c, 0xfffc2a) == edge['runner_known'], f'runner flag mismatch: {edge}'
        assert getl(c, 0xfffc2c) == edge['contacts'], f'contact flag mismatch: {edge}'
        transitions += 1
    npc = (output/'npc.bin').read_bytes()
    for case in data['contacts']:
        c = cpu()
        c.mem_write(0xff0100, rom[0x7291a:0x72a1a])
        putw(c, 0xff0100, case['x'])
        putw(c, 0xff0104, case['y'])
        c.mem_write(0xff0110, bytes([case['player_flags']]))
        c.mem_write(0xff0400, npc)
        c.mem_write(0xff0410, bytes([case['npc_flags']]))
        c.mem_write(0xff049e, bytes([case['npc_talk']]))
        putw(c, 0xffddc4, 10)
        c.mem_write(0xfffbf9, b'\x06')
        c.reg_write(UC_M68K_REG_A0, 0xff0100)
        # Stop after the interaction handler, before collision-avoidance PRNG.
        c.hook_add(UC_HOOK_CODE, lambda uc, address, size, user: uc.emu_stop(), begin=0xec90, end=0xec90)
        c.emu_start(0xeb92, 0x400000, count=1_000_000)
        assert c.reg_read(UC_M68K_REG_PC) in [0xec90, 0x400000]
        scheduled = getl(c, 0xffdd2c) == 0x21174
        assert scheduled == case['contact'], f'contact mismatch: {case}'
        if scheduled:
            assert bytes(c.mem_read(0xfff0ae, 1)) == b'\x02'
            assert getw(c, 0xffe11c) == 60
            execute(c, 0x21174, 0x1f246)
            assert c.reg_read(UC_M68K_REG_A0) == data['root']
            assert getw(c, 0xfffc2a) == 4
            assert bytes(c.mem_read(0xfff0af, 1)) == b'\xff'

print(f'PASS: {len(nodes)} JSON node/text records, {transitions} original branch/action executions, and {len(data["contacts"])} contact/gating cases; unknown hiring action remains unavailable')
