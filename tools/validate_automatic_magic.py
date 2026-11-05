"""Compare recovered prior-actor D5 and complete automatic companion decisions."""
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
for at in (0x530de,0x1232,0x3404,0xf5c,0x3648):c.mem_write(at,b'\x4e\x75');c.ctl_remove_cache(at,at+2)
def w(at,v):c.mem_write(at,struct.pack('>H',v))
def l(at,v):c.mem_write(at,struct.pack('>I',v))
def rw(at):return struct.unpack('>H',c.mem_read(at,2))[0]
def reset(wall=False):
 c.mem_write(0xff0000,bytes(65536))
 for reg in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(reg,0)
 c.reg_write(UC_M68K_REG_SR,0x2700)
 for at,v in zip((0xffddd6,0xffddd8,0xffddd2,0xffddd4),(0,0,2048,2048)):w(at,v)
 l(0xffddec,0xff3000);w(0xffddea,128);l(0xfffc04,0xff0100);w(0xffdf84,65535)
 c.mem_write(0xff1700,bytes(127 if n//64 in (0,63) or n%64 in (0,63) or (wall and n%64==32) else 0 for n in range(4096)))
def call(start,stop=0x400000):
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,at,size,user:uc.emu_stop() if at==stop else None)
 try:c.emu_start(start,stop,count=500000)
 finally:c.hook_del(hook)
 assert c.reg_read(UC_M68K_REG_PC)==stop,(hex(start),hex(c.reg_read(UC_M68K_REG_PC)))
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'automatic';subprocess.run([str(root/'target/release/shadowrealm'),'automatic-magic-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'automatic.json').read_text())
 for i,t in enumerate(data['contexts']):
  reset()
  if t['solid']:c.mem_write(0xff1700,b'\x7f'*4096)
  c.mem_write(0xff0100,bytes(t['before']));c.reg_write(UC_M68K_REG_A0,0xff0100);c.reg_write(UC_M68K_REG_D5,t['incoming']);call(0xaaa0,0xaac6)
  assert c.reg_read(UC_M68K_REG_D5)&65535==t['d5'],f'prior-actor D5 {i}: {hex(c.reg_read(UC_M68K_REG_D5))} != {hex(t["d5"])}'
 print(f'Prior actor dispatcher/collision D5 residues: {len(data["contexts"])}; no movement/collision stubs',flush=True)
 full=prefixes=equips=casts=heals=0
 for i,t in enumerate(data['decisions']):
  reset(t['wall']);prior=0xff0100+(t['slot']-1)*256
  c.mem_write(prior,bytes(t['prior_before']));c.reg_write(UC_M68K_REG_A0,prior);call(0xaaa0,0xaac6)
  assert c.reg_read(UC_M68K_REG_D5)&65535==t['index'],f'decision caller D5 {i}'
  # The complete decision has explicit incoming world/scratch state; keep the
  # D5 produced by the real preceding actor, not a guessed weapon index.
  c.mem_write(0xff0100,bytes(t['before']));c.mem_write(0xfff9fa,bytes(data['books']));l(0xfffc04,0xff0100)
  for slot in (1,2):c.mem_write(0xfffc0b+slot,bytes([0 if t['before'][slot*256+0x10]&1 else 255]))
  c.mem_write(0xffe175,bytes([255 if t['serviced'] else 0]));c.mem_write(0xffe190,bytes([t['hint']]))
  w(0xffdf84,t['engaged']);w(0xffdf86,17);w(0xffdfde,t['firing']);w(0xfff9f8,t['seed'])
  w(0xffdf94,len(t['npcs']))
  if t['npcs']:c.mem_write(0xffdf96,b''.join(struct.pack('>I',v) for v in t['npcs']))
  c.mem_write(0xffef10,b'\x11\x22\x33\x44');c.mem_write(0xffd86c,bytes(t['occupied']))
  c.reg_write(UC_M68K_REG_A0,0xff0100+t['slot']*256)
  if t['deferred']:
   assert t['deferred']=='Companion melee action is not available yet',(i,t['deferred'])
   call(0x138b8,0x141a0);prefixes+=1
  else:call(0x138b8);full+=1
  got=bytes(c.mem_read(0xff0100,2560));expected=bytes(t['after'])
  assert got==expected,f'automatic companion {i}: {[(hex(j),expected[j],got[j]) for j in range(2560) if got[j]!=expected[j]][:20]}'
  assert rw(0xfff9f8)==t['after_seed'],f'RNG {i}'
  assert c.mem_read(0xffe190,1)[0]==t['after_hint'],f'healing hint {i}'
  assert rw(0xffdf84)==t['after_engaged'] and rw(0xffdf86)==t['after_timeout'],f'alert {i}'
  assert rw(0xffdfde)==t['after_firing'],f'firing {i}'
  assert bytes(c.mem_read(0xffef10,4))==bytes(t['after_samples']),f'scratch {i}: original={list(c.mem_read(0xffef10,4))}, native={t["after_samples"]}'
  assert list(map(bool,c.mem_read(0xffd86c,18)))==t['after_occupied'],f'graphics {i}'
  assert bytes(c.mem_read(0xff0b00,1536))==bytes(t['after_pool']),f'projectiles {i}'
  assert bool(rw(0xffdfe6))==t['busy'] and bool(rw(0xffdd3a))==t['burst'],f'launch signals {i}'
  assert bool(c.mem_read(0xffe175,1)[0])==t['after_serviced'],f'scheduler {i}'
  equips+=t['equips'];casts+=t['casts'];heals+=sum(t['after'][s*256+0xb2]==12 for s in range(3))
 print(f'PASS: {full} complete automatic decisions, {prefixes} exhausted-equipment prefixes, {equips} spell equipment changes, {casts} successful casts ({heals} Heal effects); only audio/HUD/VDP omitted',flush=True)
