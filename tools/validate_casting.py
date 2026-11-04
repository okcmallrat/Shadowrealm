"""Compare original Flame Dart/Bolt and Mana Zap/Blast casts, damage and effects."""
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
with tempfile.TemporaryDirectory() as temp:
 out=Path(temp)/'casting';subprocess.run([str(root/'target/release/shadowrealm'),'casting-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'casting.json').read_text())
 for address in (0x1232,0x3404,0x530de,0x61a0,0x13be):
  c.mem_write(address,b'\x4e\x75');c.ctl_remove_cache(address,address+2)
 for i,t in enumerate(data['poses']):
  reset();c.mem_write(0xff0100,bytes(t['before']));c.reg_write(UC_M68K_REG_A0,0xff0100);call(0xaf7c)
  assert bytes(c.mem_read(0xff0100,256))==bytes(t['after']),f'cast pose {i}'
 print(f'Original cast poses: {len(data["poses"])}; no stubs',flush=True)
 for i,t in enumerate(data['casts']):
  reset();c.mem_write(0xff0100,bytes(t['before']));w(0xfff9f8,t['seed']);w(0xffdf86,17)
  flags=list(t['occupied'])
  for slot in range(3,10):flags[slot-3]=True
  c.mem_write(0xffd86c,bytes(flags));c.reg_write(UC_M68K_REG_A0,0xff0100);c.reg_write(UC_M68K_REG_A1,0xff0400);call(0x1033e)
  got=bytes(c.mem_read(0xff0100,2560));expected=bytes(t['after'])
  assert got==expected,f'cast {i}: {[(hex(j),expected[j],got[j]) for j in range(2560) if got[j]!=expected[j]][:20]}'
  assert rw(0xfff9f8)==t['after_seed'],f'cast RNG {i}'
  assert rw(0xffdf84)==t['engaged'] and rw(0xffdf86)==t['timeout'],f'cast alert {i}'
  # Failed rolls do not enter graphics allocation or synchronize native NPC flags.
  if t['result']!='Failed':assert list(map(bool,c.mem_read(0xffd86c,18)))==t['after_occupied'],f'cast allocations {i}'
 print(f'Full casts through effect admission and drain: {len(data["casts"])}; only audio/HUD omitted',flush=True)
 for i,t in enumerate(data['damages']):
  reset();c.mem_write(0xff0100,bytes(t['before']));w(0xfff9f8,t['seed']);c.mem_write(0xfff0ce,b'\x0f')
  c.reg_write(UC_M68K_REG_A0,0xff0100+t['source']*256);c.reg_write(UC_M68K_REG_A1,0xff0100+t['target']*256);call(0x55b78)
  got=bytes(c.mem_read(0xff0100,2560));expected=bytes(t['after'])
  assert got==expected,f'magic damage {i}: {[(hex(j),expected[j],got[j]) for j in range(2560) if got[j]!=expected[j]][:20]}'
  assert rw(0xfff9f8)==t['after_seed'] and c.mem_read(0xfff0dd,1)[0]==t['successes'],f'magic resistance RNG {i}'
  assert c.mem_read(0xfff0ce,1)[0]==t['reward']
 print(f'Magic resistance/damage: {len(data["damages"])}; only HUD omitted',flush=True)
 for i,t in enumerate(data['effects']):
  reset();c.mem_write(0xff0100,bytes(t['before']));w(0xfff9f8,t['seed']);c.mem_write(0xfff0ce,b'\x0f');c.mem_write(0xffd86c+t['graphic'],b'\x01')
  c.reg_write(UC_M68K_REG_A0,0xff0400);call({1:0x521e,2:0x5152,8:0x508a,9:0x4fe2}[t['item']])
  got=bytes(c.mem_read(0xff0100,2560));expected=bytes(t['after'])
  assert got==expected,f'effect {i}: {[(hex(j),expected[j],got[j]) for j in range(2560) if got[j]!=expected[j]][:20]}'
  assert rw(0xfff9f8)==t['after_seed'],f'effect RNG {i}'
  assert bool(c.mem_read(0xffd86c+t['graphic'],1)[0])==t['occupied']
  assert c.mem_read(0xfff0ce,1)[0]==t['reward'],f'effect rewards {i}'
 print(f'PASS: {len(data["effects"])} complete effect steps; only audio/HUD/sprite transfers omitted',flush=True)
