"""Compare complete original companion decisions with equipped attack spells.

Only audio and HUD/VDP transfers are omitted. Exhausted-equipment cases stop at
the original melee entry; they are compared as prefixes, not completed actions.
"""
import argparse, hashlib, json, struct, subprocess, tempfile
from pathlib import Path
from unicorn import Uc, UC_ARCH_M68K, UC_MODE_BIG_ENDIAN, UC_HOOK_CODE
from unicorn.m68k_const import *

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('rom', type=Path)
a = p.parse_args()
rom = a.rom.read_bytes()
assert hashlib.sha256(rom).hexdigest() == '92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root = Path(__file__).resolve().parents[1]
c = Uc(UC_ARCH_M68K, UC_MODE_BIG_ENDIAN)
c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for base, size in [(0, 0x200000), (0xff0000, 0x10000), (0x400000, 0x1000)]:
    c.mem_map(base, size)
c.mem_write(0, rom)
for address in (0x530de, 0x1232, 0x3404, 0xf5c, 0x3648):
    c.mem_write(address, b'\x4e\x75')
    c.ctl_remove_cache(address, address + 2)

def w(at, value): c.mem_write(at, struct.pack('>H', value))
def l(at, value): c.mem_write(at, struct.pack('>I', value))
def rw(at): return struct.unpack('>H', c.mem_read(at, 2))[0]
def call(slot, stop):
    for register in [UC_M68K_REG_D0, UC_M68K_REG_D1, UC_M68K_REG_D2, UC_M68K_REG_D3,
                     UC_M68K_REG_D4, UC_M68K_REG_D5, UC_M68K_REG_D6, UC_M68K_REG_D7,
                     UC_M68K_REG_A0, UC_M68K_REG_A1, UC_M68K_REG_A2, UC_M68K_REG_A3,
                     UC_M68K_REG_A4, UC_M68K_REG_A5, UC_M68K_REG_A6]:
        c.reg_write(register, 0)
    c.reg_write(UC_M68K_REG_A0, 0xff0100 + slot * 256)
    c.reg_write(UC_M68K_REG_SR, 0x2700)
    c.reg_write(UC_M68K_REG_A7, 0xffff00)
    l(0xffff00, 0x400000)
    ends = (0x141a0,) if stop else (0x400000,)
    hook = c.hook_add(UC_HOOK_CODE, lambda uc, at, size, user: uc.emu_stop() if at in ends else None)
    try: c.emu_start(0x138b8, 0x400000, count=500000)
    finally: c.hook_del(hook)
    assert c.reg_read(UC_M68K_REG_PC) in ends, hex(c.reg_read(UC_M68K_REG_PC))

with tempfile.TemporaryDirectory() as temp:
    out = Path(temp) / 'caster-movement'
    subprocess.run([str(root / 'target/release/shadowrealm'), 'caster-movement-data',
                    str(a.rom.resolve()), str(out)], check=True)
    data = json.loads((out / 'caster-movement.json').read_text())
    complete = prefixes = casts = 0
    for i, t in enumerate(data['decisions']):
        c.mem_write(0xff0000, bytes(65536))
        c.mem_write(0xff0100, bytes(t['before']))
        c.mem_write(0xfff9fa, bytes(data['books']))
        for at, value in zip((0xffddd6, 0xffddd8, 0xffddd2, 0xffddd4), (0, 0, 2048, 2048)): w(at, value)
        l(0xffddec, 0xff3000)
        w(0xffddea, 128)
        l(0xfffc04, 0xff0100)
        c.mem_write(0xffe175, bytes([255 if t['serviced'] else 0]))
        w(0xffdf84, t['engaged'])
        w(0xffdf86, 17)
        w(0xffdfde, t['firing'])
        w(0xfff9f8, t['seed'])
        w(0xffdf94, len(t['npcs']))
        if t['npcs']: c.mem_write(0xffdf96, b''.join(struct.pack('>I', v) for v in t['npcs']))
        c.mem_write(0xffef10, bytes(t['samples']))
        c.mem_write(0xffd86c, bytes(t['occupied']))
        c.mem_write(0xff1700, bytes(127 if j // 64 in (0, 63) or j % 64 in (0, 63) or (t['wall'] and j % 64 == 32) else 0 for j in range(4096)))
        if t['deferred']:
            assert t['deferred'] == 'Companion melee action is not available yet', (i, t['deferred'])
            prefixes += 1
        else: complete += 1
        call(t['slot'], t['deferred'] is not None)
        got = bytes(c.mem_read(0xff0100, 2560))
        expected = bytes(t['after'])
        assert got == expected, f'companion magic {i}: {[(hex(j), expected[j], got[j]) for j in range(2560) if got[j] != expected[j]][:20]}'
        assert rw(0xfff9f8) == t['after_seed'], f'RNG {i}'
        assert rw(0xffdf84) == t['after_engaged'] and rw(0xffdf86) == t['after_timeout'], f'alert {i}'
        assert rw(0xffdfde) == t['after_firing'], f'firing {i}'
        assert bytes(c.mem_read(0xffef10, 4)) == bytes(t['after_samples']), f'collision scratch {i}'
        assert list(map(bool, c.mem_read(0xffd86c, 18))) == t['after_occupied'], f'graphics {i}'
        assert bool(c.mem_read(0xffe175, 1)[0]) == t['after_serviced'], f'scheduler {i}'
        assert bytes(c.mem_read(0xff0b00, 1536)) == bytes(t['after_pool']), f'firearm fallback pool {i}'
        assert bool(rw(0xffdfe6)) == t['busy'] and bool(rw(0xffdd3a)) == t['burst'], f'firearm signals {i}'
        casts += t['casts']
    print(f'PASS: {complete} complete equipped-spell companion decisions ({casts} successful casts), '
          f'{prefixes} exhausted-equipment prefixes; only audio/HUD/VDP transfers omitted', flush=True)
