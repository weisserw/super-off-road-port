"""Lossless local PPM/PNG capture conversion, exact pixel comparisons."""
import zlib,struct,hashlib
from pathlib import Path

def ppm(path):
 data=Path(path).read_bytes();head,raw=data.split(b'\n255\n',1);parts=head.split();assert parts[0]==b'P6';w,h=map(int,parts[1:3]);assert len(raw)==w*h*3;return w,h,raw

def png(path):
 w,h,data=ppm(path)
 def chunk(tag,content):return struct.pack('>I',len(content))+tag+content+struct.pack('>I',zlib.crc32(tag+content)&0xffffffff)
 out=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(b''.join(b'\0'+data[y*w*3:(y+1)*w*3] for y in range(h))))+chunk(b'IEND',b'')
 p=Path(str(path)+'.png');p.write_bytes(out);return p
if __name__=='__main__':
 import sys
 for p in sys.argv[1:]:print(png(p))
