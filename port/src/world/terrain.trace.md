# Terrain independent source trace / contracts

Worker `/root/behavior_contracts/terrain`, frozen Round29
`reference/source/game.asm`, SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
The adjacent terrain.rs is pending domain interfaces and source-derived runnable
contracts, not a terrain implementation. Reviewed every terrain-owned region:
region-167/168/240/169/170/731/332/171/732/172/733/173/962 and terrain data unions
region-379/384/385. CPU bank00 code=physical+$8000; terrain data uses LoROM full
banked pointers, resolved by tracks loaders. No reference changes or QA occurred.

Platform references consulted: toolkit docs/development.md SNES reference table,
docs/snes.md, docs/snes-native.md, WDC W65C816S datasheet (2024-03-13)
https://www.westerndesigncenter.com/wdc/documentation/w65c816s.pdf (native M/X,
byte accumulator high B retention, CMP/ADC carry, XBA, SEP index truncation,
logical LSR). Local pinned platform reference
/root/romatlas/vendor/jgenesis/backend/snes-core/src/ppu/registers.rs
write_m7a/write_m7b/read_mpyl/read_mpym/read_mpyh confirms hardware mechanism:
signed16 A times signed8 B gives signed24 product; reading MPYL as a word gets
bits0..15, reading MPYM as a word gets bits8..23. No emulator execution or core
algorithm was used as a gameplay implementation or independent expected oracle.

## Neighborhood decode and ROM-backed windows

`load_terrain_cell_neighborhood`, game.asm:6743, physical$18EB..$192A:
SEP30, LDA0/XBA clears high accumulator B. Y high coordinate is doubled in BYTE
width, TAX in X8; X high coordinate is ASL/CLC/ADC in BYTE width. After REP20,
add the row-offset word at CPU$9F87+((2*y_hi)&255), set X16 and TAX. Thus byte
column sum is `(3*x_hi)&255`, not unrestricted column*3. Signedness of world
coordinates matters elsewhere, but indexing uses high bytes as unsigned bytes.
Table has44 supported row words, first0, next132, last5676. Natural supported
coordinates use44-column stride132. Synthetic wrap fixture x_hi=$56,y_hi=$80
addresses offset2 (not a fictitious huge grid index), proving width switching.

At index j the actual reads are a=j,b=j+3,center=j+1,c=j+$84,d=j+$87,
control=j+2. Boot clears word corner high bytes and byte$69, then neighborhood
reads overwrite only low endpoints; interpolation must keep endpoints unsigned
bytes and signed word DIFFERENCES, rather than sign-extending endpoint bytes.
`decode_terrain` accepts the exact source copied window$16B1 bytes; inclusive
last byte can be the next track's first height. Source descriptor-selected
windows start physical$20000,$216B0,$22D60,$24410,$28000,$296B0,$2AD60,$2C410,
$50000,$516AA,$52D5A,$5440A,$58000,$596B0,$5AD60,$5C410. First/last expectations
are independently inspected source storage metadata, not output learned from port.
Three owned data unions are overlapping shared windows, not independent track
allocations. All16 window contracts retain original bytes including final byte;
tracks owns bank conversion and native collection/manifest extension semantics.

Checked native neighborhood failure for incomplete data or unsupported last-row
probe is a native validation choice, not an original SNES error branch. Source
can read WRAM before/after established copied window; natural reachability and
initial contents there are not proved. `terrain-window-boundary-state` records
this residual question. Do not silently clamp/zero/pad it in implementation or
exclude a source handler; follow actual consumers if a real route reaches it.
Synthetic classification takes explicit adjacent controls so it makes no claim
about off-window storage. Future added tracks need valid supported probe bounds.

## Quadrant selection, marker history and blocking branches

`sample_terrain_height_or_block`, physical$174F..$1807, game.asm:6411..6559:
PHP preserves caller status; neighborhood sets M8/X16 on return. Clear an8-bit
quadrant accumulator; CMP rawY,rawX sets C=(Y>=X), ROL into scratch. CLC/ADC X
sets carry if rawX+rawY>=256; ROL again produces
q=2*(Y>=X)+(X+Y>=256). Loop DEC then LSR twice repeats q times. First LSR of
selected pair moves marker bit0 into C; ROR persistent terrain_control_shift
inserts it at bit7, retaining prior history>>1. Second LSR tests block bit1.
This shift occurs for heights AND blocked results. Tests cover all four q,
equality boundary, exactly256 sum, and both marker values. No input mutation or
render operation may consume this persistent history. `objects` reads high nibble
==F at game.asm:16483; after four descending wheel probes it encodes their four
markers. Marker semantics beyond this established consumer remain provisional.

Block-clear selected pair goes directly to interpolation with same caller status
on stack. Block-set inspects full original control&AA (marker bits ignored):
22→A0,0A→E0,88→20,A0→60; other non-AA masks→08. A0/E0/20/60 loads are INTERNAL
entries in BIT operand bytes at$1790/$1793/$1796/$1799. The normal BIT prefix
retains A08. `return_diagonal_block_and_probe_neighbor` is misleading as a label:
encoded$9790 is LDA#A0, not a new neighbor probe. All tails PLP then SEC; return
value is a direction code, not a terrain height. Contracts test exact encodings
without inventing compass meanings or relying on comments as an oracle.

For masked AA inspect neighboring controls in strict order:
read lower j+$86 (control of next row) maskAA=0→C0;
left j-1 open→00; upper j-$82 open→40; right j+5 open→80.
These are encoded long aliases$7F0086/$7EFFFF/$7EFF7E/$7F0005 plus X, so signed
relative offsets cross the WRAM bank boundary rather than wrap within copied data.
If none open, diagonal tests in order: left0A→E0,left88→20,rightA0→60,right22→A0,
upper88→20,upperA0→60,lower0A→E0,lower22→A0; otherwise08.
Cardinal tails similarly enter LDA inside BIT operands$17F9/$17FC/$17FF/$1802.
Follow-up runtime repair: neighbor reads now stop at the first cardinal opening,
matching these return branches. Eagerly reading all four caused a natural driven
first-race freeze at frame273 with seed123456789b: the upper neighbor preceded
the copied window even though an earlier opening already determined the result.
A second driven reproduction at frame338 genuinely needed the upper alias.
The preceding controls ($7EFF7E..$7EFFFF) have no established source write.
Native config::UNCOPIED_TERRAIN_CONTROL explicitly chooses zero for this
power-on area, matching the existing zero-memory reference baseline; physical
console power-on contents remain unproved. This supersedes rejecting those
reachable controls. Other incomplete neighborhood/asset reads remain checked.
Two edge regressions and the varied-seed Game/audio/save integration test cover
these repairs.
All eight neighbor pattern branches, four open branches and fallback are covered,
including a simultaneous-match fixture establishing priority.

## Reflection and signed triangle interpolation

`normalize_terrain_triangle_and_interpolate` physical$1807..$18EB,
game.asm:6567..6742. Raw fraction bytes are copied; negative (bit7) X reflects
via XOR FF, exchanging a↔b and c↔d. Negative Y reflects via XOR FF,
a↔c,b↔d. This is one's complement (FF→0,80→127), NOT abs/negate. Both reflected
fractions are0..127 and feed signed8 Mode7 B as nonnegative factors. Center is
unchanged. Compare y>=x selects alternate triangle including equality.

Let a,b,c refer to reflected unsigned height endpoints, m=center, fx/fy reflected
fractions. Horizontal y<x calculates signed word slope `(2*m-a-b)*fy`, reads
LOW16 product; adds LOW16 `(b-a)*fx`, then adds terrain_interpolation_base WORD.
The latter starts at DP$69 and overlaps a at$6A: boot leaves$69=0 and a's high
byte0, so this base is a<<8. Alternate y>=x calculates `(2*m-a-c)*fx+(c-a)*fy`
plus same base. Every ADC/SBC has explicit CLC/SEC, word overflow wraps;
combine products before taking result high byte so sub-byte carries survive.
Negative slopes must remain signed words, e.g a250,b0,m0,fx64,fy32 gives
64000-16000-8000=40000, high byte156. Endpoints>=128 are not negative heights.

After sum, SEP20/LDA track/CMP8 overwrites only LOW A, retaining computed high B;
XBA returns that high byte. Track8 alone then SBC#20 with SEC, wrapping byte
underflow. Both triangle tails PLP/CLC report height, not block. Example
[40,100,60,80],m80 gives horizontal(64,32)=57,alternate(32,64)=52,
diagonal(64,64)=60. Negative-X C0/32 gives87; negative-Y32/BF gives62;
bothC0/BF give80. Contracts include exact opposite endpoints, signed slope
extremes, fractional carry/truncation and track8 subtract/underflow in both paths.
No float interpolation or generic bilinear interpolation matches this source.

## Wheel geometry and scan cross-owner effects

`calculate_wheel_probe_position` physical$192A..$196A, game.asm:6776:
SEP30 truncates incoming Y to low byte; AND3 picks wheel phase60,A0,20,E0 from
physical$196A. Byte pose+phase wraps; Y component reads sine at that angle,
X reads same with an additional quarter-turn40. ASL sine byte captures sign in
carry; XBA/LDA0/(DEC if carry)/XBA sign-extends DOUBLED signed byte into word.
Then REP20/CLC/ADC world word wraps. At pose0 offsets for wheels0..3 are
[-182,+182],[-182,-182],[+182,+182],[+182,-182]. Sine +/-127 reaches +/-254;
negative world wrap and angle modulo256 are covered from actual ROM fixtures.
Only table fixture parsing is implemented in tests; no duplicate geometry oracle.

`sample_wheels_and_project_world_position` actual entry physical$1663..$169A
ONLY samples wheels; its name does not mean projection is called there.
SEP30 picks last wheel index (3,7,11,15) for the current truck, zeroes four flags,
then probes descending wheel index while scratch goes3..0. Returned carry-clear
height overwrites corresponding global wheel height. Carry-set adds1 to source
block code and writes per-truck block flag; prior sampled height is retained.
After all four, loads truck_surface_summary[current_truck_index>>1] into working
surface height. It DOES NOT average new heights here; physics owns subsequent
average/suspension stage. Native API receives logical truck slot to avoid leaking
assembly byte-index encodings. Contracts exercise actual geometry/field sampling
integration, partial blocked heights, unchanged other trucks, flags+1, descending
marker history, and prior summary rather than a mock-return oracle.

`initialize_start_grid` game.asm:1130 calls calculate_wheel_probe_position with
inherited working pose and Y low truncation (natural load/grid ends Y=$0600→00).
Grid initialization does not overwrite working pose with chosen grid heading.
Tracks' initial_surface_origin is therefore a BASE; terrain transforms to final
sample coordinates. Grid source return application remains tracks owner.
Motion's `update_four_trucks` calls terrain scan after proposed displacement,
then objects special surface, then accepted/blocked commit and later projection.
Do not move these calls or average into sampling; rejected proposal keeps source
sample/marker effects as traced by motion. Collision directly calls single probes
with own explicit origin and terrain result interpretation.

## World-to-screen projection

`project_working_world_position` physical$169A..$174F, game.asm:6305..6410,
called by truck commit/blocked path and object projection (not implicitly wheel
sampling). Source saves worldY; gets M7 products using signed word coordinates:
pyX=bits8..23(worldY*34), pxX=bits8..23(worldX*signed8 Xscale).
Add WORD with wrapping, logical shift right4, add Xoffset WORD with wrapping.
Do NOT combine full products before truncating: x1,y7,scale103 has two0 partial
products although full combined numerator341 would have high byte1.
Ypartial=bits8..23(worldY*signed8 Yscale)+Yoffset WORD. Height masks low byte,
shifts5 then signed16*64, reads bits8..23 =height*8. Compute WORD
(2C00-Ypartial-height*8), logical shift4. SEP20/STZ screenYhi then STA screenY
stores only LOW result byte. Native ScreenPosition deliberately types Y asu8.
Original scales<=104 but added byte values128..255 must preserve signed factors.

For x1000,y2000,scales103/67,height40,offsets0: X=(1648+1088)>>4=171,
Y=(11264-2144-320)>>4=550→38. OffsetsFFFF/+16 produce170/37. Signed
xFF00,yFF00 produces X=4087,Y196 because negative word sums use logical
shift, not signed right shift. Added-content signed scaleFF with x1000,y1000
produces X33,Y193. Zero,255height, sub-byte truncation, signed underflow,
word offset wrap and low-Y truncation are explicit source-derived expectations.

## Evidence and limits

All domain functions remain unimplemented stubs. The native suite compiles,
discovers these tests and executes their assertions/stubs; recorded failures
are expected pending implementation, not passing fidelity claims. Local ROM
fixtures load owned data without embedding original graphics/audio assets.
No emulator experiment, behavior implementation, QA, toolkit edit or audit.
Exact commands, outcomes and logs: checks/contracts/terrain-results.json.
