"""Compare ordinary unarmed damage and actor decisions with original M68000."""
import argparse,hashlib,json,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN,UC_HOOK_CODE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args()
rom=a.rom.read_bytes();assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1];c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for at,size in [(0,0x200000),(0xff0000,65536),(0x400000,4096)]:c.mem_map(at,size)
c.mem_write(0,rom);c.mem_write(0x1232,b'\x4e\x75') # HUD tile transfer only.
def w(at,v):c.mem_write(at,struct.pack('>H',v))
def l(at,v):c.mem_write(at,struct.pack('>I',v))
def rw(at):return struct.unpack('>H',c.mem_read(at,2))[0]
def reset():
 c.mem_write(0xff0000,bytes(65536));c.reg_write(UC_M68K_REG_SR,0x2700)
 for r in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(r,0)
def call(start):
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000);c.emu_start(start,0x400000,count=10000000);assert c.reg_read(UC_M68K_REG_PC)==0x400000
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'data';subprocess.run([str(root/'target/release/shadowrealm'),'unarmed-data',str(a.rom.resolve()),str(out)],check=True);data=json.loads((out/'unarmed.json').read_text())
 for i,t in enumerate(data['damage']):
  reset();c.mem_write(0xff0100,bytes(t['before']));w(0xfff9f8,t['seed']);c.mem_write(0xffe186,bytes([t['mode']]));c.mem_write(0xfff0ce,bytes([t['count']]));l(0xfffc04,0xff0100)
  c.reg_write(UC_M68K_REG_A0,0xff0100+t['source']*256);c.reg_write(UC_M68K_REG_A1,0xff0100+t['target']*256);call(0x55c76)
  actual=bytes(c.mem_read(0xff0100,len(t['after'])));expected=bytes(t['after'])
  assert actual==expected,f'damage {i}: {[(hex(j),actual[j],expected[j]) for j in range(len(actual)) if actual[j]!=expected[j]][:20]}'
  assert rw(0xfff9f8)==t['after_seed'],f'RNG {i}: {rw(0xfff9f8)} != {t["after_seed"]}'
  assert c.mem_read(0xfff0ce,1)[0]==t['after_count'],f'reward {i}'
  assert c.mem_read(0xfff0dd,1)[0]==t['last'],f'last dice {i}'
 print(f'PASS: {len(data["damage"])} complete original unarmed damage decisions, all actor bytes, armor/foci/cyberware, defense/resistance, health/condition, mode immunity, reward and RNG; only HUD tile transfer omitted',flush=True)
 # Complete ordinary attack, including real range, allocation, alert, damage and
 # reaction. Omit only audio and HUD tiles. Pool exhaustion still executes.
 c.mem_write(0x530de,b'\x4e\x75');c.ctl_remove_cache(0x530de,0x530e0)
 for i,t in enumerate(data['strikes']):
  reset();c.mem_write(0xff0100,bytes(t['before']));c.mem_write(0xff0b00,bytes(t['pool_before']));w(0xfff9f8,t['seed']);w(0xffddc4,2);l(0xfffc04,0xff0100)
  c.reg_write(UC_M68K_REG_A0,0xff0100+t['source']*256);call(0xb1dc)
  actual=bytes(c.mem_read(0xff0100,len(t['after'])));expected=bytes(t['after']);assert actual==expected,f'strike {i}: {[(hex(j),actual[j],expected[j]) for j in range(len(actual)) if actual[j]!=expected[j]][:20]}'
  actual=bytes(c.mem_read(0xff0b00,len(t['pool'])));expected=bytes(t['pool']);assert actual==expected,f'strike FX {i}: {[(hex(j),actual[j],expected[j]) for j in range(len(actual)) if actual[j]!=expected[j]][:20]}'
  assert rw(0xfff9f8)==t['after_seed'] and rw(0xffdf84)==t['engaged'] and rw(0xffdf86)==t['timeout'],f'strike RNG/alert {i}'
  assert rw(0xffe166)==t['defeat']
 print(f'PASS: {len(data["strikes"])} complete original close attacks, poses, hit FX/full pools, reactions, party defeat, alert and RNG; only audio/HUD omitted',flush=True)
 for i,t in enumerate(data['falls']):
  reset();c.mem_write(0xff0100,bytes(t['before']));c.reg_write(UC_M68K_REG_A0,0xff0100+t['slot']*256)
  c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
  hook=c.hook_add(UC_HOOK_CODE,lambda uc,at,size,user:uc.emu_stop(),begin=0xaa50,end=0xaa50)
  try:c.emu_start(0xa94a,0x400000,count=100000)
  finally:c.hook_del(hook)
  assert c.reg_read(UC_M68K_REG_PC)==0xaa50
  actual=bytes(c.mem_read(0xff0100,len(t['after'])));expected=bytes(t['after']);assert actual==expected,f'fall {i}: {[(hex(j),actual[j],expected[j]) for j in range(len(actual)) if actual[j]!=expected[j]][:20]}'
  assert rw(0xffe166)==t['defeat']
 print(f'PASS: {len(data["falls"])} original zero-condition incapacitations, preserved health, poses/flags and defeat admission; only audio omitted',flush=True)
 for i,t in enumerate(data['clocks']):
  reset();c.mem_write(0xff0100,bytes(t['before']));c.reg_write(UC_M68K_REG_A0,0xff0100);call(0x5fdc)
  c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000);hook=c.hook_add(UC_HOOK_CODE,lambda uc,at,size,user:uc.emu_stop(),begin=0x60ae,end=0x60ae)
  try:c.emu_start(0x600e,0x400000,count=100000)
  finally:c.hook_del(hook)
  actual=bytes(c.mem_read(0xff0100,256));expected=bytes(t['after']);assert actual==expected,f'zero-period FX clock {i}: {[(hex(j),actual[j],expected[j]) for j in range(256) if actual[j]!=expected[j]][:20]}'
 print(f'PASS: {len(data["clocks"])} original zero-period hit-FX animation clocks and terminal clamps',flush=True)

 for i,t in enumerate(data['controlled']):
  reset();c.mem_write(0xff0100,bytes(t['before']));c.mem_write(0xff0b00,bytes(t['pool_before']))
  w(0xfff9f8,t['seed']);w(0xffddc4,2);l(0xfffc04,0xff0100);w(0xffdd22,t['input']);w(0xffdd04,65535);w(0xffdfe6,65535 if t['busy'] else 0)
  c.reg_write(UC_M68K_REG_A0,0xff0100+t['source']*256);call(0xad1a)
  actual=bytes(c.mem_read(0xff0100,len(t['after'])));expected=bytes(t['after'])
  assert actual==expected,f'controlled {i}: {[(hex(j),actual[j],expected[j]) for j in range(len(actual)) if actual[j]!=expected[j]][:20]}'
  actual=bytes(c.mem_read(0xff0b00,len(t['pool'])));expected=bytes(t['pool'])
  assert actual==expected,f'controlled FX {i}'
  assert rw(0xfff9f8)==t['after_seed'] and rw(0xffdf84)==t['engaged'] and rw(0xffdf86)==t['timeout'],f'controlled RNG/alert {i}'
  assert bool(rw(0xffdfe6))==t['after_busy'],f'controlled queue {i}'
  assert c.mem_read(0xfff0ce,1)[0]==t['count'] and c.mem_read(0xfff0dd,1)[0]==t['last'],f'controlled reward/dice {i}'
 print(f'PASS: {len(data["controlled"])} whole original controlled unarmed entries, all party slots/templates, held/queued input, cooldown, targetless/out-of-range/inactive swings, damage/rewards, full FX pools and RNG; only audio/HUD omitted',flush=True)

 # Complete companion scheduling/target acquisition/attack/pursuit and exhausted
 # firearm conversion. $3404 is HUD refresh; the actual equipment rules execute.
 c.mem_write(0x3404,b'\x4e\x75');c.ctl_remove_cache(0x3404,0x3406)
 for i,t in enumerate(data['companions']):
  reset();c.mem_write(0xff0100,bytes(t['before']));w(0xfff9f8,t['seed']);w(0xffddc4,2);l(0xfffc04,0xff0100)
  for at,v in zip((0xffddd6,0xffddd8,0xffddd2,0xffddd4),(0,0,2048,2048)):w(at,v)
  l(0xffddec,0xff3000);w(0xffddea,128);w(0xffdf84,65535);w(0xffdf86,17);w(0xffdf94,1);l(0xffdf96,0xff0400)
  c.mem_write(0xffe175,bytes([255 if t['serviced'] else 0]));c.mem_write(0xffef10,b'\x11\x22\x33\x44')
  c.reg_write(UC_M68K_REG_A0,0xff0100+t['source']*256);call(0x138b8)
  actual=bytes(c.mem_read(0xff0100,len(t['after'])));expected=bytes(t['after'])
  assert actual==expected,f'companion {i}: {[(hex(j),actual[j],expected[j]) for j in range(len(actual)) if actual[j]!=expected[j]][:20]}'
  assert bytes(c.mem_read(0xff0b00,1536))==bytes(t['pool']),f'companion FX {i}'
  assert rw(0xfff9f8)==t['after_seed'] and rw(0xffdf84)==t['engaged'] and rw(0xffdf86)==t['timeout'],f'companion RNG/alert {i}'
  assert bool(c.mem_read(0xffe175,1)[0])==t['after_serviced'],f'companion schedule {i}'
  assert bytes(c.mem_read(0xffef10,4))==bytes(t['samples']),f'companion scratch {i}'
  assert c.mem_read(0xfff0ce,1)[0]==t['count'] and c.mem_read(0xfff0dd,1)[0]==t['last'],f'companion reward/dice {i}'
 print(f'PASS: {len(data["companions"])} whole original companion decisions, both slots, target acquisition, pursuit, close attacks, cooldown, exhausted firearms and finite reloads, actor/FX state, scheduler and RNG; only audio/HUD omitted',flush=True)
