"""Compare Gunderson offers, courier visits/rewards, failure and service routing."""
import argparse, hashlib, json, struct, subprocess, tempfile
from pathlib import Path
from unicorn import Uc, UC_ARCH_M68K, UC_MODE_BIG_ENDIAN, UC_HOOK_CODE
from unicorn.m68k_const import *

p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args()
rom=a.rom.read_bytes();assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1]
c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for base,size in [(0,0x200000),(0xff0000,65536),(0x400000,4096)]:c.mem_map(base,size)
c.mem_write(0,rom)
def w(at,v):c.mem_write(at,struct.pack('>H',v))
def l(at,v):c.mem_write(at,struct.pack('>I',v))
def rw(at):return struct.unpack('>H',c.mem_read(at,2))[0]
def rl(at):return struct.unpack('>I',c.mem_read(at,4))[0]
def reset():
 c.mem_write(0xff0000,bytes(65536));c.reg_write(UC_M68K_REG_SR,0x2700)
 for r in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(r,0)
def call(start,stop=0x400000):
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,at,size,user:uc.emu_stop() if at==stop else None)
 try:c.emu_start(start,stop,count=200000)
 finally:c.hook_del(hook)
 assert c.reg_read(UC_M68K_REG_PC)==stop,(hex(start),hex(c.reg_read(UC_M68K_REG_PC)))
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'jobs';subprocess.run([str(root/'target/release/shadowrealm'),'job-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'jobs.json').read_text());counts=[0,0,0]
 for i,t in enumerate(data['offers']):
  reset();c.mem_write(0xfffbec,bytes(t['before']));c.mem_write(0xfffc1c,bytes([t['flags']]));l(0xfffc22,t['carried']);w(0xfff9f8,t['seed']);c.mem_write(0xff018c,bytes([t['negotiating_stat']]))
  call(0x1e69c)
  assert bytes(c.mem_read(0xfffbec,10))==bytes(t['after']),f'offer {i}'
  assert c.mem_read(0xfffc1c,1)[0]==t['after_flags'] and rl(0xfffc22)==t['carried'],f'offer flags {i}'
  assert rw(0xfff9f8)==t['after_seed'],f'offer RNG {i}'
  assert rw(0xfff0c0)==t['base_pay'],f'offer base pay {i}'
  counts[t['after'][1]]+=1
 print(f'Original Gunderson offers: {len(data["offers"])} (ghoul/bodyguard/courier={counts}); no stubs',flush=True)
 events=[0,0,0]
 for i,t in enumerate(data['visits']):
  reset();c.mem_write(0xfffbec,bytes(t['raw']));c.mem_write(0xfffc1c,bytes([t['flags']]));l(0xfffc22,t['carried']);l(0xfffbfe,t['balance']);c.mem_write(0xff0100,bytes(t['before']));c.mem_write(0xfffbf8,bytes([t['district'],t['location']]));w(0xfff9f8,321)
  call(0x1eedc)
  assert bytes(c.mem_read(0xfffbec,10))==bytes(t['after_raw']),f'visit record {i}'
  assert c.mem_read(0xfffc1c,1)[0]==t['after_flags'] and rl(0xfffc22)==t['after_carried'],f'visit flags {i}'
  assert rl(0xfffbfe)==t['after_balance'],f'visit payment {i}'
  assert bytes(c.mem_read(0xff0100,768))==bytes(t['after']),f'visit actors/Karma {i}'
  assert c.mem_read(0xfff0d5,1)[0]==t['event'] and rw(0xfff9f8)==321,f'visit event/RNG {i}'
  events[{0:0,255:1,1:2}[t['event']]]+=1
 print(f'Original courier visits: {len(data["visits"])} (none/pickup/payment={events}); no stubs',flush=True)
 # Stop before the dialogue engine. The only omitted call is the failure notice UI.
 c.mem_write(0x1f246,b'\x4e\x75');c.ctl_remove_cache(0x1f246,0x1f248)
 for i,t in enumerate(data['failures']):
  reset();c.mem_write(0xfffbec,bytes(t['raw']));c.mem_write(0xfffc1c,bytes([t['flags']]));l(0xfffc22,t['carried']);call(0xc64c,0xc716)
  assert bytes(c.mem_read(0xfffbec,10))==bytes(t['after_raw']),f'failure record {i}'
  assert c.mem_read(0xfffc1c,1)[0]==t['after_flags'] and rl(0xfffc22)==t['after_carried'],f'failure flags {i}'
 print(f'Original carried-package defeat prefixes: {len(data["failures"])}; only notice UI omitted; stop before medical recovery',flush=True)
 c.mem_write(0x1f246,rom[0x1f246:0x1f248]);c.ctl_remove_cache(0x1f246,0x1f248)
 for t in data['stops']:
  for delivery in [False,True]:
   reset();loc=t['location'];c.mem_write(0xfffbec,bytes([0,2,2,loc,0,2,loc,1,0,50]));c.mem_write(0xfffc1c,bytes([int(delivery)]));c.mem_write(0xfffbf8,bytes([2,loc]));l(0xfffbfe,1000)
   call(0x1f5ca,0x1f246);node=c.reg_read(UC_M68K_REG_A0)
   if loc==0:
    assert node==0x1113d2
    call(0x226ca);node=c.reg_read(UC_M68K_REG_A0)
   assert node==t['delivery' if delivery else 'pickup'],f'courier service {loc}/{delivery}: {hex(node)}'
 print(f'PASS: {len(data["stops"])*2} original Redmond service selections through real mission dispatch, including the prison continuation; no gameplay stubs',flush=True)
 reset();w(0xfffbec,65535);c.mem_write(0xfffbf8,bytes([2,4]));call(0x1f5ca,0x1f246)
 assert c.reg_read(UC_M68K_REG_A0)==0x110d1e and c.mem_read(0xfff0d6,1)[0]==255,'Jump House entrance permission'
 for i,t in enumerate(data['entrances']):
  reset();w(0xffddc4,2);c.mem_write(0xffddc6,bytes(t['saved']));w(0xff0100,t['player'][0]);w(0xff0104,t['player'][1]);c.mem_write(0xffef10,b'\xc4'*4);c.reg_write(UC_M68K_REG_A0,0xff0100)
  call(0xbc36)
  assert [rw(0xffdf44+j*2) for j in range(6)]==sum(t['return_positions'],[]),f'Jump House return formation {i}'
  assert [rw(0xffdf50+j*2) for j in range(2)]==t['camera'],f'Jump House saved camera {i}'
  c.mem_write(0xfffbf9,b'\x04');call(0x167ae,0x1683a)
  assert rw(0xffddc4)==t['target'] and bytes(c.mem_read(0xffde26,98))==struct.pack('>H',2)+bytes(t['saved']),f'Jump House saved route {i}'
  at=struct.unpack_from('>I',rom,0x3232+13*4)[0];c.mem_write(0xffddc6,rom[at:at+96]);call(0x2b34,0x2b76)
  assert [rw(0xffddc6+j*2) for j in range(2)]==t['arrival_camera'],f'Jump House arrival camera {i}'
  assert [rw(0xff0100+s*256+p) for s in range(3) for p in [0,4]]==sum(t['positions'],[]),f'Jump House spawn {i}'
 print(f'PASS: Jump House entry permission plus {len(data["entrances"])} original save/route/return-formation/spawn cases; no stubs',flush=True)
