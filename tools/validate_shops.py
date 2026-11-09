"""Compare Ares purchases, bargaining and explicit firearm selection to M68000."""
import argparse,hashlib,json,struct,subprocess,tempfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_M68K,UC_MODE_BIG_ENDIAN,UC_HOOK_CODE
from unicorn.m68k_const import *
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom',type=Path);a=p.parse_args()
rom=a.rom.read_bytes();assert hashlib.sha256(rom).hexdigest()=='92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e'
root=Path(__file__).resolve().parents[1]
c=Uc(UC_ARCH_M68K,UC_MODE_BIG_ENDIAN);c.ctl_set_cpu_model(UC_CPU_M68K_M68000)
for base,size in [(0,0x200000),(0xff0000,65536),(0x400000,4096)]:c.mem_map(base,size)
c.mem_write(0,rom)
def w(at,v):c.mem_write(at,struct.pack('>H',v))
def l(at,v):c.mem_write(at,struct.pack('>I',v))
def rl(at):return struct.unpack('>I',c.mem_read(at,4))[0]
def reset():
 c.mem_write(0xff0000,bytes(65536));c.reg_write(UC_M68K_REG_SR,0x2700)
 for r in [UC_M68K_REG_D0,UC_M68K_REG_D1,UC_M68K_REG_D2,UC_M68K_REG_D3,UC_M68K_REG_D4,UC_M68K_REG_D5,UC_M68K_REG_D6,UC_M68K_REG_D7,UC_M68K_REG_A0,UC_M68K_REG_A1,UC_M68K_REG_A2,UC_M68K_REG_A3,UC_M68K_REG_A4,UC_M68K_REG_A5,UC_M68K_REG_A6]:c.reg_write(r,0)
def call(start,stop=0x400000):
 c.reg_write(UC_M68K_REG_A7,0xffff00);l(0xffff00,0x400000)
 hook=c.hook_add(UC_HOOK_CODE,lambda uc,at,size,user:uc.emu_stop() if at==stop else None)
 try:c.emu_start(start,stop,count=100000)
 finally:c.hook_del(hook)
 assert c.reg_read(UC_M68K_REG_PC)==stop,(hex(start),hex(c.reg_read(UC_M68K_REG_PC)))
def patch_ui(addresses):
 for at in addresses:c.mem_write(at,b'\x4e\x75');c.ctl_remove_cache(at,at+2)
with tempfile.TemporaryDirectory() as tmp:
 out=Path(tmp)/'data';subprocess.run([str(root/'target/release/shadowrealm'),'shop-data',str(a.rom.resolve()),str(out)],check=True)
 data=json.loads((out/'shops.json').read_text())
 # Verify real location dispatcher through the opening; no gameplay stubs.
 reset();c.mem_write(0xfffbf8,b'\x02\x0b');call(0x1fa78,0x1f246)
 assert c.reg_read(UC_M68K_REG_A0)==0x110dd2
 assert c.mem_read(0xfff0b6,2)==b'\x01\xff'
 print('PASS: original Ares location, opening and shop/type selection',flush=True)
 # Omit only display, waiting and confirmation input. The complete stock,
 # inventory, price, bargaining, funds, cap and write branches still execute.
 patch_ui([0x11c0,0xc0da,0xc216,0xd94,0x57b8c])
 for i,t in enumerate(data['purchases']):
  reset();at=0xff0100+t['slot']*256;c.mem_write(at,bytes(t['before']));l(0xfffc04,at);l(0xfffbfe,t['balance']);c.mem_write(0xfff0b6,b'\x01\xff');c.mem_write(0xfff0d0,bytes([t['row']]));c.mem_write(0xfff0dc,bytes([0 if t['confirmed'] else 1]));c.reg_write(UC_M68K_REG_D0,1)
  call(0x56e3a if t['category']==0 else 0x56e44)
  assert bytes(c.mem_read(at,256))==bytes(t['after']),f'purchase actor {i}'
  assert rl(0xfffbfe)==t['after_balance'],f'purchase balance {i}'
  assert rl(0xfff0be)==t['price'],f'quote {i}: {rl(0xfff0be)} != {t["price"]}'
  assert c.mem_read(0xfff112,1)[0]==(255 if t['unavailable'] else 0),f'stock eligibility {i}'
 print(f'PASS: {len(data["purchases"])} complete original Ares purchases/refusals, prices, negotiating values, clip byte boundaries and cancellations; only display/wait/confirmation UI omitted',flush=True)
 patch_ui([0x1232,0x3404,0x11624])
 for i,t in enumerate(data['equipment']):
  reset();at=0xff0100+t['slot']*256;c.mem_write(at,bytes(t['before']));l(0xfffc04,at);c.mem_write(0xfff0d1,bytes([t['target']//2]));c.mem_write(0xfff0d0,bytes([t['target']%2]));call(0x12168)
  actual=bytes(c.mem_read(at,256));expected=bytes(t['after'])
  assert actual==expected,f'equipment {i}, changes {[(hex(j),actual[j],expected[j]) for j in range(256) if actual[j]!=expected[j]]}'
 print(f'PASS: {len(data["equipment"])} complete original explicit firearm selections, toggles, spell returns, ammo rounding and inactive actors; only HUD/menu redraw omitted',flush=True)
