"""Compare target lists, acquisition/retention, RNG and close range with M68000.

The empty-list end-combat callback is counted and omitted; no combat shutdown,
weapon actions, damage, hostile AI or mission behavior is claimed here. The
projectile-probe comparisons execute the full original query without call stubs.
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
assert hashlib.sha256(rom).hexdigest() == '92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root = Path(__file__).resolve().parents[1]
c = Uc(UC_ARCH_M68K, UC_MODE_BIG_ENDIAN)
c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for base, size in [(0, 0x200000), (0xff0000, 0x10000), (0x400000, 0x1000)]:
    c.mem_map(base, size)
c.mem_write(0, rom)
c.mem_write(0x1dbf0, b'\x4e\x75')
callbacks = [0]
c.hook_add(UC_HOOK_CODE, lambda uc, address, size, user: callbacks.__setitem__(0, callbacks[0] + 1), begin=0x1dbf0, end=0x1dbf0)

def putw(address, value):
    c.mem_write(address, struct.pack('>H', value))

def getw(address):
    return struct.unpack('>H', c.mem_read(address, 2))[0]

def call(start):
    c.reg_write(UC_M68K_REG_SR, 0x2700)
    c.reg_write(UC_M68K_REG_A7, 0xffff00)
    c.mem_write(0xffff00, struct.pack('>I', 0x400000))
    hook = c.hook_add(UC_HOOK_CODE, lambda uc, address, size, user: uc.emu_stop(), begin=0x400000, end=0x400000)
    try:
        c.emu_start(start, 0x400000, count=500_000)
    finally:
        c.hook_del(hook)
    assert c.reg_read(UC_M68K_REG_PC) == 0x400000, f'{start:x} did not return'

def compare(expected, label):
    got = bytes(c.mem_read(0xff0100, len(expected)))
    expected = bytes(expected)
    assert got == expected, f'{label}: {[(hex(i), expected[i], got[i]) for i in range(len(expected)) if expected[i] != got[i]][:20]}'

def lists(case, label):
    for count_address, table, key in [(0xffdf94, 0xffdf96, 'npcs'), (0xffdfc8, 0xffdfca, 'party')]:
        count = getw(count_address)
        pointers = list(struct.unpack(f'>{count}I', c.mem_read(table, count * 4))) if count else []
        assert pointers == case[key], f'{label} {key}: {pointers} != {case[key]}'

with tempfile.TemporaryDirectory() as temp:
    out = Path(temp) / 'combat'
    subprocess.run([str(root / 'target/release/shadowrealm'), 'combat-data', str(a.rom.resolve()), str(out)], check=True)
    data = json.loads((out / 'combat.json').read_text())
    for i, case in enumerate(data['weapon_timing']):
        c.mem_write(0xff0100, bytes(case['before']))
        c.reg_write(UC_M68K_REG_A0, 0xff0100)
        c.reg_write(UC_M68K_REG_D0, case['mode'])
        call(0xabfc)
        expected = bytearray(case['before'])
        expected[0xdc:0xde] = struct.pack('>H', case['timer'])
        compare(expected, f'cooldown {i}')
        c.mem_write(0xff0100, bytes(case['before']))
        c.reg_write(UC_M68K_REG_D0, case['angle'])
        call(0xb39a)
        compare(case['ranged'], f'ranged setup {i}')
        c.mem_write(0xff0100, bytes(case['before']))
        # Stop at dispatcher continuation, or inactive actor epilogue.
        def stop_tick(uc, address, size, user):
            if address in (0xa912, 0xaac6):
                uc.emu_stop()
        hook = c.hook_add(UC_HOOK_CODE, stop_tick)
        c.reg_write(UC_M68K_REG_A7, 0xffff00)
        c.reg_write(UC_M68K_REG_SR, 0x2700)
        try:
            c.emu_start(0xa8fa, 0x400000, count=30)
        finally:
            c.hook_del(hook)
        assert c.reg_read(UC_M68K_REG_PC) in (0xa912, 0xaac6)
        compare(case['tick'], f'cooldown tick {i}')
    print(f"Weapon timing: {len(data['weapon_timing'])} full actor comparisons each for cooldown, dispatcher tick and ranged setup; no stubs")
    replaced = retained = empty = 0
    for i, case in enumerate(data['selections']):
        c.mem_write(0xff0000, bytes(65536))
        c.mem_write(0xff0100, bytes(case['before']))
        putw(0xffdfe8, 0xffff)
        putw(0xfff9f8, case['seed'])
        call(0x13874)
        lists(case, f'list {i}')
        c.reg_write(UC_M68K_REG_A0, 0xff0100 + case['source'] * 256)
        callbacks[0] = 0
        call(0x14492)
        compare(case['after'], f'acquisition {i}')
        assert getw(0xfff9f8) == case['after_seed'], f'RNG {i}'
        assert bool(callbacks[0]) == case['empty_request'], f'empty-list callback {i}'
        replaced += case['seed'] != case['after_seed']
        empty += case['empty_request']
        offset = case['source'] * 256 + 0xa2
        old = bytes(case['before'][offset:offset + 4])
        retained += old != bytes(4) and old == bytes(case['after'][offset:offset + 4])
    assert replaced and retained and empty
    for i, case in enumerate(data['range']):
        c.mem_write(0xff0100, bytes(case['source']))
        c.mem_write(0xff0200, bytes(case['target']))
        c.reg_write(UC_M68K_REG_A0, 0xff0100)
        c.reg_write(UC_M68K_REG_A1, 0xff0200)
        call(0x15c58)
        assert bool(c.reg_read(UC_M68K_REG_SR) & 1) == case['in_reach'], f'close-range gate {i}'
        if case['heading'] is not None:
            assert c.reg_read(UC_M68K_REG_D2) & 65535 == case['heading'], f'close-range heading {i}'
            assert c.reg_read(UC_M68K_REG_D3) & 65535 == case['range'], f'close-range distance {i}'
    c.mem_write(0xff0000, bytes(65536))
    putw(0xfff9f8, 0x3344)
    for phase, case in enumerate(data['ticks']):
        c.mem_write(0xff0100, bytes(case['before']))
        putw(0xffdf88, phase)
        putw(0xffdfe8, 1 if phase == 0 else 0)
        call(0x13874)
        lists(case, f'refresh phase {phase}')
        c.reg_write(UC_M68K_REG_A0, 0xff0100)
        callbacks[0] = 0
        call(0x14492)
        compare(case['after'], f'refresh/acquire phase {phase}')
        assert getw(0xfff9f8) == case['seed'], f'refresh RNG {phase}'
        assert bool(callbacks[0]) == case['empty_request'], f'refresh empty callback {phase}'
    line_counts = [0,0,0,0]
    for i,case in enumerate(data['lines']):
        grid=data['line_grids'][case['grid']]
        c.mem_write(0xff0000,bytes(65536))
        source_slot=struct.unpack_from('>H',bytes(case['source']),0x3e)[0]
        source_address=0xff0100+source_slot*256
        other_address=0xff0100+((source_slot+1)%3)*256
        c.mem_write(source_address,bytes(case['source']))
        c.mem_write(0xff0400,bytes(case['target']))
        # Intervening actors are intentionally ignored by the original probe
        # mode: only the requested target pointer can terminate this query.
        other=bytearray(case['source'])
        sx,sy=struct.unpack_from('>H',other,0)[0],struct.unpack_from('>H',other,4)[0]
        tx,ty=struct.unpack_from('>H',bytes(case['target']),0)[0],struct.unpack_from('>H',bytes(case['target']),4)[0]
        struct.pack_into('>H',other,0,(sx+tx)//2)
        struct.pack_into('>H',other,4,(sy+ty)//2)
        struct.pack_into('>H',other,0x3e,(source_slot+1)%3)
        c.mem_write(other_address,bytes(other))
        c.mem_write(0xff1700,bytes(grid['cells']))
        rectangles=b''.join(struct.pack('>5H',*r) for r in grid['rectangles'])
        c.mem_write(0xff3000,rectangles)
        c.mem_write(0xffddec,struct.pack('>I',0xff3000))
        for at,value in zip([0xffddd6,0xffddd8,0xffddd2,0xffddd4],grid['bounds']):putw(at,value)
        c.mem_write(0xffef10,bytes(case['initial_samples']))
        callbacks[0]=0
        c.reg_write(UC_M68K_REG_A0,source_address)
        c.reg_write(UC_M68K_REG_A1,0xff0400)
        steps=[0]
        hit=[False]
        hs=c.hook_add(UC_HOOK_CODE,lambda uc,address,size,user:steps.__setitem__(0,steps[0]+1),begin=0x14f26,end=0x14f26)
        hh=c.hook_add(UC_HOOK_CODE,lambda uc,address,size,user:hit.__setitem__(0,True),begin=0x15ba0,end=0x15ba0)
        call(0x14d8e)
        c.hook_del(hs);c.hook_del(hh)
        outcome=1 if c.reg_read(UC_M68K_REG_SR)&1 else (0 if hit[0] else 2)
        assert outcome==case['outcome'],f'line outcome {i} grid {case["grid"]}: original {outcome}, native {case["outcome"]}'
        assert steps[0]==case['steps'],f'line steps {i}: {steps[0]} != {case["steps"]}'
        got=bytes(c.mem_read(0xff1380,256));expected=bytes(case['projectile'])
        assert got==expected,f'projectile record {i}: {[(hex(j),expected[j],got[j]) for j in range(256) if got[j]!=expected[j]][:16]}'
        assert bytes(c.mem_read(0xffef10,4))==bytes(case['samples']),f'line scratch {i}'
        assert bytes(c.mem_read(source_address,256))==bytes(case['source']),f'probe mutated source {i}'
        assert bytes(c.mem_read(0xff0400,256))==bytes(case['target']),f'probe mutated target {i}'
        assert getw(0xffdfd8)==0,f'probe flag was not cleared {i}'
        assert bytes(c.mem_read(other_address,256))==bytes(other),f'probe mutated intervening actor {i}'
        assert callbacks[0]==0,f'probe called omitted combat-end code {i}'
        line_counts[outcome]+=1
    assert all(line_counts[j]>0 for j in [0,1,2]),f'line outcomes not exercised: {line_counts}'
print(f'PASS: {len(data["selections"])} target-list/acquisition cases ({replaced} random replacements, {retained} retained targets, {empty} empty-list callbacks); {len(data["range"])} close-range probes; {len(data["ticks"])} refresh/retention/RNG phases; {len(data["lines"])} projectile probes (hit/blocked/limit={line_counts[:3]})')
