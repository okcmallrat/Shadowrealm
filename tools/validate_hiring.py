"""Differential checks for Trent first-hire prices, payment and recruitment.

Requires the supplied ROM and Unicorn. UI/VDP-only routines are skipped; original
fee arithmetic, compact snapshot reload, party allocation and mutation execute.
This is not a full game, save history, job completion or companion-AI oracle.
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
    p.error('requires supplied ROM revision')
root = Path(__file__).resolve().parents[1]

def cpu():
    c = Uc(UC_ARCH_M68K, UC_MODE_BIG_ENDIAN)
    c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
    for address, size in [(0, 0x200000), (0xff0000, 0x10000), (0x400000, 0x1000)]:
        c.mem_map(address, size)
    c.mem_write(0, rom)
    c.reg_write(UC_M68K_REG_SR, 0x2700)
    def ui_return(uc, address, size, user):
        sp = uc.reg_read(UC_M68K_REG_A7)
        ret = int.from_bytes(uc.mem_read(sp, 4), 'big')
        uc.reg_write(UC_M68K_REG_A7, sp+4)
        uc.reg_write(UC_M68K_REG_PC, ret)
    # $c0da and $c216 preserve all working registers; $1f5a8 clears UI cells.
    for address in [0xc0da, 0xc216, 0x1f5a8]:
        c.hook_add(UC_HOOK_CODE, ui_return, begin=address, end=address)
    c.hook_add(UC_HOOK_CODE, lambda uc, address, size, user: uc.emu_stop(), begin=0x400000, end=0x400000)
    return c

def getl(c, p):
    return int.from_bytes(c.mem_read(p, 4), 'big')

def putl(c, p, v):
    c.mem_write(p, struct.pack('>I', v))

def call(c, address, stop=0x400000):
    c.reg_write(UC_M68K_REG_A7, 0xffff00)
    putl(c, 0xffff00, 0x400000)
    execute(c, address, stop)

def execute(c, address, stop):
    c.emu_start(address, stop, count=1_000_000)
    assert c.reg_read(UC_M68K_REG_PC) == stop, f'{address:x} did not reach {stop:x}'

with tempfile.TemporaryDirectory() as temp:
    out = Path(temp)/'hiring'
    subprocess.run([str(root/'target/release/shadowrealm'), 'hiring-data', str(a.rom.resolve()), str(out)], check=True)
    data = json.loads((out/'hiring.json').read_text())
    c = cpu()
    call(c, 0x7fc, 0x806)
    assert getl(c, 0xfffbfe) == data['initial_nuyen'] == 20
    for quote in data['quotes']:
        c = cpu()
        c.mem_write(0xff0100, rom[0x7291a:0x72a1a])
        for offset, value in zip([0x7a, 0x8b, 0x8c], quote['stats']):
            c.mem_write(0xff0100+offset, bytes([value]))
        c.mem_write(0xfff0ae, b'\x02')
        c.mem_write(0xfffc14, bytes([quote['opinion'] & 255]))
        call(c, 0x21226)
        assert [getl(c, p) for p in [0xffef16, 0xffef36]] == [quote['short_term'], quote['permanent']], quote
        assert c.reg_read(UC_M68K_REG_A0) == 0xff958
    for case in data['cases']:
        case_dir = out/case['directory']
        before = (case_dir/'before-party.bin').read_bytes()
        npc = (case_dir/'before-npc.bin').read_bytes()
        c = cpu()
        c.mem_write(0xff0100, before)
        c.mem_write(0xff0400, npc)
        c.mem_write(0xfffc0c, bytes(case['initial_indices']))
        c.mem_write(0xfff0ae, b'\x02')
        putl(c, 0xfffbfe, case['funds'])
        putl(c, 0xffe128, 0xff0400)
        # Initialize the first-hire archive for Trent, using original compact
        # serialization from the unmodified full template (no rehire history).
        c.mem_write(0xff1000, rom[0x72e1a:0x72f1a])
        c.reg_write(UC_M68K_REG_A0, 0xff1000)
        c.reg_write(UC_M68K_REG_A1, 0xfff1fc+3*74)
        call(c, 0x597ee)
        # Snapshot the existing live party; $42e6/$5966e rehydrate these fields.
        call(c, 0x59798)
        call(c, 0x21226)
        assert c.reg_read(UC_M68K_REG_A0) == case['quote_node'], case
        if case['party_size'] < 3:
            assert getl(c, 0xffef16 if case['contract'] == 0 else 0xffef36) == case['price']
            start = 0x21326 if case['contract'] == 0 else 0x21332
            # Stop before the HUD refresh. Insufficient-funds returns directly.
            c.reg_write(UC_M68K_REG_A7, 0xffff00)
            putl(c, 0xffff00, 0x400000)
            c.emu_start(start, 0x2138e, count=1_000_000)
            pc = c.reg_read(UC_M68K_REG_PC)
            # The return hook also stops the insufficient-funds guard outcome.
            if pc == 0x2138e:
                execute(c, 0x21394, 0x213a8)  # remove NPC and copy coordinates
                execute(c, 0x213ae, 0x400000)  # final response selector, excluding VDP
            else:
                assert pc == 0x400000, f'unexpected purchase PC {pc:x}'
        assert c.reg_read(UC_M68K_REG_A0) == case['response'], case
        assert getl(c, 0xfffbfe) == case['balance'], case
        assert list(c.mem_read(0xfffc0c, 2)) == case['indices_after'], case
        assert bool(c.mem_read(0xff0410, 1)[0] & 1) == case['npc_present'], case
        expected = bytearray(c.mem_read(0xff0100, case['party_after']*256))
        if case['party_after'] > case['party_size']:
            # Native exploration's existing animation-rate choice; AI scheduling
            # that computes the original rate is not claimed by this milestone.
            struct.pack_into('>H', expected, case['party_size']*256+0xca, 30)
        native = (case_dir/'after-party.bin').read_bytes()
        assert native == expected, f'party bytes differ in {case}: {[hex(i) for i in range(len(native)) if native[i] != expected[i]][:16]}'

print(f'PASS: {len(data["quotes"])} original fee quotes and {len(data["cases"])} native full-party/funds-boundary/contract purchases; money, recruit indices, NPC removal and party records match')
