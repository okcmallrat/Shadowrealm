"""Compare Heal callback, cast, completion and effect timing with original routines."""
import argparse,hashlib,json,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN,UC_HOOK_CODE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args()
rom=a.rom.read_bytes();assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1]
c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for base,size in [(0,0x200000),(0xff0000,0x10000),(0x400000,0x1000)]:c.mem_map(base,size)
c.mem_write(0,rom)
def w(p,v):c.mem_write(p,struct.pack('>H',v))
def l(p,v):c.mem_write(p,struct.pack('>I',v))
def rw(p):return struct.unpack('>H',c.mem_read(p,2))[0]
def reset():
 c.mem_write(0xff0000,bytes(65536))
 for r in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4]:c.reg_write(r,0)
 c.reg_write(UC_M68K_REG_SR,0x2700)
def call(start,stops=()):
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,addr,size,user:uc.emu_stop() if addr in stops or addr==0x400000 else None)
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
 try:c.emu_start(start,0x400000,count=100000)
 finally:c.hook_del(hook)
 pc=c.reg_read(UC_M68K_REG_PC);assert pc in (*stops,0x400000),hex(pc)
 return pc
def grid(wall=False):
 c.mem_write(0xff1700,bytes([0x7f if wall else 0])*4096);l(0xffddec,0xff3000);w(0xffddea,128)
 for at,value in zip([0xffddd6,0xffddd8,0xffddd2,0xffddd4],[0,0,2048,2048]):w(at,value)
def actors(expected,label):
 got=bytes(c.mem_read(0xff0100,len(expected)));expected=bytes(expected)
 assert got==expected,f'{label}: {[(hex(i),expected[i],got[i]) for i in range(len(expected)) if got[i]!=expected[i]][:20]}'
with tempfile.TemporaryDirectory() as temp:
 out=Path(temp)/'healing';subprocess.run([str(root/'target/release/shadowrealm'),'healing-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'healing.json').read_text())
 for i,t in enumerate(data['callbacks']):
  reset();grid(t['wall']);c.mem_write(0xff0100,bytes(t['before']));w(0xffdf84,0xffff if t['engaged'] else 0);c.reg_write(UC_M68K_REG_A0,0xff0100);call(0xb574)
  actors(t['after'],f'Heal callback {i}')
 print(f'Original Heal initializer movement callbacks: {len(data["callbacks"])}; no stubs',flush=True)
 for address in (0x1232,0x3404,0x530de,0x61a0,0x13be):
  c.mem_write(address,b'\x4e\x75');c.ctl_remove_cache(address,address+2)
 for i,t in enumerate(data['completions']):
  reset();c.mem_write(0xff0100,bytes(t['before']));l(0xfffc04,0xff0100)
  c.reg_write(UC_M68K_REG_A0,0xff0100+t['target']*256);c.reg_write(UC_M68K_REG_A1,0xff0100+t['caster']*256);call(0x55c2c)
  actors(t['after'],f'Heal completion {i}')
 print(f'Original Heal health/condition/fallen-state completions: {len(data["completions"])}; only HUD omitted',flush=True)
 for i,t in enumerate(data['casts']):
  reset();grid();c.mem_write(0xff0100,bytes(t['before']));w(0xfff9f8,t['seed']);w(0xffdf86,17);c.mem_write(0xffd86c,bytes(t['occupied']))
  c.reg_write(UC_M68K_REG_A0,0xff0100+t['caster']*256);c.reg_write(UC_M68K_REG_A1,0xff0100+t['target']*256);call(0x1033e)
  actors(t['after'],f'complete Heal cast {i}')
  assert rw(0xfff9f8)==t['after_seed'],f'Heal cast RNG {i}'
  assert rw(0xffdf84)==t['engaged'] and rw(0xffdf86)==t['timeout'],f'Heal cast alert {i}'
  assert list(map(bool,c.mem_read(0xffd86c,18)))==t['after_occupied'],f'Heal allocations {i}'
 print(f'Complete original Heal casts, self/party targets and drain: {len(data["casts"])}; only audio/HUD omitted',flush=True)
 for i,t in enumerate(data['effects']):
  reset();c.mem_write(0xff0100,bytes(t['before']));l(0xfffc04,0xff0100);c.mem_write(0xffd86c+17,b'\x01');w(0xfff9f8,123)
  c.reg_write(UC_M68K_REG_A0,0xff0100+t['target']*256);call(0x56de)
  actors(t['after'],f'Heal effect {i}');assert rw(0xfff9f8)==t['seed']
  assert bool(c.mem_read(0xffd86c+17,1)[0])==t['occupied']
 print(f'PASS: {len(data["effects"])} complete Heal effect steps; only HUD/sprite transfers omitted',flush=True)
