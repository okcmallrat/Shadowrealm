"""Compare targeting, obstruction, timing, dice and bounded original launches.

The empty-list end-combat callback is counted and omitted; no combat shutdown,
complete weapon actions, damage, hostile AI or mission behavior is claimed here. The
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
for base, size in [(0, 0x200000), (0xff0000, 0x10000), (0x400000, 0x1000), (0xfffff000, 0x1000)]:
    c.mem_map(base, size)
c.mem_write(0, rom)
c.mem_write(0x1dbf0, b'\x4e\x75')
callbacks = [0]
c.hook_add(UC_HOOK_CODE, lambda uc, address, size, user: callbacks.__setitem__(0, callbacks[0] + 1), begin=0x1dbf0, end=0x1dbf0)

def putw(address, value):
    c.mem_write(address, struct.pack('>H', value))

def getw(address):
    return struct.unpack('>H', c.mem_read(address, 2))[0]

def call(start, count=500_000):
    c.reg_write(UC_M68K_REG_SR, 0x2700)
    c.reg_write(UC_M68K_REG_A7, 0xffff00)
    c.mem_write(0xffff00, struct.pack('>I', 0x400000))
    hook = c.hook_add(UC_HOOK_CODE, lambda uc, address, size, user: uc.emu_stop(), begin=0x400000, end=0x400000)
    try:
        c.emu_start(start, 0x400000, count=count)
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
    for i,case in enumerate(data['controls']):
        c.mem_write(0xff0100,bytes(case['actor']));putw(0xffdd22,case['input']);putw(0xffdfe6,0xffff if case['pending'] else 0)
        c.reg_write(UC_M68K_REG_A0,0xff0100);c.reg_write(UC_M68K_REG_SR,0x2700)
        def gate_stop(uc,address,size,user):
            if address in (0xad82,0xae26,0xad5c):uc.emu_stop()
        hook=c.hook_add(UC_HOOK_CODE,gate_stop)
        try:c.emu_start(0xad1a,0x400000,count=100)
        finally:c.hook_del(hook)
        pc=c.reg_read(UC_M68K_REG_PC)
        assert pc in (0xad82,0xae26,0xad5c)
        assert (pc==0xad82)==case['request']
        assert bool(getw(0xffdfe6))==case['after_pending']
        compare(case['actor'],f'fire request {i}')
    c.mem_write(0x530de,b'\x4e\x75');c.mem_write(0x1232,b'\x4e\x75')
    for i,case in enumerate(data['reloads']):
        c.mem_write(0xff0100,bytes(case['before']));c.mem_write(0xfffc04,struct.pack('>I',0xff0100));c.reg_write(UC_M68K_REG_D1,0)
        call(0xaad2)
        compare(case['after'],f'reload {i}')
        assert bool(c.reg_read(UC_M68K_REG_SR)&1)==case['reloaded']
    for i,case in enumerate(data['failures']):
        c.mem_write(0xff0100,bytes(case['before']));c.mem_write(0xfffc04,struct.pack('>I',0xff0100));putw(0xffdd04,case['direction']);c.reg_write(UC_M68K_REG_D1,0)
        call(0xadf8)
        compare(case['after'],f'controlled failure/reload {i}')
    c.mem_write(0x530de,rom[0x530de:0x530e0]);c.mem_write(0x1232,rom[0x1232:0x1234])
    for i,case in enumerate(data['animations']):
        c.mem_write(0xff0100,bytes(case['before']));c.reg_write(UC_M68K_REG_A0,0xff0100)
        call(0x5fdc)
        hook=c.hook_add(UC_HOOK_CODE,lambda uc,address,size,user:uc.emu_stop(),begin=0x60ae,end=0x60ae)
        try:c.emu_start(0x600e,0x400000,count=200)
        finally:c.hook_del(hook)
        assert c.reg_read(UC_M68K_REG_PC)==0x60ae
        compare(case['after'],f'combat animation {i}')
    print(f'Fire requests/reloads/animation clocks: {len(data["controls"])}/{len(data["reloads"])}/{len(data["animations"])}; only reload audio/HUD omitted')
    print(f'Controlled failed-launch/reload continuations: {len(data["failures"])}; only audio/HUD omitted')
    c.mem_write(0x530de,b'\x4e\x75')
    reaction_deaths=0
    for i,case in enumerate(data['reactions']):
        c.mem_write(0xff0000,bytes(65536));c.mem_write(0xff0100,bytes(case['before']))
        putw(0xffe166,17)
        sounds=[]
        hook=c.hook_add(UC_HOOK_CODE,lambda uc,address,size,user:sounds.append(uc.reg_read(UC_M68K_REG_D0)&0xffff),begin=0x530de,end=0x530de)
        c.reg_write(UC_M68K_REG_A0,0xff0100+case['defender']*256)
        try:call(0xaff2)
        finally:c.hook_del(hook)
        compare(case['after'],f'reaction {i}')
        assert getw(0xffe166)==case['timer']
        assert sounds==([] if case['sound'] is None else [case['sound']])
        reaction_deaths+=case['sound'] is not None
    c.mem_write(0x530de,rom[0x530de:0x530e0])
    print(f'Original reaction setup: {len(data["reactions"])} ({reaction_deaths} death sounds); only audio omitted')
    # Original baseline damage, including party helper data changes and defeat
    # bookkeeping. Only the low-level HUD tile transfer is omitted.
    c.mem_write(0x1232,b'\x4e\x75')
    damaged = defeated = 0
    for i,case in enumerate(data['damages']):
        c.mem_write(0xff0000,bytes(65536))
        c.mem_write(0xff0100,bytes(case['before']))
        c.mem_write(0xff0b00,bytes(case['projectile']))
        w=case['world']
        c.mem_write(0xffe186,bytes([w['mode']]))
        c.mem_write(0xfff0ce,bytes([w['reward']]))
        c.mem_write(0xfffbed,bytes([w['special']]))
        putw(0xfffbf0,w['counter42'])
        c.mem_write(0xfffbfa,bytes([w['counter13']]))
        c.mem_write(0xfff0cc,b'\xaa');putw(0xfff112,0x1234)
        c.mem_write(0xfff0dd,b'\xac');putw(0xfff9f8,case['seed'])
        c.reg_write(UC_M68K_REG_A0,0xff0b00)
        c.reg_write(UC_M68K_REG_A1,0xff0100+case['defender']*256)
        observed={'rolls':0,'pool':None,'difficulty':None}
        def damage_hook(uc,address,size,user):
            if address==0xd3a:observed['rolls']+=1
            elif address==0xdba:
                observed['pool']=uc.reg_read(UC_M68K_REG_D6)&0xffff
                observed['difficulty']=uc.reg_read(UC_M68K_REG_D4)&0xffff
        hook=c.hook_add(UC_HOOK_CODE,damage_hook)
        try:call(0x55948)
        finally:c.hook_del(hook)
        compare(case['after'],f'damage actors {i}')
        assert bytes(c.mem_read(0xff0b00,256))==bytes(case['projectile'])
        assert observed=={key:case[key] for key in ('rolls','pool','difficulty')},f'damage roll inputs {i}: {observed}'
        assert getw(0xfff9f8)==case['after_seed'],f'damage RNG {i}'
        after=case['after_world']
        assert bytes(c.mem_read(0xfff0dd,1))==bytes([after['successes']])
        assert bytes(c.mem_read(0xfff0cc,1))==bytes([after['kind']])
        assert getw(0xfff112)==after['penetration']
        assert bytes(c.mem_read(0xfff0ce,1))==bytes([after['reward']]),f'damage reward {i}'
        assert getw(0xfffbf0)==after['counter42']
        assert bytes(c.mem_read(0xfffbfa,1))==bytes([after['counter13']])
        damaged+=case['before'][case['defender']*256+0x4c]!=case['after'][case['defender']*256+0x4c]
        defeated+=case['defeat']
    c.mem_write(0x1232,rom[0x1232:0x1234])
    assert damaged and defeated
    print(f'Baseline physical damage: {len(data["damages"])} ({damaged} health changes, {defeated} NPC defeat paths); only HUD transfer omitted')
    # Execute original dice and bounded $55640 with no instruction stubs.
    for i, case in enumerate(data['dice']):
        putw(0xfff9f8, case['seed'])
        c.reg_write(UC_M68K_REG_D6, case['pool'])
        c.reg_write(UC_M68K_REG_D4, case['difficulty'])
        rolls = [0]
        hook = c.hook_add(UC_HOOK_CODE, lambda uc, address, size, user: rolls.__setitem__(0,rolls[0]+1), begin=0xd3a, end=0xd3a)
        try:
            call(0xdba, count=4_000_000)
        finally:
            c.hook_del(hook)
        assert bytes(c.mem_read(0xfff0dd, 1)) == bytes([case['successes']]), f'dice successes {i}'
        assert getw(0xfff9f8) == case['after_seed'], f'dice RNG {i}'
        assert rolls[0] == case['rolls'], f'dice rolls {i}'
    for i, case in enumerate(data['attack_rolls']):
        c.mem_write(0xff0100, bytes(case['source']))
        c.mem_write(0xff0b00, bytes(256))
        putw(0xfff9f8, case['seed'])
        c.reg_write(UC_M68K_REG_A0, 0xff0100)
        c.reg_write(UC_M68K_REG_A1, 0xff0b00)
        c.reg_write(UC_M68K_REG_D0, 0x1234)
        observed = {'rolls': 0, 'pool': None, 'difficulty': None}
        def roll_hook(uc, address, size, user):
            if address == 0xd3a:
                observed['rolls'] += 1
            elif address == 0xdba:
                observed['pool'] = uc.reg_read(UC_M68K_REG_D6) & 0xffff
                observed['difficulty'] = uc.reg_read(UC_M68K_REG_D4) & 0xffff
        hook = c.hook_add(UC_HOOK_CODE, roll_hook)
        try:
            call(0x55640)
        finally:
            c.hook_del(hook)
        expected = bytearray(case['source']); expected[0x9f] = case['successes']
        compare(expected, f'attack roll {i}')
        projectile = bytearray(256); projectile[0x9f] = case['successes']
        assert bytes(c.mem_read(0xff0b00,256)) == bytes(projectile)
        assert getw(0xfff9f8) == case['after_seed'], f'attack RNG {i}'
        assert observed == {key:case[key] for key in ('rolls','pool','difficulty')}, f'attack pool/difficulty/rolls {i}: {observed}, expected {[case[key] for key in ("rolls","pool","difficulty")]}'
        assert c.reg_read(UC_M68K_REG_D0) & 0xffff == 0x1234
    print(f'Original dice: {len(data["dice"])}; ranged attack rolls without ongoing effects: {len(data["attack_rolls"])}; no stubs')
    # Full normal-world launch with both supplied and original rolls.
    # The original-roll variant omits only audio and low-level HUD transfers.
    launch_stubs = (0x55640, 0x530de, 0xf5c, 0x1232, 0x3648)
    for address in launch_stubs:
        c.mem_write(address, b'\x4e\x75')
    full_outcomes = [0, 0, 0]
    for i, original_case in enumerate(data['launches']):
        for use_original_roll in (False, True):
            case = dict(original_case)
            if use_original_roll:
                case.update(original_case['rolled'])
            c.mem_write(0x55640, rom[0x55640:0x55642] if use_original_roll else b'\x4e\x75')
            putw(0xfff9f8, original_case['seed'])
            c.mem_write(0xfff0dd, b'\xac')
            source_address = 0xff0100 + case['source_slot'] * 256
            target_address = 0xff0100 + case['target_slot'] * 256
            actors = bytearray(10 * 256)
            actors[case['source_slot'] * 256:case['source_slot'] * 256 + 256] = bytes(case['source'])
            actors[case['target_slot'] * 256:case['target_slot'] * 256 + 256] = bytes(case['target'])
            c.mem_write(0xff0100, bytes(actors))
            c.mem_write(0xff0b00, bytes(case['pool']))
            w = case['world']
            c.mem_write(0xffe186, bytes([w['mode']]))
            putw(0xffddc4, w['map_id'])
            c.mem_write(0xffe177, bytes([w['override']]))
            putw(0xffdf84, w['engaged'])
            putw(0xffdf86, w['timeout'])
            c.mem_write(0xffe162, struct.pack('>I', w['event']))
            putw(0xffdf40, w['environment'])
            c.mem_write(0xffffff, bytes([w['last']]))
            # Absolute-short $ffff is sign-extended by the CPU; the Genesis bus
            # masks to 24 bits. Unicorn needs that last-RAM-byte alias explicitly.
            c.mem_write(0xffffffff, bytes([w['last']]))
            c.mem_write(0xffdd6d, bytes([w['alternate']]))
            putw(0xffdd60, w['origin'][0]); putw(0xffdd62, w['origin'][1])
            putw(0xffdfe6, 0xffff if w['busy'] else 0)
            putw(0xffdfd8, 0)  # Live initialization, not the read-only probe.
            calls = {'roll': 0, 'sounds': [], 'dice': 0}
            def launch_hook(uc, address, size, user):
                if address == 0x55640:
                    calls['roll'] += 1
                    if not use_original_roll:
                        uc.mem_write(source_address + 0x9f, bytes([case['successes']]))
                        uc.mem_write(uc.reg_read(UC_M68K_REG_A1) + 0x9f, bytes([case['successes']]))
                elif address == 0xd3a:
                    calls['dice'] += 1
                elif address == 0x530de:
                    calls['sounds'].append(uc.reg_read(UC_M68K_REG_D0) & 0xffff)
            hook = c.hook_add(UC_HOOK_CODE, launch_hook)
            c.reg_write(UC_M68K_REG_A0, source_address)
            try:
                call(0x1514e)
            except Exception as error:
                raise AssertionError(f'full launch {i}, PC={c.reg_read(UC_M68K_REG_PC):x}') from error
            finally:
                c.hook_del(hook)
            full_outcomes[case['outcome']] += 1
            actors[case['source_slot'] * 256:case['source_slot'] * 256 + 256] = bytes(case['after_source'])
            actors[case['target_slot'] * 256:case['target_slot'] * 256 + 256] = bytes(case['after_target'])
            compare(actors, f'full launch actors {i}')
            got_pool = bytes(c.mem_read(0xff0b00, 1536))
            assert got_pool == bytes(case['after_pool']), f'full launch pool {i}: {[(hex(j),x,y) for j,(x,y) in enumerate(zip(case["after_pool"],got_pool)) if x != y][:15]}'
            expected = case['after_world']
            assert (getw(0xffdf84), getw(0xffdf86)) == (expected['engaged'], expected['timeout'])
            assert (getw(0xffdd60), getw(0xffdd62)) == tuple(expected['origin'])
            assert bytes(c.mem_read(0xffdd6d, 1)) == bytes([expected['alternate']])
            assert getw(0xffdd3a) == (0xffff if expected['burst'] else 0)
            assert getw(0xffdfe6) == (0xffff if expected['busy'] else 0)
            assert calls['roll'] == (0 if case['outcome'] == 0 else 1)
            assert calls['sounds'] == ([] if case['sound'] is None else [case['sound']])
            assert bool(c.reg_read(UC_M68K_REG_SR) & 1) == (case['outcome'] != 2)
            if use_original_roll:
                assert getw(0xfff9f8) == case['after_seed'], f'launch RNG {i}'
                assert calls['dice'] == case['rolls'], f'launch dice count {i}'
                assert bytes(c.mem_read(0xfff0dd,1)) == bytes([0xac if case['successes'] is None else case['successes']])
    for address in launch_stubs:
        c.mem_write(address, rom[address:address + 2])
    assert all(full_outcomes), full_outcomes
    print(f'Normal-world launch pairs with supplied and original rolls: {len(data["launches"])} (rejected/FX-full/initialized={full_outcomes}); only audio/HUD transfers omitted in the original-roll variant')
    # Omit only low-level HUD/VDP transfers. Execute $3404/$3548 on item removal.
    hud_transfers = (0xf5c, 0x1232, 0x3648)
    for address in hud_transfers:
        c.mem_write(address, b'\x4e\x75')
    for i, case in enumerate(data['resources']):
        c.mem_write(0xff0100, bytes(case['before']) + bytes(512))
        c.reg_write(UC_M68K_REG_A0, 0xff0100)
        call(0x38ac)
        compare(case['after'], f'resource {i}')
        assert bytes(c.mem_read(0xff0200, 512)) == bytes(512)
        assert bool(c.reg_read(UC_M68K_REG_D1) & 0xffff) == case['admitted']
    outcomes = [0] * 5
    for i, case in enumerate(data['admissions']):
        c.mem_write(0xff0100, bytes(case['before']) + bytes(512))
        pool = bytearray(6 * 256)
        for slot in range(6):
            pool[slot * 256 + 0x10] = 0x40 | ((case['mask'] >> slot) & 1)
        c.mem_write(0xff0b00, bytes(pool))
        c.mem_write(0xffe186, bytes([case['mode']]))
        putw(0xffdd3a, 0x1234)
        putw(0xffdfe6, 0xffff if case['initial_busy'] else 0)
        c.reg_write(UC_M68K_REG_A0, 0xff0100)
        c.reg_write(UC_M68K_REG_A7, 0xffff00)
        c.reg_write(UC_M68K_REG_SR, 0x2700)
        path = []
        def prefix_hook(uc, address, size, user):
            path.append(address)
            if address in (0x151be, 0x152f4):
                uc.emu_stop()
        hook = c.hook_add(UC_HOOK_CODE, prefix_hook)
        try:
            c.emu_start(0x1514e, 0x400000, count=500_000)
        finally:
            c.hook_del(hook)
        pc = c.reg_read(UC_M68K_REG_PC)
        assert pc in (0x151be, 0x152f4), f'prefix {i} did not stop'
        outcome = 0 if pc == 0x151be else {
            0x15182: 1, 0x1519c: 2, 0x151a8: 2, 0x151b2: 3, 0x151ba: 4
        }[path[-2]]
        assert outcome == case['outcome'], f'prefix outcome {i}'
        outcomes[outcome] += 1
        if outcome == 0:
            assert c.reg_read(UC_M68K_REG_A1) == 0xff0b00 + case['slot'] * 256
            assert c.reg_read(UC_M68K_REG_D0) & 0xffff == 9 + case['slot']
        compare(case['after'], f'admission {i}')
        assert bytes(c.mem_read(0xff0b00, len(pool))) == bytes(pool)
        assert bytes(c.mem_read(0xff0200, 512)) == bytes(512)
        assert getw(0xffdd3a) == (0xffff if case['burst'] else 0)
        assert getw(0xffdfe6) == (0xffff if case['busy'] else 0)
    for address in hud_transfers:
        c.mem_write(address, rom[address:address + 2])
    assert all(outcomes), outcomes
    print(f"Resource checks: {len(data['resources'])}; launch prefixes: {len(data['admissions'])} (ready/mode/state/empty/pool={outcomes}); only HUD transfers omitted")
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
    # Live handler: execute movement, actor scans and burst rolls unchanged.
    # Stop at the real damage routine entry; omit only sound playback.
    c.mem_write(0x530de,b'\x4e\x75')
    flight_events = [0]*6
    completed_impacts=0
    for i, case in enumerate(data['flights']):
        grid = data['line_grids'][case['grid']]
        c.mem_write(0xff0000,bytes(65536))
        c.mem_write(0xff0100,bytes(case['actors']))
        c.mem_write(0xff0b00,bytes(case['before']))
        c.mem_write(0xff1700,bytes(grid['cells']))
        c.mem_write(0xff3000,b''.join(struct.pack('>5H',*r) for r in grid['rectangles']))
        c.mem_write(0xffddec,struct.pack('>I',0xff3000))
        for at,value in zip([0xffddd6,0xffddd8,0xffddd2,0xffddd4],grid['bounds']):putw(at,value)
        c.mem_write(0xffef10,bytes(case['samples']))
        putw(0xffdf68,0xffff if case['deactivate'] else 0)
        putw(0xffdd50,0xabcd)
        putw(0xfff9f8,case['seed'])
        c.mem_write(0xfff0dd,b'\xac')
        observed = {'steps':0,'rolls':0,'burst':False,'sound':None,'actor':None}
        def flight_hook(uc,address,size,user):
            if address == 0x14f26: observed['steps'] += 1
            elif address == 0xd3a: observed['rolls'] += 1
            elif address == 0x55640: observed['burst'] = True
            elif address == 0x530de: observed['sound'] = uc.reg_read(UC_M68K_REG_D0)&0xffff
            elif address in (0x15b62,0x55948):
                observed['actor'] = (uc.reg_read(UC_M68K_REG_A1)-0xff0100)//256
                if address == 0x55948: uc.emu_stop()
        hook = c.hook_add(UC_HOOK_CODE,flight_hook)
        c.reg_write(UC_M68K_REG_A0,0xff0b00)
        c.reg_write(UC_M68K_REG_A7,0xffff00)
        c.reg_write(UC_M68K_REG_SR,0x2700)
        c.mem_write(0xffff00,struct.pack('>I',0x400000))
        try:
            c.emu_start(0x14e0a,0x400000,count=500_000)
        finally:
            c.hook_del(hook)
        pc = c.reg_read(UC_M68K_REG_PC)
        assert pc in (0x400000,0x55948), f'flight {i} did not return/reach damage'
        event = (2 if pc == 0x55948 else 1 if observed['sound'] is not None else
                 4 if observed['burst'] else 3 if case['before'][0x11]&8 and
                 struct.unpack_from('>h',bytes(case['before']),0x16)[0]>=0 else
                 5 if case['before'][0x11]&8 else 0)
        assert event == case['event'], f'flight event {i}: {event} != {case["event"]}'
        flight_events[event] += 1
        got = bytes(c.mem_read(0xff0b00,256))
        assert got == bytes(case['after']), f'flight bytes {i}: {[(hex(j),x,y) for j,(x,y) in enumerate(zip(case["after"],got)) if x!=y][:15]}'
        compare(case['after_actors'],f'flight actors {i}')
        assert bytes(c.mem_read(0xffef10,4)) == bytes(case['after_samples']), f'flight scratch {i}'
        assert getw(0xfff9f8) == case['after_seed'], f'flight RNG {i}'
        assert getw(0xffdd50) == (0xabcd if case['phase'] is None else case['phase'])
        assert bool(getw(0xffdf68)) == case['after_deactivate']
        assert observed['steps'] == case['steps']
        assert observed['actor'] == case['actor']
        if event == 2:
            assert c.reg_read(UC_M68K_REG_D1)&0xff == case['actors'][case['actor']*256+0x4c]
        assert observed['sound'] == case['sound']
        assert observed['rolls'] == case['rolls']
        assert bytes(c.mem_read(0xfff0dd,1)) == bytes([0xac if case['successes'] is None else case['successes']])
        if case['resolved'] is not None:
            # Replay through actual damage and reaction setup, omitting only
            # sound playback and the low-level party HUD transfer.
            c.mem_write(0x1232,b'\x4e\x75')
            c.mem_write(0xff0100,bytes(case['actors']));c.mem_write(0xff0b00,bytes(case['before']))
            c.mem_write(0xffef10,bytes(case['samples']))
            putw(0xfff9f8,case['seed']);putw(0xffdf68,0xffff if case['deactivate'] else 0)
            for address in (0xfffbf0,0xfff112):putw(address,0)
            for address in (0xffe186,0xfff0ce,0xfffbed,0xfffbfa,0xfff0cc):c.mem_write(address,b'\x00')
            putw(0xffe166,17);c.mem_write(0xfff0dd,b'\xac')
            complete={'sounds':[],'rolls':0}
            def completed_hook(uc,address,size,user):
                if address==0x530de:complete['sounds'].append(uc.reg_read(UC_M68K_REG_D0)&0xffff)
                elif address==0xd3a:complete['rolls']+=1
            hook=c.hook_add(UC_HOOK_CODE,completed_hook)
            c.reg_write(UC_M68K_REG_A0,0xff0b00)
            try:call(0x14e0a)
            finally:c.hook_del(hook)
            end=case['resolved']
            compare(end['actors'],f'complete impact actors {i}')
            assert bytes(c.mem_read(0xff0b00,256))==bytes(end['projectile']),f'complete impact projectile {i}'
            assert getw(0xfff9f8)==end['seed']
            assert complete['rolls']==end['rolls']
            assert complete['sounds']==([] if end['sound'] is None else [end['sound']])
            w=end['world']
            assert bytes(c.mem_read(0xfff0ce,1))==bytes([w['reward']])
            assert getw(0xfffbf0)==w['counter42'] and bytes(c.mem_read(0xfffbfa,1))==bytes([w['counter13']])
            assert getw(0xffe166)==w['timer'] and getw(0xfff112)==w['penetration']
            assert bytes(c.mem_read(0xfff0cc,1))==bytes([w['kind']]) and bytes(c.mem_read(0xfff0dd,1))==bytes([w['successes']])
            completed_impacts+=1
            c.mem_write(0x1232,rom[0x1232:0x1234])
    c.mem_write(0x530de,rom[0x530de:0x530e0])
    assert all(flight_events), flight_events
    print(f'Live projectile handlers: {len(data["flights"])} (moving/miss/damage-pending/animating/restarted/inactive={flight_events}); only audio omitted, damage stopped at entry')
    print(f'Complete live impacts through damage and reaction: {completed_impacts}; only audio/HUD transfers omitted')
print(f'PASS: {len(data["selections"])} target-list/acquisition cases ({replaced} random replacements, {retained} retained targets, {empty} empty-list callbacks); {len(data["range"])} close-range probes; {len(data["ticks"])} refresh/retention/RNG phases; {len(data["lines"])} projectile probes (hit/blocked/limit={line_counts[:3]})')
