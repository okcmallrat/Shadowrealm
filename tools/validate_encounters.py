"""Compare bounded street-encounter gate/selection/placement/spawn with the ROM.
No instruction stubs: execute real RNG, actor/graphics allocation and alert logic.
Scripted encounter callbacks and unsupported combat groups remain outside scope.
"""
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
 for r in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(r,0)
 c.reg_write(UC_M68K_REG_SR,0x2700)
def call(start,stops=()):
 hooks=[c.hook_add(UC_HOOK_CODE,lambda uc,addr,size,user:uc.emu_stop(),begin=s,end=s) for s in stops]
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
 try:c.emu_start(start,0x400000,count=1000000)
 finally:
  for h in hooks:c.hook_del(h)
 pc=c.reg_read(UC_M68K_REG_PC)
 assert pc in (*stops,0x400000),f'Unexpected stop {pc:x}'
 return pc
def load_map(m):
 w(0xffddc4,m);address=struct.unpack_from('>I',rom,0x3232+4*m)[0]
 c.mem_write(0xffddc6,rom[address:address+96]);w(0xffdf8a,0xffff);call(0xee10)
def actors(case):
 expected=bytes(case['after']);got=bytes(c.mem_read(0xff0100,len(expected)))
 assert got==expected,[(hex(i),expected[i],got[i]) for i in range(len(expected)) if expected[i]!=got[i]][:20]
with tempfile.TemporaryDirectory() as temp:
 out=Path(temp)/'out';subprocess.run([str(root/'target/release/shadowrealm'),'encounter-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'encounters.json').read_text())
 for i,t in enumerate(data['selections']):
  reset();c.mem_write(0xfffbf8,bytes([t['map']]));w(0xfff9f8,t['seed']);call(0x59986)
  assert (c.reg_read(UC_M68K_REG_D0)&65535,c.reg_read(UC_M68K_REG_A6),rw(0xfff9f8))==(t['kind'],t['address'],t['after_seed']),f'selection {i}'
 for i,t in enumerate(data['gates']):
  reset();l(0xfffc04,0xff0100);w(0xffdfe8,t['paused']);c.mem_write(0xff0111,bytes([t['flags']]));c.mem_write(0xffe18b,bytes([t['blocked']]));c.mem_write(0xffe16f,bytes([t['script']]));w(0xffdf84,t['engaged']);c.mem_write(0xffe177,bytes([t['count']]));w(0xffdf88,t['tick']);w(0xffde18,t['chance']);w(0xfff9f8,t['seed'])
  pc=call(0x1e166,(0x1e1d0,));assert (pc==0x1e1d0)==t['due'] and rw(0xfff9f8)==t['after_seed'],f'gate {i}'
 for i,t in enumerate(data['positions']):
  reset();load_map(t['map']);w(0xffddc6,t['camera'][0]);w(0xffddc8,t['camera'][1]);call(0x1e40c if t['close'] else 0x1e302)
  result=[];at=0xffdd50
  while rw(at):result.append([rw(at),rw(at+2)]);at+=4
  assert result==t['points'],f'positions {i}: {result} != {t["points"]}'
 for i,t in enumerate(data['groups']):
  reset();w(0xffddc4,t['map']);c.mem_write(0xff0100,bytes(t['before']));w(0xfff9f8,t['seed'])
  for slot in range(3,10):
   actor=t['before'][slot*256:(slot+1)*256]
   if actor[0x10]&9==1:c.mem_write(0xffd86c+struct.unpack_from('>H',bytes(actor),0x30)[0],b'\x01')
  for j,point in enumerate(t['points']):w(0xffdd50+j*4,point[0]);w(0xffdd52+j*4,point[1])
  c.reg_write(UC_M68K_REG_A6,t['group']);call(0x1e228);actors(t)
  assert rw(0xfff9f8)==t['after_seed'] and c.mem_read(0xffe177,1)[0]==t['count'],f'group {i}'
  assert rw(0xffdf84)==(0xffff if t['count'] else 0),f'alert {i}'
 for i,t in enumerate(data['offscreen']):
  reset();load_map(5);c.mem_write(0xff0100,bytes(t['before']));w(0xffdf84,0xffff);l(0xfffc04,0xff0100);c.reg_write(UC_M68K_REG_A0,0xff0400);call(0x13904);actors(t)
 print('PASS: '+', '.join(f'{len(data[k])} {k}' for k in data)+'; no instruction stubs')
