"""Independent captured PCM summaries and spectral comparison, not expectations."""
import wave,struct,cmath,math,json,sys
from pathlib import Path

def read(path):
 with wave.open(str(path),'rb') as w:
  rate=w.getframerate();channels=w.getnchannels();v=struct.unpack('<'+'h'*(w.getnframes()*channels),w.readframes(w.getnframes()));return rate,[sum(v[i:i+channels])/channels/32768 for i in range(0,len(v),channels)]
def fft(x):
 n=len(x)
 if n==1:return x
 a=fft(x[::2]);b=fft(x[1::2]);b=[cmath.exp(-2j*math.pi*k/n)*z for k,z in enumerate(b)];return [a[k]+b[k]for k in range(n//2)]+[a[k]-b[k]for k in range(n//2)]
def spectrum(v):
 n=4096;out=[0.0]*(n//2)
 for offset in range(0,len(v)-n+1,n//2):
  f=fft([v[offset+i]*(.5-.5*math.cos(2*math.pi*i/(n-1))) for i in range(n)])
  for i in range(len(out)):out[i]+=abs(f[i])**2
 return out
def summary(path):
 rate,v=read(path);s=spectrum(v);peaks=sorted(range(4,len(s)),key=lambda i:s[i],reverse=True)[:12]
 return {'rate':rate,'frames':len(v),'rms':math.sqrt(sum(x*x for x in v)/len(v)),'peak':max(map(abs,v)),'spectral_peaks_hz':[round(i*rate/4096,2)for i in peaks]},s
if __name__=='__main__':
 out={};spectra=[]
 for path in sys.argv[1:]:out[path],s=summary(path);spectra.append(s)
 if len(spectra)==2:
  a,b=spectra;out['power_spectrum_cosine']=sum(x*y for x,y in zip(a,b))/math.sqrt(sum(x*x for x in a)*sum(x*x for x in b))
 print(json.dumps(out,indent=2))
