"""Convert native PPM frames to a shareable PNG and GIF (requires Pillow)."""
import argparse
from pathlib import Path
from PIL import Image
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('directory',type=Path)
a=p.parse_args()
paths=sorted(a.directory.glob('frame-*.ppm'))
if not paths:p.error('directory has no frame-*.ppm files')
frames=[]
for path in paths:
    with Image.open(path) as frame:frames.append(frame.convert('RGB'))
frames[0].save(a.directory/'scene.png')
frames[0].save(a.directory/'animation.gif',save_all=True,append_images=frames[1:],duration=17,loop=0)
print(f'Converted {len(frames)} native frames; {len({frame.tobytes() for frame in frames})} distinct images')
