"""Compare physical contact/separation and ordered party motion with M68000 code.

The $eccc dialogue/quest callback is omitted from contact-only fixtures; its
implemented Trent path has a separate oracle. Travel fixtures stop at the
original location-dispatch boundary; unrecovered travel scripts remain guarded.
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

def cpu():
    c = Uc(UC_ARCH_M68K, UC_MODE_BIG_ENDIAN)
    c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
    for base, size in [(0, 0x200000), (0xff0000, 0x10000), (0x400000, 0x1000)]:
        c.mem_map(base, size)
    c.mem_write(0, rom)
    # Contact fixtures isolate physical effects; skip the independently tested
    # script callback, retaining the real heading and RNG routines.
    c.mem_write(0xeccc, b'\x4e\x75')
    return c

def putw(c, address, value):
    c.mem_write(address, struct.pack('>H', value))

def getw(c, address):
    return struct.unpack('>H', c.mem_read(address, 2))[0]

def call(c, start, stop=0x400000):
    c.reg_write(UC_M68K_REG_SR, 0x2700)
    c.reg_write(UC_M68K_REG_A7, 0xffff00)
    c.mem_write(0xffff00, struct.pack('>I', 0x400000))
    hook = c.hook_add(UC_HOOK_CODE, lambda uc, address, size, user: uc.emu_stop(), begin=stop, end=stop)
    try:
        c.emu_start(start, stop, count=500_000)
    finally:
        c.hook_del(hook)
    assert c.reg_read(UC_M68K_REG_PC) == stop, f'{start:x} stopped at {c.reg_read(UC_M68K_REG_PC):x}'

def compare(c, address, expected, label):
    got = bytes(c.mem_read(address, len(expected)))
    expected = bytes(expected)
    assert got == expected, f'{label}: {[(hex(i), expected[i], got[i]) for i in range(len(expected)) if expected[i] != got[i]][:20]}'

with tempfile.TemporaryDirectory() as temp:
    out = Path(temp) / 'avoidance'
    subprocess.run([str(root / 'target/release/shadowrealm'), 'avoidance-data', str(a.rom.resolve()), str(out)], check=True)
    data = json.loads((out / 'avoidance.json').read_text())
    c = cpu()
    found = adjusted = 0
    for i, case in enumerate(data['contacts']):
        c.mem_write(0xff0100, bytes(0xa00))
        for slot, record in zip(case['slots'], case['before']):
            c.mem_write(0xff0100 + slot * 256, bytes(record))
        putw(c, 0xfff9f8, case['seed'])
        c.reg_write(UC_M68K_REG_A0, 0xff0100 + case['slots'][0] * 256)
        call(c, 0xeb92)
        for slot, record in zip(case['slots'], case['after']):
            compare(c, 0xff0100 + slot * 256, record, f'contact {i}, slot {slot}')
        assert getw(c, 0xfff9f8) == case['after_seed'], f'contact RNG {i}'
        found += case['found']
        adjusted += case['found'] and case['after_seed'] & 0x7000 == 0
    assert found > 0 and adjusted > 0
    cpus = {}
    for map_id in [0, 3, 5, 10]:
        c = cpu()
        descriptor = struct.unpack_from('>I', rom, 0x3232 + map_id * 4)[0]
        c.mem_write(0xffddc6, rom[descriptor:descriptor + 96])
        putw(c, 0xffddc4, map_id)
        putw(c, 0xffdf8a, 0xffff)
        call(c, 0xee10)
        cpus[map_id] = c
    probes = {address: 0 for address in [0xb522, 0xb4e0, 0xb504]}
    third_clear = [0]
    def count_probe(uc, address, size, user):
        probes[address] += 1
    def count_third_clear(uc, address, size, user):
        third_clear[0] += uc.reg_read(UC_M68K_REG_D0) & 0xffff == 0
    for i, case in enumerate(data['movements']):
        c = cpus[case['map']]
        c.mem_write(0xff0200, bytes(case['before']))
        c.reg_write(UC_M68K_REG_A0, 0xff0200)
        putw(c, 0xffdf88, case['phase'])
        hooks = [c.hook_add(UC_HOOK_CODE, count_probe, begin=address, end=address) for address in probes]
        hooks.append(c.hook_add(UC_HOOK_CODE, count_third_clear, begin=0xb518, end=0xb518))
        call(c, 0xaa88, 0xaac6)
        for hook in hooks:
            c.hook_del(hook)
        compare(c, 0xff0200, case['after'], f'separation {i}, map {case["map"]}')
    assert all(probes.values()), f'first/second/third probe branches missing: {probes}'
    assert third_clear[0] > 0, 'third clear candidate discard was not exercised'
    timeline_ticks = 0
    for trace in data['timelines']:
        c = cpu()
        c.mem_write(0xff0100, bytes(trace['initial']))
        c.mem_write(0xff1700, bytes(data['cells']))
        c.mem_write(0xfffc04, struct.pack('>I', 0xff0100 + trace['selected'] * 256))
        for address, value in [(0xffddd2, 512), (0xffddd4, 512), (0xffddea, 32), (0xfff9f8, trace['seed'])]:
            putw(c, address, value)
        for phase, tick in enumerate(trace['ticks']):
            c.mem_write(0xffe175, b'\x00')
            putw(c, 0xffdf88, phase)
            for slot in range(3):
                c.reg_write(UC_M68K_REG_A0, 0xff0100 + slot * 256)
                if slot != trace['selected']:
                    call(c, 0x138b8)
                call(c, 0xaa82, 0xaac6)
            compare(c, 0xff0100, tick['actors'], f'ordered selected={trace["selected"]}, tick={phase}')
            assert getw(c, 0xfff9f8) == tick['seed'], f'ordered RNG {phase}'
        timeline_ticks += len(trace['ticks'])
    requests = discarded_requests = 0
    third = [False]
    c = cpu()
    c.mem_write(0x423c, b'\x4e\x75')  # omit location-label/UI update only
    stopped = [False]
    def dispatch_stop(uc, address, size, user):
        stopped[0] = True
        uc.emu_stop()
    hook = c.hook_add(UC_HOOK_CODE, dispatch_stop, begin=0x1651e, end=0x1651e)
    end_hook = c.hook_add(UC_HOOK_CODE, lambda uc,address,size,user: uc.emu_stop(), begin=0xaac6,end=0xaac6)
    third_hook = c.hook_add(UC_HOOK_CODE, lambda uc,address,size,user: third.__setitem__(0, True), begin=0xb504,end=0xb504)
    for i, case in enumerate(data['travel']):
        c.mem_write(0xff0000, bytes(65536))
        c.mem_write(0xff0100, bytes(case['before']))
        c.mem_write(0xff1700, bytes(data['travel_grids'][case['grid']]))
        for address,value in [(0xffddd2,512),(0xffddd4,512),(0xffddea,32),(0xffddc4,case['map']),
                              (0xffddc6,0x1880),(0xffddc8,0x200),(0xffdf88,case['phase'])]:
            putw(c,address,value)
        c.reg_write(UC_M68K_REG_A0,0xff0100)
        stopped[0] = False
        third[0] = False
        # Stop before the real location-dispatch call, or after an ineligible
        # response returns. No travel/script routine is replaced with a guess.
        c.reg_write(UC_M68K_REG_SR,0x2700)
        c.reg_write(UC_M68K_REG_A7,0xffff00)
        c.mem_write(0xffff00,struct.pack('>I',0x400000))
        c.emu_start(0xaa88,0xaac6,count=500_000)
        expected_request = case['request']
        assert stopped[0] == (expected_request is not None), f'travel request {i}, PC={c.reg_read(UC_M68K_REG_PC):x}'
        assert c.reg_read(UC_M68K_REG_PC) in [0xaac6,0x1651e], f'travel timeout {i}'
        compare(c,0xff0100,case['after'],f'travel actor {i}')
        if expected_request is not None:
            requests += 1
            discarded_requests += third[0]
            assert bytes(c.mem_read(0xfffbf9,1))[0] == expected_request['code'] & 31
            assert getw(c,0xff01cc) == expected_request['orientation'] << 14
            if case['return_positions'] is not None:
                expected = struct.pack('>6H',*[v for xy in case['return_positions'] for v in xy])
                compare(c,0xffdf44,expected,f'travel return formation {i}')
    c.hook_del(hook)
    c.hook_del(end_hook)
    c.hook_del(third_hook)
    assert discarded_requests > 0, "no travel request from a discarded third probe was exercised"
    assert requests > 0
print(f'PASS: {len(data["contacts"])} contact cases ({found} contacts, {adjusted} RNG adjustments); {len(data["movements"])} response steps; {timeline_ticks} ordered party/contact/follower ticks with all three leader slots; {len(data["travel"])} contact/travel gates ({requests} dispatch requests, {discarded_requests} after discarded third probes)')
