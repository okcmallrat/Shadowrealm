"""Compare normal recovery, runner archives and opinion changes with original M68000.

Compact save/restore, party snapshots, penalty arithmetic, medical core and arrival
execute without stubs. The complete companion-removal prefix omits only HUD/UI
refresh ($3404/$38fa); the controlled pointer remains Joshua for those fixtures.
Mission callbacks and Windows GUI behavior are outside this oracle.
"""
import argparse
import hashlib
import json
import struct
import subprocess
import tempfile
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

def w(address, value):
    c.mem_write(address, struct.pack('>H', value))

def l(address, value):
    c.mem_write(address, struct.pack('>I', value))

def rw(address):
    return struct.unpack('>H', c.mem_read(address, 2))[0]

def rl(address):
    return struct.unpack('>I', c.mem_read(address, 4))[0]

def run(start, stop=0x400000):
    c.reg_write(UC_M68K_REG_SR, 0x2700)
    c.reg_write(UC_M68K_REG_A7, 0xffff00)
    l(0xffff00, 0x400000)
    hook = c.hook_add(UC_HOOK_CODE, lambda uc, addr, size, user: uc.emu_stop(), begin=stop, end=stop)
    try:
        c.emu_start(start, 0x400000, count=100_000)
    finally:
        c.hook_del(hook)
    assert c.reg_read(UC_M68K_REG_PC) == stop, hex(c.reg_read(UC_M68K_REG_PC))

def equal(address, expected, label):
    actual = bytes(c.mem_read(address, len(expected)))
    expected = bytes(expected)
    assert actual == expected, (label, [(hex(i), expected[i], actual[i]) for i in range(len(expected)) if actual[i] != expected[i]][:20])

def ui_return(uc, address, size, user):
    sp = uc.reg_read(UC_M68K_REG_A7)
    ret = int.from_bytes(uc.mem_read(sp, 4), 'big')
    uc.reg_write(UC_M68K_REG_A7, sp + 4)
    uc.reg_write(UC_M68K_REG_PC, ret)

with tempfile.TemporaryDirectory() as tmp:
    out = Path(tmp) / 'out'
    subprocess.run([str(root / 'target/release/shadowrealm'), 'clinic-data', str(a.rom.resolve()), str(out)], check=True)
    data = json.loads((out / 'clinic.json').read_text())
    for i, case in enumerate(data['cases']):
        c.mem_write(0xff0100, bytes(case['before']))
        l(0xfffc1e, case['flags'])
        l(0xfffbfe, case['balance'])
        run(0xc726, 0xc7c0)
        equal(0xff0100, case['after'], ('medical', i))
        assert rl(0xfffbfe) == case['after_balance']
        assert case['balance'] - rl(0xfffbfe) == case['fee']
        assert c.reg_read(UC_M68K_REG_A0) == case['node']
    assert struct.unpack_from('>II', rom, data['node']) == (data['text'], 0xffffffff)
    assert struct.unpack_from('>5I', rom, data['insured_node']) == (data['insured_text'], data['more_text'], 0, 0, data['continuation'])
    assert struct.unpack_from('>II', rom, data['continuation']) == (data['continuation_text'], 0xffffffff)
    for start, plan in [(0x2a44, data['plan']), (0x2a32, data['insured_plan'])]:
        run(start, 0x2a52)
        assert c.reg_read(UC_M68K_REG_D0) & 65535 == plan['map'] and rl(0xffe114) == plan['spawn']
        run(0x2b78, 0x2bd0)
        assert [rw(0xffddc6), rw(0xffddc8)] == plan['camera']
        assert [[rw(0xff0100 + i * 256), rw(0xff0104 + i * 256)] for i in range(3)] == plan['positions']
        assert rl(0xffe114) == 0
    archives = json.loads((out / 'archives.json').read_text())
    for i, case in enumerate(archives['compact']):
        c.mem_write(0xff1000, bytes(case['before']))
        c.reg_write(UC_M68K_REG_A0, 0xff1000)
        c.reg_write(UC_M68K_REG_A1, 0xff2000)
        run(0x597ee)
        equal(0xff2000, case['packed'], ('pack', i))
        assert c.reg_read(UC_M68K_REG_A1) == 0xff2000 + 74
        c.mem_write(0xff1000, bytes(case['restore_before']))
        c.mem_write(0xff2000, bytes(case['record']))
        c.reg_write(UC_M68K_REG_A0, 0xff2000)
        c.reg_write(UC_M68K_REG_A1, 0xff1000)
        run(0x598b0)
        equal(0xff1000, case['restored'], ('restore', i))
        assert c.reg_read(UC_M68K_REG_A0) == 0xff2000 + 74
    for i, case in enumerate(archives['snapshots']):
        c.mem_write(0xff0100, bytes(case['party']) + bytes(768 - len(case['party'])))
        c.mem_write(0xfff1fc, bytes(case['before']))
        c.mem_write(0xfffc0c, bytes(case['indices']))
        run(0x59798)
        equal(0xfff1fc, case['after'], ('snapshot', i))
    for case in archives['penalties']:
        c.mem_write(0xfffc14, bytes([case['before']]))
        c.reg_write(UC_M68K_REG_D7, 3)
        c.reg_write(UC_M68K_REG_D6, 0xfffe)
        run(0xc8b0)
        equal(0xfffc14, [case['after']], 'penalty')
    hooks = [c.hook_add(UC_HOOK_CODE, ui_return, begin=address, end=address) for address in [0x3404, 0x38fa]]
    try:
        for i, case in enumerate(archives['defeats']):
            c.mem_write(0xff0100, bytes(case['party']) + bytes(768 - len(case['party'])))
            c.mem_write(0xfff1fc, bytes(case['before']))
            c.mem_write(0xfffc0c, bytes(case['indices']))
            c.mem_write(0xfffc12, bytes(case['opinions']))
            l(0xfffc04, 0xff0100)
            run(0xc716, 0xc726)
            equal(0xfff1fc, case['after'], ('defeat archives', i))
            equal(0xfffc12, case['after_opinions'], ('defeat opinion', i))
            equal(0xfffc0c, [255, 255], 'removed indices')
            for slot in range(1, 3):
                assert c.mem_read(0xff0100 + slot * 256 + 0x10, 1)[0] & 1 == 0
    finally:
        for hook in hooks:
            c.hook_del(hook)
    for i, case in enumerate(archives['rehires']):
        c.mem_write(0xff0100, bytes(case['leader']) + bytes(512))
        c.mem_write(0xff0400, bytes(case['npc']))
        c.mem_write(0xfff1fc, bytes(case['archive']))
        c.mem_write(0xfffc0c, b'\xff\xff')
        c.mem_write(0xfffc12, bytes(case['opinions']))
        c.mem_write(0xfff0ae, b'\x02')
        l(0xffe128, 0xff0400)
        l(0xfffbfe, 10000)
        # Original quote arithmetic; omit UI only while obtaining the price.
        quote_hooks = [c.hook_add(UC_HOOK_CODE, ui_return, begin=address, end=address) for address in [0xc0da, 0xc216, 0x1f5a8]]
        try:
            run(0x21226)
        finally:
            for hook in quote_hooks:
                c.hook_del(hook)
        assert rl(0xffef16 if case['contract'] == 0 else 0xffef36) == case['price']
        run(0x21326 if case['contract'] == 0 else 0x21332, 0x2138e)
        run(0x21394, 0x213a8)
        # $ca=30 is the existing native animation rate, not compact persistence.
        w(0xff02ca, 30)
        equal(0xff0200, case['after'], ('rehire', i))
        equal(0xff0100, case['leader'], ('rehire leader', i))
        equal(0xfffc0c, [3, 255], 'rehire indices')
        assert rl(0xfffbfe) == case['balance']
    for case in archives['opinions']:
        c.mem_write(0xfff0ae, b'\x02')
        c.mem_write(0xfffc14, bytes([case['opinion'] & 255]))
        run(0x21174, 0x211cc)
        assert c.reg_read(UC_M68K_REG_A0) == case['root']
        run(0x211e8)
        assert c.reg_read(UC_M68K_REG_A0) == case['topic']
    print(f"PASS: {len(data['cases'])} medical cores, both passage/arrival routes; {len(archives['compact'])} compact save + restore pairs; {len(archives['snapshots'])} party snapshots; all 256 penalty bytes; {len(archives['defeats'])} companion-removal prefixes; {len(archives['rehires'])} full rehire purchases; negative/zero-opinion dialogue selectors")
