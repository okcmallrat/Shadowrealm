"""Package the cross-built Windows exploration executable and its runtime DLL."""
import argparse,hashlib,json,shutil,subprocess,zipfile
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('llvm_mingw',type=Path);a=p.parse_args()
root=Path(__file__).resolve().parents[1]
output=root/'artifacts/windows/Shadowrealm-exploration'
output.mkdir(parents=True,exist_ok=True)
exe=root/'target/x86_64-pc-windows-gnullvm/release/shadowrealm-game.exe'
if not exe.is_file():p.error('cross-built executable is missing')
# Require a real Windows PE executable, not a renamed native Linux binary.
if exe.read_bytes()[:2]!=b'MZ':p.error('invalid Windows executable')
shutil.copy2(exe,output/'Shadowrealm.exe')
shutil.copy2(a.llvm_mingw/'x86_64-w64-mingw32/bin/libunwind.dll',output/'libunwind.dll')
shutil.copy2(a.llvm_mingw/'LICENSE.TXT',output/'LLVM-LICENSE.txt')
shutil.copy2(root/'docs/WINDOWS-README.txt',output/'README.txt')
# Include the licenses of the Rust desktop dependencies, preserving original text.
cargo_home=Path(__import__('os').environ.get('CARGO_HOME',str(Path.home()/'.cargo')))
licenses=output/'licenses';licenses.mkdir(exist_ok=True)
for crate in ['minifb-0.28.0','winapi-0.3.9','raw-window-handle-0.6.2']:
    found=list((cargo_home/'registry/src').glob(f'*/{crate}'))
    if not found:p.error(f'license source missing for {crate}')
    for source in found[0].glob('LICENSE*'):
        if source.is_file():shutil.copy2(source,licenses/f'{crate}-{source.name}')
payload=[output/name for name in ['Shadowrealm.exe','libunwind.dll','LLVM-LICENSE.txt','README.txt']]
payload.extend(path for path in licenses.iterdir() if path.is_file())
manifest={str(path.relative_to(output)):hashlib.sha256(path.read_bytes()).hexdigest() for path in payload}
(output/'SHA256.json').write_text(json.dumps(manifest,indent=2)+'\n')
archive=output.parent/'Shadowrealm-windows-x64-exploration.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
    for path in sorted(payload+[output/'SHA256.json']):
        z.write(path,str(path.relative_to(output.parent)))
print(f'Packaged {archive} ({archive.stat().st_size:,} bytes)')
print('ROM not bundled. Supply Shadowrun.bin beside Shadowrealm.exe.')
