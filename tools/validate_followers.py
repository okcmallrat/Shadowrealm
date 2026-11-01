"""Compare the bounded noncombat follower routines with original 68000 execution.

Uses the original collision raster/queries and the pre-existing ROM oracle.
Dynamic actor avoidance and combat are excluded explicitly; fallback wall hugging
is compared through rotated pocket decisions and complete scheduler/movement traces.
"""
import argparse, hashlib, json, struct, subprocess, tempfile
from pathlib import Path
from unicorn import Uc, UC_ARCH_M68K, UC_MODE_BIG_ENDIAN, UC_HOOK_CODE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args();rom=a.rom.read_bytes()
assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1]
def long(p):return struct.unpack_from('>I',rom,p)[0]
def cpu():
 c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
 for base,size in [(0,0x200000),(0xff0000,0x10000),(0x400000,0x1000)]:c.mem_map(base,size)
 c.mem_write(0,rom);c.reg_write(UC_M68K_REG_SR,0x2700);return c
def reset(c):
 for reg in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(reg,0)
 c.reg_write(UC_M68K_REG_SR,0x2700)
def call(c,start,stop=0x400000):
 c.reg_write(UC_M68K_REG_A7,0xffff00);c.mem_write(0xffff00,struct.pack('>I',0x400000))
 # Explicit hook also catches endpoints inside a previously cached Unicorn TB.
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,address,size,user:uc.emu_stop(),begin=stop,end=stop)
 try:c.emu_start(start,stop,count=500_000)
 finally:c.hook_del(hook)
 return c.reg_read(UC_M68K_REG_PC)
def actual(c):return bytes(c.mem_read(0xff0200,256))
def compare(c,case,label):
 expected=bytes(case['after']);got=actual(c)
 if got!=expected:
  differences=[f'{i:x}: native={expected[i]:02x}, original={got[i]:02x}' for i in range(256) if expected[i]!=got[i]]
  raise AssertionError(f'{label} map {case["map"]} pos {bytes(case["before"])[:8].hex()} differences {differences[:24]}')
with tempfile.TemporaryDirectory() as temp:
 out=Path(temp)/'followers';subprocess.run([str(root/'target/release/shadowrealm'),'follower-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'followers.json').read_text());c=cpu()
 for i,case in enumerate(data['math']):
  reset(c)
  for reg,value in [(UC_M68K_REG_D5,case['from'][0]),(UC_M68K_REG_D4,case['from'][1]),(UC_M68K_REG_D3,case['to'][0]),(UC_M68K_REG_D2,case['to'][1])]:c.reg_write(reg,value)
  assert call(c,0x15d3c)==0x400000
  assert [c.reg_read(UC_M68K_REG_D2)&65535,c.reg_read(UC_M68K_REG_D3)&65535]==[case['angle'],case['range']],f'heading case {i}'
  for reg,value in [(UC_M68K_REG_D0,case['from'][0]),(UC_M68K_REG_D1,case['from'][1]),(UC_M68K_REG_D2,case['to'][0]),(UC_M68K_REG_D3,case['to'][1])]:c.reg_write(reg,value)
  assert call(c,0x15ca4)==0x400000
  assert c.reg_read(UC_M68K_REG_D0)&65535==case['stop_distance'],f'distance case {i}'
 cpus={}
 for map_id in [0,3,5,10]:
  c=cpu();c.mem_write(0xffddc4,struct.pack('>H',map_id));descriptor=long(0x3232+map_id*4);c.mem_write(0xffddc6,rom[descriptor:descriptor+96]);c.mem_write(0xffdf8a,b'\xff\xff');assert call(c,0xee10)==0x400000
  cpus[map_id]=c
 supported=withheld=boundary=wallhug=0
 for i,case in enumerate(data['decisions']):
  c=cpus[case['map']];reset(c);c.mem_write(0xff0100,bytes(case['leader']));c.mem_write(0xff0200,bytes(case['before']));c.mem_write(0xfffc04,struct.pack('>I',0xff0100));c.reg_write(UC_M68K_REG_A0,0xff0200)
  pc=call(c,0x14508,0x14598)
  assert pc==0x14598,f'formation {i} stopped at {pc:x}'
  assert list(struct.unpack('>HH',c.mem_read(0xff0296,4)))==case['goal'],f'formation case {i}'
  c.mem_write(0xff0200,bytes(case['before']));reset(c);c.reg_write(UC_M68K_REG_A0,0xff0200)
  hook=c.hook_add(UC_HOOK_CODE,lambda uc,address,size,user:uc.emu_stop(),begin=0x1470e,end=0x1470e)
  outside=[False]
  def grid_boundary(uc,address,size,user):
   reg=UC_M68K_REG_D4 if address in [0x14872,0x1489c] else UC_M68K_REG_D1
   if uc.reg_read(reg)&65535>=4096:outside[0]=True;uc.emu_stop()
  boundary_hooks=[c.hook_add(UC_HOOK_CODE,grid_boundary,begin=p,end=p) for p in [0x148b8,0x14866,0x14872,0x14890,0x1489c]]
  try:pc=call(c,0x14508)
  except Exception as e:raise RuntimeError(f'decision {i}, map {case["map"]}, PC {c.reg_read(UC_M68K_REG_PC):x}, before {bytes(case["before"])[:8].hex()}') from e
  finally:
   c.hook_del(hook)
   for h in boundary_hooks:c.hook_del(h)
  if not case['supported']:
   assert pc==0x1470e or outside[0],f'withheld a route that did not require fallback/boundary: case {i}, PC {pc:x}'
   withheld+=1
   if outside[0]:boundary+=1
   else:wallhug+=1
   continue
  assert pc==0x400000,f'unexpected fallback/timeout in decision {i}: {pc:x}'
  compare(c,case,f'decision {i}');supported+=1
 for i,case in enumerate(data['movement']):
  c=cpus[case['map']];reset(c);c.mem_write(0xff0200,bytes(case['before']));c.reg_write(UC_M68K_REG_A0,0xff0200)
  assert call(c,0xaaa0,0xaac6)==0xaac6
  compare(c,case,f'movement {i}')
 fallback_ticks=0
 visits={address:0 for address in [0x1470e,0x14764,0x14800,0x14914,0x14a54,0x14aaa]}
 def count_visit(uc,address,size,user):
  visits[address]+=1
 for trace in [data['timeline'], *data['fallback_timelines']]:
  c=cpu();c.mem_write(0xff0100,bytes(trace['initial']));c.mem_write(0xff1700,bytes(trace['cells']));c.mem_write(0xfffc04,struct.pack('>I',0xff0100))
  for address,value in [(0xffddd2,512),(0xffddd4,512),(0xffddd6,0),(0xffddd8,0),(0xffddea,32)]:c.mem_write(address,struct.pack('>H',value))
  hooks=[c.hook_add(UC_HOOK_CODE,count_visit,begin=address,end=address) for address in visits]
  for tick,expected in enumerate(trace['ticks']):
   c.mem_write(0xffe175,b'\x00')
   if tick<trace['warmup_frames']:
    reset(c);c.reg_write(UC_M68K_REG_A0,0xff0100);assert call(c,0xae5e)==0x400000;assert call(c,0xaaa0,0xaac6)==0xaac6
   for actor in range(0xff0200,0xff0100+len(expected),256):
    reset(c);c.reg_write(UC_M68K_REG_A0,actor);assert call(c,0x138b8)==0x400000
    if tick<trace['warmup_frames']:assert call(c,0xae5e)==0x400000
    reset(c);c.reg_write(UC_M68K_REG_A0,actor);assert call(c,0xaaa0,0xaac6)==0xaac6
   got=bytes(c.mem_read(0xff0100,len(expected)))
   assert got==bytes(expected),f'timeline rotation {trace.get("rotation", "regular")} tick {tick} differs at {[(hex(i),expected[i],got[i]) for i in range(len(expected)) if got[i]!=expected[i]][:24]}'
  for hook in hooks:c.hook_del(hook)
  if 'rotation' in trace:fallback_ticks+=len(trace['ticks'])
 for i,case in enumerate(data['fallback_decisions']):
  c=cpu();c.mem_write(0xff1700,bytes(data['fallback_timelines'][case['rotation']]['cells']));c.mem_write(0xfffc04,struct.pack('>I',0xff0100))
  for address,value in [(0xffddd2,512),(0xffddd4,512),(0xffddd6,0),(0xffddd8,0),(0xffddea,32)]:c.mem_write(address,struct.pack('>H',value))
  reset(c);c.mem_write(0xff0100,bytes(case['leader']));c.mem_write(0xff0200,bytes(case['before']));c.reg_write(UC_M68K_REG_A0,0xff0200)
  hooks=[c.hook_add(UC_HOOK_CODE,count_visit,begin=address,end=address) for address in visits]
  assert case['supported'];assert call(c,0x14508)==0x400000
  for hook in hooks:c.hook_del(hook)
  compare(c,dict(case,map=f'rotation {case["rotation"]}'),f'fallback decision {i}')
 assert all(visits.values()),f'wall-hugging branches not exercised: {visits}'
 c=cpu();c.mem_write(0xff1700,bytes(data['boxed_cells']));c.mem_write(0xfffc04,struct.pack('>I',0xff0100))
 for address,value in [(0xffddd2,512),(0xffddd4,512),(0xffddd6,0),(0xffddd8,0)]:c.mem_write(address,struct.pack('>H',value))
 for case in data['closed_cases']:
  reset(c);c.mem_write(0xff0100,bytes(case['leader']));c.mem_write(0xff0200,bytes(case['before']));c.reg_write(UC_M68K_REG_A0,0xff0200)
  assert case['supported'];assert call(c,0x14508)==0x400000
  compare(c,dict(case,map='boxed'), 'closed pocket')
  assert bytes(case['after'])[:8]==bytes(case['before'])[:8], 'closed-pocket decision moved actor'
print(f'PASS: {len(data["math"])} heading/distance pairs; {len(data["decisions"])} formation goals, {supported} supported decisions, {withheld} withheld raster routes; {len(data["movement"])} movement records; 900 three-runner scheduler/warm-up ticks; {fallback_ticks} rotated wall-hugging ticks; {len(data["fallback_decisions"])} fallback/waypoint decisions; {len(data["closed_cases"])} closed-pocket decisions')
