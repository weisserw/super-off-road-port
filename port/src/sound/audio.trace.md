# Audio independent contract trace

Worker: `/root/behavior_contracts/audio`, Stage3 only. Implementation is pending.
Primary revision: frozen Round29 `reference/source/game.asm`, SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
`reference/reading.asm` and `reference/source/symbols.asm` provide navigation;
comments and ledger summaries were checked against instructions and local ROM.
No reference, original ledger identity/range, toolkit or QA record was changed.

The main CPU routines use DB/DP zero and explicit M widths. The uploaded driver
uses SPC700 direct page zero, byte arithmetic, little-endian word pairs and
SPC carry/borrow rules. Its code is translated into native sequence/voice rules,
not retained as executable instructions. Immutable DataAddress identifies a
resource view, not a mutable emulated RAM map.

Hardware reference consulted: [Fullsnes original specifications](https://problemkaputt.de/fullsnes.htm),
SPC ALU and division-overflow sections, APU timing and DSP sections. MUL updates
flags from its high product byte; ADDW flags describe a word result. SBC subtracts
an extra one when carry is clear. Timer0 clocks at8kHz, the DSP at32kHz. For the
specific YA=0,X=0 division the documented hardware result is A=FF. These facts
are used in fixed fixtures, without implementing an instruction oracle.
Toolkit documentation inspected: `/root/romatlas/docs/development.md` platform
references and `docs/snes-native.md`, plus the independent native audio API.

## Source ownership and native boundaries

* definitions.rs: upload metadata, immutable data views, parallel pointer rows,
  effective pitch/instrument reads, seven-byte envelopes, sample directory/BRR.
* sequencer.rs: all34 command variants and counts, sequence/call/loop state,
  song replacement, eight voices and update/DSP-write ordering.
* voice.rs: actual note/duration/key masks, target/current pitch and envelopes,
  modulation, portamento, auxiliary stream and envelope stages.
* host.rs: main human mask, host command/ack/heartbeat, reserved-voice priority,
  partial effect reset, literal master-fade arithmetic and delayed update.
* synthesis.rs: driver plus native mixer, predictor/envelopes/noise, sample clock,
  resampling and complete clone/snapshot/observation state.

Every function body receiving source behavior is explicitly unimplemented.
Constructors used only to declare native fixtures/default empty state implement
no driver behavior. Tests call actual domain interfaces and assert mutated state
or pixels-independent PCM, not a second implementation of the source algorithm.
Existing game/frontends still render the skeleton. Stage4 must integrate Sound
with Port's existing Game audio, tick/reset/clone/full snapshots and observations;
passing an isolated sound test is not that integration.

## Host/main-CPU upload and sound gate

`send_race_sound_if_human`, physical0256..0265, saves the full command, tests
active_human_mask AND00FF and either writes command word APUIO2/3 or discards it.
An upper-only mask0100 is silent; lowbit80 is nonzero and permits the same command.
The source does not test the requesting truck's human bit. Race music at017E
uses active_human_mask (the symbol HIGHSCORE_DIGITS is a DP06 alias), and sends
selector `(order&7)+6`; SPC command1 decrements this, selecting songs5..12.

`upload_audio_block_streams`, physical005FFE..00609C, writes reload55 to port1,
then transfers streams from CPU0E8000 and0F8000. Each record is lengthLE16 and
destinationLE16, followed by length bytes. It waits for IPL BBAA, sends CC for
the first header, uses an incrementing byte token for payload acknowledgements,
and distinguishes nonempty versus zero-length records using the overflow flag
from the header ADC7F. A zero record supplies the execution entry and clears all
four ports after acknowledgement. This hardware-only protocol is replaced by
validated immutable resource loading; it is not native CPU execution.

Original records independently read from ROM:

| Physical stream | Payload | Destination | Terminator | Consumed |
| --- | --- | --- | --- | --- |
|070000|7E00|0400|length0,entry0400|7E08|
|078000|7E00|8200|length0,entry0400|7E08|

Thus ROM070004..077E04 maps to immutable uploaded0400..8200 and
078004..07FE04 maps to8200..10000. Physical077E04..078004 is not sample data.
The bytes after each zero terminator are outside its upload. Bounds/truncation,
16-bit destination overflow and exact consumed length have independent fixtures.
No payload header or bank-tail byte is silently inserted into a sample.

## Immutable tables and data consumers

The source's eight low/high sequence-pointer row pairs start0E20/0E40 and advance
40 per voice. Each row allocates32 columns, but the original songs are16; spare
zero columns do not imply extra playable songs. Song0 pointers all1567, whose
80 token deactivates each voice. Song1 voice pointers are independently read as
1568,15BB,164A,1670,16A8,1755,17A5,1807. Song15 first pointer40B1 is not silent.
Config pointer rows1020/1040 index **song-1**, not song. Song1 config1154;
song15 config1536. The loader copies Y=8C down through0, exactly141 bytes.

Initial envelope records:28 rows of7 bytes at1090..1154. The copy destination1097
is row1, so rows1..20 are entirely replaced and row21's first byte (timer reload)
is replaced by byte140. Row0 and rows22..27 persist. The clear helper's eleven duration stores80,82,..94 also initialize the first
three current-envelope cells90/92/94 to1, unlike the other five current cells0.
This is an observable domain initialization effect, not a reason for a RAM map.
Reloading a song does not
restore immutable envelope defaults: load initialized resources once, retain the
live table and overwrite only these141 bytes. The contract seeds a live table
explicitly to distinguish persistent row0 from accidental ROM reinitialization.

Song0's byte index decrements toFF, so its config pointer is assembled from
111F and113F, not the nominal row's last column. Initial values give0100, outside
the uploaded data region. Those pointer cells can themselves change after song
configuration installs. No arbitrary low ARAM values are declared original facts.
All eight song0 sequences immediately stop before consuming configuration, so a
source-informed native silent-reset policy may preserve existing live rows rather
than fabricate low-ARAM bytes. Effects following this silence still must initialize
all fields before dependence, or record a residual assumption. This is not proof
that all conditional consumers of low-ARAM configuration are irrelevant.

Effect rows1060/1070/1080 have16 entries;6 aliases5, both438C. Priorities7=3 and8=4.
No encoded selector guard exists; native file content must validate its supported
selectors, not reinterpret bytes beyond these arrays as additional named effects.
Instrument arrays5084 and50A4 allocate32 bytes. Modular instrument33 reads note
base50A5 (scale-array byte1) and scale50C5 (silence payload); it must not be clamped
to31. Effective immutable read views preserve these reads without adding fictional
instrument records. Pitch low/high bases0CED/0D42 use modular byte note indices,
including0,249,254; note0 reads80/68. The final voice-mask complement7F overlaps
one pitch byte; resource role overlap does not duplicate or alter storage.

The directory at5000 has33 start/loop word pairs. Decode nine-byte BRR blocks to
first END, then validate any active loop target's nine-byte alignment and chain
membership. Directory32 is a single03 silence block at50C4. Sample27 has2268
bytes and terminalB1 (END, no LOOP). Aliased starts remain aliases, not invented
samples. Sample20 starts7944 and spans3663 bytes through8793; its split at8200
occurs within the physical sample chain and may cross a BRR block. Its fixed
18-byte split vector is2E23FFB41FE12F00DF44ED01A0FFF033CA13, read independently
across the upload mapping. Every directory sample is loaded from these consumers.

DSP initialization: register list0CAC has25 values and negativeFF sentinel;
values0CC6. First nine pairs are6C/E0,5C/FF,2D/0,3D/0,4D/0,6D/3,7D/0,0D/0,5D/50.
DIR50 selects directory5000; echo delay0, feedback0, ADSR1 registers05..75 zero.
The source uses per-voice direct GAIN written to register7, not ADSR envelopes
invented from an instrument name. Native mixer API limitations are below.

## Host polling and reserved effects

Entry0400 initializes stack, silences master/echo, installs DSP constants, loads
song0, does a driver update and starts timer0. The steady poll checks flag08
**before** reload55, command and heartbeat processing. Flag08 therefore requests
a direct update without acknowledging the command. Flag55 writes DSP E0 and
returns to IPL. Command1 decrements selector then clears/loads song and sets
master7F00, FLG33; selected-song shadow is inside the cleared DP range, while a
native requested-song ID may separately identify the audible track. Command2
arbitrates an effect then acknowledges even if rejected. Command3 sign-extends
argument and shifts three times:FF→FFF8,80→FC00. Unknown commands are unacked.

Changed heartbeat: retain input value, restart timer0(divisor42), perform one
immediate update, output a bounded diagnostic DP dump, wait for nonzero timer,
then update a second time. Same heartbeat alone does nothing. Host input/output
latches are separate; the status dump does not overwrite the host's input flags.
Native scheduling replaces SPC polling/instruction overhead by an exact264 DSP
sample interval for the second update (66 ticks at8kHz), preserving ordering and
pending timer across snapshots. Source instruction timing/fast-mode throughput
is not claimed cycle-exact; see uncertainty record. No busy loop blocks the host.

Effect arbitration uses voice6 then7: first inactive6 wins regardless of priorities;
else inactive7; else incoming>=priority6 wins6, else>=priority7 wins7, otherwise
reject. It does not simply select the quieter/lower-priority voice. Equality
replaces. On accept, install sequence and priority, clear noise bit6/7 only,
clear target step/slew interval/suppress-key-on/mod-direction-reload/portamento/
secondary-mode/transpose/pitch-low-offset/pitch-modulation/fixed-duration/release
threshold/delta bytes, duration1, activeFF. Instrument, volume, current AND target
pitch and unlisted controls persist until sequence setup. Stack cursor is reset
to the source slot's initial range; a native fresh frame list represents that.
Source effect prefixes initialize instrument before a nonzero interval-zero note.
Effects5/6 install AC before that note, but the note calculation establishes
current/target envelope before tick helpers. Fixed integration fixtures cover1/5/6.

## Master fade and driver update order

At0546, fade integration precedes the optional slow divider. **Source correction:**
positive-step ADDW branches BMI to store; otherwise sets high7F and stops the
step, keeping the computed LOW byte. Thus1000+0008→7F08 with step0, while
7FF8+0008→8000 with step8. Do not replace this literal branch by a plausible
saturating signed volume algorithm. Negative path stores nonnegative sums but
clamps negative sums' high byte to0 and stops:0000+FFF8→00F8, not0000.
These flag-derived tests override the ledger's generic clamp description.

If flag04, DEC slow_counter; return unless zero, then reload20. Zero initially
wrapsFF, not an immediate tick. Fade still advanced on skipped calls. Pending
key masks/voice duration remain untouched during a skipped call.

Full update writes KON(old pending), clears pending-on, writes noise mask, then
voices0..7 in order. Each voice may produce a new pending-on/off. It writes
KOFF(new pending), clears pending-off, emits six DSP registers per voice in order
volumeL,volumeR,pitchLO,pitchHI,SRCN,GAIN (register7, not5/6), then masterL/R and
KOFF0. New note KON is therefore committed on the next successful driver update,
not immediately when the parser encounters the note. Source stop80 merely
marks inactive; it does not add key-off or erase pitch/gain. Actual native mixer
commits must preserve these ordered events, not only a final event label list.

## Sequence language, call/loop state and commands

Byte reader uses sequence pointer then increments low/high with carry, preserving
fetched-byte flags by OR0. Nonnegative tokens are notes/rest; negative even tokens
80..C2 dispatch34 little-endian vectors at0A10. Odd/out-of-range synthetic content
is rejected by native validation; original dispatch has no bounds guard. Native
call/loop frames replace the packed per-voice stack while preserving balance and
return pointer after operands. A counted-loop frame holds start+count; repeat
DEC count, rewind if nonzero, otherwise drop frame. Count0 wraps255 and eventually
executes256 iterations, not zero. Reject malformed/balanceless file content rather
than reading arbitrary host memory; bounded parsing rejects a zero-time jump loop.

|Token|Operands|Actual operation|
|---|---:|---|
|80|0|deactivate, return without note helpers|
|82|2|jump LE pointer and resume parser|
|84|2|push after-operands pointer and call|
|86|0|pop pointer and resume|
|88|1|push counted loop at current pointer|
|8A|0|decrement/rewind/drop loop|
|8C|1|fixed duration; zero restores note operand|
|8E|0|force exactly next duration operand; clear when read|
|90|1|transpose byte|
|92|1|instrument byte|
|94/96|1|left/right volume|
|98|3|delay,aux pointerLE; clear amp/cursor/primary/enable, set secondary|
|9A|1|additive low pitch offset|
|9C/9E|3|direction+1/-1, delay,step,period; period copied live|
|A0|1|portamento step|
|A2|0|disable reload direction, retain live direction until reset|
|A4/A6|1|release delta/fixed; clear opposite field|
|A8|2|slew step then interval|
|AA|0|zero slew interval|
|AC|3|target step,interval,delay; delay copied live|
|AE|1|record byte*7 low-byte; primary mode on,secondary/enable off|
|B0/B2|0|noise bit OR/AND inverse|
|B4|1|FLG=(operand&1F)OR20|
|B6|0|primary on,secondary+enable off|
|B8|0|enableFF only, retain modes|
|BA|0|secondary on,primary+enable off, retain stream pointer|
|BC/BE|0|allow/suppress new key-on|
|C0|0|one-shot transpose skip bit1|
|C2|0|reload song1, regardless selected-song shadow, master7F00|

**B6/B8 correction:** vectorB6 is0BCE (`spc_command_enable_auxiliary_mode`), vectorB8
is0BC6 (`spc_command_reset_primary_auxiliary_mode`). Nearby source label order is
opposite token order. Contracts follow vectors, not label prose or old inventory.
All families have executable mutation/parser contracts, not only operand counts.
C2 clears/replaces all voice state through song load, then resumes current slot's
new stream; it is not a no-op or an arbitrary requested-song reload.

## Note, pitch, modulation and auxiliary behavior

Inactive voices return without decrement. Active duration DEC; zero wrapsFF.
On expiry parse commands until note/rest or stop. A nonzero note adds instrument
base then optional transpose in byte width. LSR one-shot skip supplies carry;
only odd bit consumes skip. With zero slew interval, current note-envelope is
initialized and pitch calculated immediately, BEFORE this tick's target stepping.
A rest skips all note initialization/key masks/skip shift but still reads duration
and establishes release threshold. Fixed duration suppresses operand except one-shot
force8E; force clears after its read. Relative release delta subtracts with SETC,
wrapping byte; delta0 selects fixed threshold.

Pitch: split lookup is a16-bit base P. MUL low and scale, retain high byte; MUL
high and scale, add retained byte, then add original P with word wrap. This is
P+floor(P*scale/256), not a floating musical frequency approximation. Fixed vectors
1280 scale80→1BC0,FFFF scale01→00FE. Transposed note index wraps modulo256.
New notes reset modulation delay/direction, period=reload>>1 and offset0.
Primary/secondary mode enable gates only their respective resets; helpers continue
according to existing live state, so reset conditions are not generic note stops.

Target-step timer first restores changed note-origin and reloads target delay;
otherwise adds step byte with wrap and reloads interval. Timer0 underflowsFF.
Current slew delay0 snaps/recalculates; delay nonzero decrements/reloads then
moves by step and clamps only using unsigned post-add/subtract compare. Conditional
underflow2-5→FD remainsFD if target1, not a saturating1. Target step runs BEFORE slew.

Modulation delay1 becomes0 and returns; motion begins following tick. Positive
adds step with carry; negative two's-complements the byte then adds FF high. For
negative direction and step0, low unchanged but high decrements (1234→1134).
Period decrements after motion; nonzero reload reverses signed direction, zero
reload retains it. Portamento0 snaps target. Otherwise compare unsigned word,
step toward target and snap crossing, preserving source byte borrow/carry.
Final DSP pitch keeps modulation-low carry separately from low-offset carry:
12FF+0001+FF→13FF,12FF+0002+FF→1400.

Auxiliary processing only in secondary mode. Delay0 does NOT fetch; delay expiry
reads at byte cursor. Ordinary bytes update amplitude;80 resets cursor0 and rereads;
other negatives decrement cursor (hold marker). Reload delay after a token.
At duration==release_threshold, scan from stream start to first negative marker
and set cursor just after it, retaining byte arithmetic. This release scan still
occurs when delay0. Native bounds validation must reject absent markers or a
zero-time restart with no ordinary token; no infinite host loop.

## Seven-byte envelope stages and arithmetic corrections

Rows are reload,start,rise,peak,fall,sustain,release. Stages are rise/fall/sustain/
release/done via separate low/high vectors0889/088E; stage2 and4 are RET bodies.
At nonzero duration==threshold set release stage, save amplitude, tick0, THEN
DEC stage timer and dispatch on zero. Reload row timer on dispatch. Duration0
never reinitializes release. Stage timer0 underflowsFF.

Rise: delta=peak-start byte, multiply by tick, divide by rise-1, add start byte,
store amplitude, tick++; when tick==rise advance stage and tick0. **Correction:**
rise duration0 returns WITHOUT advancing stage or resetting tick. Rise duration1
at tick0 invokes hardware divide0; its A=FF gives start10→amplitude9, then advances.
Normal row start10,rise3,peak30 yields amplitudes10,20,30 then fall.

Fall/release execute CLRC then SBC tick, so remaining=duration-tick-1, not
`duration-tick`. Fall computes sustain+floor((peak-sustain)*remaining/duration).
When remaining0 it sets sustain, advances stage and resets tick. With peak30,
sustain6,duration3, amplitudes22,14,6 then sustain. Release similarly multiplies
saved amplitude and remaining/duration;30,duration3 gives20,10,0. Release finish
advances to done but DOES NOT reset tick (stays2 in this fixture). Zero duration
fall/release take balanced stack exits with unchanged state; do not invent a
stage advance from the generic ledger summary. Normal quotient vectors are
independent hand arithmetic, not calls to native implementation outputs.

## Native synthesis fidelity and state contracts

Original sample data remains BRR; shared independent native helper decodes it.
Synthetic headers13 with nibbles17/8F yield2,14,-16,-2. A filter1/range2 block27
with nibbles11 yields4,6,8,10 and retained predictor gives34 at next block start,
not a reset4. Loop targets retain predictor history; terminal nonloop stops.

A synthetic constant4096 BRR signal, direct gain127, volume127, master127 gives
4000 PCM by the helper's documented fixed-point stages. Right volume80 is signed
-128 and yields-4033. These fixtures specify the selected native synthesis helper,
not an assertion of Gaussian/hardware sample-exactness. Gain key-off releases,
retrigger restarts sample, mute advances clock, noise selects LFSR and softreset
releases. The source's ADSR1 zero selects direct GAIN; do not add an unrelated ADSR
instrument model. Echo is disabled in source initialization; retained echo helper
state must still snapshot if enabled by a supported extension.

Toolkit native DSP documents linear rather than Gaussian interpolation, no cycle
exact key-on/register races/pitch-modulation bus/echo-ARAM overlap. No device or
original-runtime waveform experiment was executed. Preserve these as explicit
implementation/QA limits rather than claiming hardware-equivalent synthesis.
Game modulation is implemented in driver pitch, not substituted by unavailable DSP
pitch modulation. Required native output supports frontend rates by explicit
resampling; phase/chunking must be deterministic (441 frames at44.1k consumes320
native frames) and share the same Sound owner on desktop/browser/headless.

Snapshot contracts start active playback and a pending delayed heartbeat, advance
fractional output, clone/restore, compare subsequent nonzero samples and full state,
then reject malformed restore atomically. All mutable predictor/envelopes/noise,
source position, native/host clocks, pending keys/commands, sequence stack/cursors,
141-byte live envelope overlap and resampler phase must fork. Immutable definitions
alone may share Arc. Observation is read-only and does not drain audio. Toolkit
Mixer currently exposes Clone but no public complete serialization; stage4 may
need a reusable serializer contract/library change with required toolkit logging.
No shared-library change is silently bundled into this contract stage.

## Unknowns, consumer limits and preserved decisions

Original unknown070149 contains6F (RET) after effect initializer's established RET,
before separately called0546. Original unknown0704FF also6F between completed fall
body and separately entered08FC release tails. Direct calls/jumps and34 command/
5 envelope vectors inspected establish no entry at either isolated byte. Their
bytes are in the uploaded resource view but preserving them is not native logic.
Unknown identities/ranges remain unchanged and classification remains unknown;
implementation should use these consumer facts plus a documented reasonable
assessment, not demand complete disassembly or invent natural execution.

Patterned tail077E08..078000 and07FE08..080000 lie beyond zero upload terminators;
no SPC view is assigned. Uploaded patterned07FA2B..07FE04 follows all established
first-END sample chains (last endsFC27), remains immutable source data with no
established sequence/sample consumer. Do not relabel it padding or silently append
it to the last sample. These observations justify review leads, not ledger exclusions.
All717 primary ranges are indexed below with original labels/source context.
Overlap and live-copy consumers above define semantics; the index alone is not
proof that preserving bytes implements a song or driver command.

## Execution record and handoff

101 substantive audio tests are native-discovered. Rust1.90.0 locked compile/list
pass (610 total suite tests). Audio run:0 pass/101 explicit pending-unimplemented
panics,0 ignored. All four existing infrastructure tests pass separately.
`checks/contracts/audio-results.json` records exact commands/logs. Initial compile
log retained. No implementation, emulator/device/QA experiment or final audit ran.

Stage4 must implement all pending native functions and actual Game composition,
read source again, retain shared session audio_commands through ordered command
submission, and snapshot the complete Sound owner. Final sound checks and original
initial audio comparison belong to their authorized later stages. Uncertain timing,
song0 configuration and native synthesis limitations remain explicit assumptions,
not excuses to omit song/effect/voice/command behavior.

## Exhaustive primary source navigation (717 ranges)

Each row retains original ledger ID, physical half-open span, original unit labels
and source links. Source/reference and native behavior consumers are described
above; no disposition or source identity is changed by this appendix.

|ID|Physical range|Source units|Source links|
|---|---|---|---|
|region-115|000256..000263|send_race_sound_if_human|game.asm:376|
|region-222|000263..000265|discard_attract_sound_command|game.asm:393|
|region-26|005FFE..00609C|upload_audio_block_streams|game.asm:18362|
|region-88|070000..070004|spc_upload_block1_header|game.asm:20250|
|region-78|070004..0700EB|spc_initialize_and_poll_host|game.asm:20262|
|region-79|0700EB..070149|spc_start_priority_effect_voice|game.asm:20388|
|unknown-070149|070149..07014A|Unknown|game.asm:20434|
|region-80|07014A..070230|spc_update_master_fade_and_eight_voices|game.asm:20446|
|region-81|070230..070304|spc_load_song_and_process_voice|game.asm:20574|
|region-82|070304..0703E9|spc_pitch_envelope_and_modulation_helpers|game.asm:20702|
|region-83|0703E9..07048D|spc_portamento_and_sequence_envelopes|game.asm:20832|
|region-439|07048D..070497|spc_envelope_dispatch_bytes|game.asm:20929|
|region-802|070497..0704FF|interpolate_spc_envelope_stages|game.asm:20939|
|unknown-0704ff|0704FF..070500|Unknown|game.asm:21012|
|region-803|070500..070531|finish_spc_envelope_release|game.asm:21022|
|region-84|070531..07060B|spc_start_note_and_initialize_voice_effects|game.asm:21081|
|region-85|07060B..070614|spc_advance_sequence_pointer|game.asm:21206|
|region-440|070614..070616|spc_command_80_vector|game.asm:21219|
|region-441|070616..070618|spc_command_82_vector|game.asm:21244|
|region-442|070618..07061A|spc_command_84_vector|game.asm:21269|
|region-443|07061A..07061C|spc_command_86_vector|game.asm:21294|
|region-444|07061C..07061E|spc_command_88_vector|game.asm:21319|
|region-445|07061E..070620|spc_command_8a_vector|game.asm:21344|
|region-446|070620..070622|spc_command_8c_vector|game.asm:21369|
|region-1348|070622..070624|spc_command_8e_vector|game.asm:21392|
|region-447|070624..070626|spc_command_90_vector|game.asm:21406|
|region-448|070626..070628|spc_command_92_vector|game.asm:21431|
|region-449|070628..07062A|spc_command_94_vector|game.asm:21456|
|region-450|07062A..07062C|spc_command_96_vector|game.asm:21481|
|region-1013|07062C..07062E|spc_auxiliary_stream_command_vector|game.asm:21505|
|region-451|07062E..070630|spc_command_9a_vector|game.asm:21527|
|region-452|070630..070632|spc_command_9c_vector|game.asm:21552|
|region-453|070632..070634|spc_command_9e_vector|game.asm:21577|
|region-454|070634..070636|spc_command_a0_vector|game.asm:21602|
|region-455|070636..070638|spc_command_a2_vector|game.asm:21627|
|region-456|070638..07063A|spc_command_a4_vector|game.asm:21652|
|region-457|07063A..07063C|spc_command_a6_vector|game.asm:21677|
|region-458|07063C..07063E|spc_command_a8_vector|game.asm:21702|
|region-459|07063E..070640|spc_command_aa_vector|game.asm:21727|
|region-856|070640..070642|spc_target_envelope_command_vector|game.asm:21752|
|region-460|070642..070644|spc_command_ae_vector|game.asm:21776|
|region-461|070644..070646|spc_command_b0_vector|game.asm:21801|
|region-462|070646..070648|spc_command_b2_vector|game.asm:21826|
|region-463|070648..07064A|spc_command_b4_vector|game.asm:21851|
|region-464|07064A..07064C|spc_command_b6_vector|game.asm:21876|
|region-465|07064C..07064E|spc_command_b8_vector|game.asm:21901|
|region-1349|07064E..070650|spc_command_ba_vector|game.asm:21925|
|region-466|070650..070652|spc_command_bc_vector|game.asm:21939|
|region-467|070652..070654|spc_command_be_vector|game.asm:21964|
|region-468|070654..070656|spc_command_c0_vector|game.asm:21989|
|region-469|070656..070658|spc_command_c2_vector|game.asm:22014|
|region-804|070658..070823|spc_sequence_command_handlers, spc_command_set_target_envelope, spc_command_set_auxiliary_stream|game.asm:22042|
|region-86|070823..070832|spc_read_auxiliary_stream_byte|game.asm:22307|
|region-805|070832..07085C|spc_sequence_stack_helpers|game.asm:22325|
|region-87|07085C..0708B0|spc_write_voice_dsp_and_initialize_timer|game.asm:22363|
|region-266|0708B0..0708CA|spc_initial_dsp_registers|game.asm:22420|
|region-267|0708CA..0708E3|spc_initial_dsp_values|game.asm:22428|
|region-1014|0708E3..0708F2|spc_voice_bit_complement_pairs|game.asm:22436|
|region-1015|0708F2..0708F3|spc_pitch_selected_byte_000|game.asm:22459|
|region-1016|0708F3..0708F4|spc_pitch_selected_byte_001|game.asm:22482|
|region-1017|0708F4..0708F5|spc_pitch_selected_byte_002|game.asm:22505|
|region-1018|0708F5..0708F6|spc_pitch_selected_byte_003|game.asm:22528|
|region-1019|0708F6..0708F7|spc_pitch_selected_byte_004|game.asm:22551|
|region-1246|0708F7..0708F8|spc_pitch_low_index_06_cell|game.asm:22573|
|region-1020|0708F8..0708F9|spc_pitch_selected_byte_006|game.asm:22594|
|region-1247|0708F9..0708FA|spc_pitch_low_index_08_cell|game.asm:22616|
|region-1021|0708FA..0708FB|spc_pitch_selected_byte_008|game.asm:22637|
|region-1248|0708FB..0708FC|spc_pitch_low_index_10_cell|game.asm:22659|
|region-1022|0708FC..0708FD|spc_pitch_selected_byte_010|game.asm:22680|
|region-1023|0708FD..0708FE|spc_pitch_selected_byte_011|game.asm:22703|
|region-1389|0708FE..0708FF|r12_pitch_low_index_0d_0708fe|game.asm:22719|
|region-1249|0708FF..070900|spc_pitch_low_index_14_cell|game.asm:22727|
|region-1024|070900..070901|spc_pitch_selected_byte_014|game.asm:22748|
|region-1250|070901..070902|spc_pitch_low_index_16_cell|game.asm:22770|
|region-1391|070902..070903|r12_pitch_low_index_11_070902|game.asm:22784|
|region-1251|070903..070904|spc_pitch_low_index_18_cell|game.asm:22792|
|region-1393|070904..070905|r12_pitch_low_index_13_070904|game.asm:22806|
|region-1252|070905..070906|spc_pitch_low_index_20_cell|game.asm:22814|
|region-1395|070906..070907|r12_pitch_low_index_15_070906|game.asm:22828|
|region-1253|070907..070908|spc_pitch_low_index_22_cell|game.asm:22836|
|region-1397|070908..070909|r12_pitch_low_index_17_070908|game.asm:22850|
|region-1025|070909..07090A|spc_pitch_selected_byte_023|game.asm:22858|
|region-1254|07090A..07090B|spc_pitch_low_index_25_cell|game.asm:22880|
|region-1026|07090B..07090C|spc_pitch_selected_byte_025|game.asm:22901|
|region-1399|07090C..07090D|r12_pitch_low_index_1b_07090c|game.asm:22916|
|region-1027|07090D..07090E|spc_pitch_selected_byte_027|game.asm:22924|
|region-1028|07090E..07090F|spc_pitch_selected_byte_028|game.asm:22947|
|region-1255|07090F..070910|spc_pitch_low_index_30_cell|game.asm:22969|
|region-1029|070910..070911|spc_pitch_selected_byte_030|game.asm:22990|
|region-1030|070911..070912|spc_pitch_selected_byte_031|game.asm:23013|
|region-1031|070912..070913|spc_pitch_selected_byte_032|game.asm:23036|
|region-1032|070913..070914|spc_pitch_selected_byte_033|game.asm:23059|
|region-1033|070914..070915|spc_pitch_selected_byte_034|game.asm:23082|
|region-1034|070915..070916|spc_pitch_selected_byte_035|game.asm:23105|
|region-1035|070916..070917|spc_pitch_selected_byte_036|game.asm:23128|
|region-1036|070917..070918|spc_pitch_selected_byte_037|game.asm:23151|
|region-1037|070918..070919|spc_pitch_selected_byte_038|game.asm:23174|
|region-1038|070919..07091A|spc_pitch_selected_byte_039|game.asm:23197|
|region-1039|07091A..07091B|spc_pitch_selected_byte_040|game.asm:23220|
|region-1040|07091B..07091C|spc_pitch_selected_byte_041|game.asm:23243|
|region-1041|07091C..07091D|spc_pitch_selected_byte_042|game.asm:23266|
|region-1042|07091D..07091E|spc_pitch_selected_byte_043|game.asm:23289|
|region-1043|07091E..07091F|spc_pitch_selected_byte_044|game.asm:23312|
|region-1044|07091F..070920|spc_pitch_selected_byte_045|game.asm:23335|
|region-1045|070920..070921|spc_pitch_selected_byte_046|game.asm:23358|
|region-1046|070921..070922|spc_pitch_selected_byte_047|game.asm:23381|
|region-1047|070922..070923|spc_pitch_selected_byte_048|game.asm:23404|
|region-1048|070923..070924|spc_pitch_selected_byte_049|game.asm:23427|
|region-1049|070924..070925|spc_pitch_selected_byte_050|game.asm:23450|
|region-1050|070925..070926|spc_pitch_selected_byte_051|game.asm:23473|
|region-1051|070926..070927|spc_pitch_selected_byte_052|game.asm:23496|
|region-1052|070927..070928|spc_pitch_selected_byte_053|game.asm:23519|
|region-1053|070928..070929|spc_pitch_selected_byte_054|game.asm:23542|
|region-1054|070929..07092A|spc_pitch_selected_byte_055|game.asm:23565|
|region-1055|07092A..07092B|spc_pitch_selected_byte_056|game.asm:23588|
|region-1056|07092B..07092C|spc_pitch_selected_byte_057|game.asm:23611|
|region-1057|07092C..07092D|spc_pitch_selected_byte_058|game.asm:23634|
|region-1058|07092D..07092E|spc_pitch_selected_byte_059|game.asm:23657|
|region-1059|07092E..07092F|spc_pitch_selected_byte_060|game.asm:23680|
|region-1060|07092F..070930|spc_pitch_selected_byte_061|game.asm:23703|
|region-1061|070930..070931|spc_pitch_selected_byte_062|game.asm:23726|
|region-1062|070931..070932|spc_pitch_selected_byte_063|game.asm:23749|
|region-1063|070932..070933|spc_pitch_selected_byte_064|game.asm:23772|
|region-1064|070933..070934|spc_pitch_selected_byte_065|game.asm:23795|
|region-1065|070934..070935|spc_pitch_selected_byte_066|game.asm:23818|
|region-1066|070935..070936|spc_pitch_selected_byte_067|game.asm:23841|
|region-1067|070936..070937|spc_pitch_selected_byte_068|game.asm:23864|
|region-1068|070937..070938|spc_pitch_selected_byte_069|game.asm:23887|
|region-1069|070938..070939|spc_pitch_selected_byte_070|game.asm:23910|
|region-1070|070939..07093A|spc_pitch_selected_byte_071|game.asm:23933|
|region-1071|07093A..07093B|spc_pitch_selected_byte_072|game.asm:23956|
|region-1072|07093B..07093C|spc_pitch_selected_byte_073|game.asm:23979|
|region-1073|07093C..07093D|spc_pitch_selected_byte_074|game.asm:24002|
|region-1074|07093D..07093E|spc_pitch_selected_byte_075|game.asm:24025|
|region-1075|07093E..07093F|spc_pitch_selected_byte_076|game.asm:24048|
|region-1076|07093F..070940|spc_pitch_selected_byte_077|game.asm:24071|
|region-1077|070940..070941|spc_pitch_selected_byte_078|game.asm:24094|
|region-1078|070941..070942|spc_pitch_selected_byte_079|game.asm:24117|
|region-1079|070942..070943|spc_pitch_selected_byte_080|game.asm:24140|
|region-1401|070943..070944|r12_pitch_low_index_52_070943|game.asm:24155|
|region-1403|070944..070945|r12_pitch_low_index_53_070944|game.asm:24162|
|region-1405|070945..070946|r12_pitch_low_index_54_070945|game.asm:24169|
|region-1256|070946..070947|spc_pitch_high_index_00_cell|game.asm:24177|
|region-1080|070947..070948|spc_pitch_selected_byte_085|game.asm:24198|
|region-1081|070948..070949|spc_pitch_selected_byte_086|game.asm:24221|
|region-1082|070949..07094A|spc_pitch_selected_byte_087|game.asm:24244|
|region-1083|07094A..07094B|spc_pitch_selected_byte_088|game.asm:24267|
|region-1084|07094B..07094C|spc_pitch_selected_byte_089|game.asm:24290|
|region-1257|07094C..07094D|spc_pitch_high_index_06_cell|game.asm:24312|
|region-1085|07094D..07094E|spc_pitch_selected_byte_091|game.asm:24333|
|region-1258|07094E..07094F|spc_pitch_high_index_08_cell|game.asm:24355|
|region-1086|07094F..070950|spc_pitch_selected_byte_093|game.asm:24376|
|region-1259|070950..070951|spc_pitch_high_index_10_cell|game.asm:24398|
|region-1087|070951..070952|spc_pitch_selected_byte_095|game.asm:24419|
|region-1088|070952..070953|spc_pitch_selected_byte_096|game.asm:24442|
|region-1390|070953..070954|r12_pitch_high_index_0d_070953|game.asm:24458|
|region-1260|070954..070955|spc_pitch_high_index_14_cell|game.asm:24466|
|region-1089|070955..070956|spc_pitch_selected_byte_099|game.asm:24487|
|region-1261|070956..070957|spc_pitch_high_index_16_cell|game.asm:24509|
|region-1392|070957..070958|r12_pitch_high_index_11_070957|game.asm:24523|
|region-1262|070958..070959|spc_pitch_high_index_18_cell|game.asm:24531|
|region-1394|070959..07095A|r12_pitch_high_index_13_070959|game.asm:24545|
|region-1263|07095A..07095B|spc_pitch_high_index_20_cell|game.asm:24553|
|region-1396|07095B..07095C|r12_pitch_high_index_15_07095b|game.asm:24567|
|region-1264|07095C..07095D|spc_pitch_high_index_22_cell|game.asm:24575|
|region-1398|07095D..07095E|r12_pitch_high_index_17_07095d|game.asm:24589|
|region-1090|07095E..07095F|spc_pitch_selected_byte_108|game.asm:24597|
|region-1265|07095F..070960|spc_pitch_high_index_25_cell|game.asm:24619|
|region-1091|070960..070961|spc_pitch_selected_byte_110|game.asm:24640|
|region-1400|070961..070962|r12_pitch_high_index_1b_070961|game.asm:24655|
|region-1092|070962..070963|spc_pitch_selected_byte_112|game.asm:24663|
|region-1093|070963..070964|spc_pitch_selected_byte_113|game.asm:24686|
|region-1266|070964..070965|spc_pitch_high_index_30_cell|game.asm:24708|
|region-1094|070965..070966|spc_pitch_selected_byte_115|game.asm:24729|
|region-1095|070966..070967|spc_pitch_selected_byte_116|game.asm:24752|
|region-1096|070967..070968|spc_pitch_selected_byte_117|game.asm:24775|
|region-1097|070968..070969|spc_pitch_selected_byte_118|game.asm:24798|
|region-1098|070969..07096A|spc_pitch_selected_byte_119|game.asm:24821|
|region-1099|07096A..07096B|spc_pitch_selected_byte_120|game.asm:24844|
|region-1100|07096B..07096C|spc_pitch_selected_byte_121|game.asm:24867|
|region-1101|07096C..07096D|spc_pitch_selected_byte_122|game.asm:24890|
|region-1102|07096D..07096E|spc_pitch_selected_byte_123|game.asm:24913|
|region-1103|07096E..07096F|spc_pitch_selected_byte_124|game.asm:24936|
|region-1104|07096F..070970|spc_pitch_selected_byte_125|game.asm:24959|
|region-1105|070970..070971|spc_pitch_selected_byte_126|game.asm:24982|
|region-1106|070971..070972|spc_pitch_selected_byte_127|game.asm:25005|
|region-1107|070972..070973|spc_pitch_selected_byte_128|game.asm:25028|
|region-1108|070973..070974|spc_pitch_selected_byte_129|game.asm:25051|
|region-1109|070974..070975|spc_pitch_selected_byte_130|game.asm:25074|
|region-1110|070975..070976|spc_pitch_selected_byte_131|game.asm:25097|
|region-1111|070976..070977|spc_pitch_selected_byte_132|game.asm:25120|
|region-1112|070977..070978|spc_pitch_selected_byte_133|game.asm:25143|
|region-1113|070978..070979|spc_pitch_selected_byte_134|game.asm:25166|
|region-1114|070979..07097A|spc_pitch_selected_byte_135|game.asm:25189|
|region-1115|07097A..07097B|spc_pitch_selected_byte_136|game.asm:25212|
|region-1116|07097B..07097C|spc_pitch_selected_byte_137|game.asm:25235|
|region-1117|07097C..07097D|spc_pitch_selected_byte_138|game.asm:25258|
|region-1118|07097D..07097E|spc_pitch_selected_byte_139|game.asm:25281|
|region-1119|07097E..07097F|spc_pitch_selected_byte_140|game.asm:25304|
|region-1120|07097F..070980|spc_pitch_selected_byte_141|game.asm:25327|
|region-1121|070980..070981|spc_pitch_selected_byte_142|game.asm:25350|
|region-1122|070981..070982|spc_pitch_selected_byte_143|game.asm:25373|
|region-1123|070982..070983|spc_pitch_selected_byte_144|game.asm:25396|
|region-1124|070983..070984|spc_pitch_selected_byte_145|game.asm:25419|
|region-1125|070984..070985|spc_pitch_selected_byte_146|game.asm:25442|
|region-1126|070985..070986|spc_pitch_selected_byte_147|game.asm:25465|
|region-1127|070986..070987|spc_pitch_selected_byte_148|game.asm:25488|
|region-1128|070987..070988|spc_pitch_selected_byte_149|game.asm:25511|
|region-1129|070988..070989|spc_pitch_selected_byte_150|game.asm:25534|
|region-1130|070989..07098A|spc_pitch_selected_byte_151|game.asm:25557|
|region-1131|07098A..07098B|spc_pitch_selected_byte_152|game.asm:25580|
|region-1132|07098B..07098C|spc_pitch_selected_byte_153|game.asm:25603|
|region-1133|07098C..07098D|spc_pitch_selected_byte_154|game.asm:25626|
|region-1134|07098D..07098E|spc_pitch_selected_byte_155|game.asm:25649|
|region-1135|07098E..07098F|spc_pitch_selected_byte_156|game.asm:25672|
|region-1136|07098F..070990|spc_pitch_selected_byte_157|game.asm:25695|
|region-1137|070990..070991|spc_pitch_selected_byte_158|game.asm:25718|
|region-1138|070991..070992|spc_pitch_selected_byte_159|game.asm:25741|
|region-1139|070992..070993|spc_pitch_selected_byte_160|game.asm:25764|
|region-1140|070993..070994|spc_pitch_selected_byte_161|game.asm:25787|
|region-1141|070994..070995|spc_pitch_selected_byte_162|game.asm:25810|
|region-1142|070995..070996|spc_pitch_selected_byte_163|game.asm:25833|
|region-1143|070996..070997|spc_pitch_selected_byte_164|game.asm:25856|
|region-1144|070997..070998|spc_pitch_selected_byte_165|game.asm:25879|
|region-1402|070998..070999|r12_pitch_high_index_52_070998|game.asm:25894|
|region-1404|070999..07099A|r12_pitch_high_index_53_070999|game.asm:25901|
|region-1406|07099A..07099B|r12_pitch_high_index_54_07099a|game.asm:25908|
|region-1407|07099B..07099C|r12_pitch_high_index_55_07099b|game.asm:25915|
|region-1347|07099C..07099D|spc_conditional_effect_pitch_high_index86|game.asm:25925|
|region-1454|07099D..0709A0|Data|game.asm:25932|
|region-1145|0709A0..0709A1|spc_pitch_selected_byte_174|game.asm:25940|
|region-1455|0709A1..0709EA|Data|game.asm:25951|
|region-1146|0709EA..0709EB|spc_pitch_selected_byte_248|game.asm:25959|
|region-1456|0709EB..0709EF|Data|game.asm:25970|
|region-1147|0709EF..0709F0|spc_pitch_selected_byte_253|game.asm:25978|
|region-1457|0709F0..070A24|Data|game.asm:25989|
|region-470|070A24..070A25|spc_song_00_voice_0_pointer_lo|game.asm:25996|
|region-471|070A25..070A26|spc_song_01_voice_0_pointer_lo|game.asm:26021|
|region-472|070A26..070A27|spc_song_02_voice_0_pointer_lo|game.asm:26046|
|region-473|070A27..070A28|spc_song_03_voice_0_pointer_lo|game.asm:26071|
|region-474|070A28..070A29|spc_song_04_voice_0_pointer_lo|game.asm:26096|
|region-1148|070A29..070A31|spc_song_05_12_voice_0_low|game.asm:26120|
|region-475|070A31..070A32|spc_song_13_voice_0_pointer_lo|game.asm:26149|
|region-476|070A32..070A33|spc_song_14_voice_0_pointer_lo|game.asm:26174|
|region-477|070A33..070A34|spc_song_15_voice_0_pointer_lo|game.asm:26199|
|region-1350|070A34..070A3F|spc_reserved_voice0_low_slice_0e30|game.asm:26223|
|region-1149|070A3F..070A40|spc_pitch_selected_byte_333|game.asm:26238|
|region-1351|070A40..070A44|spc_reserved_voice0_low_slice_0e3c|game.asm:26261|
|region-478|070A44..070A45|spc_song_00_voice_0_pointer_hi|game.asm:26275|
|region-479|070A45..070A46|spc_song_01_voice_0_pointer_hi|game.asm:26300|
|region-480|070A46..070A47|spc_song_02_voice_0_pointer_hi|game.asm:26325|
|region-481|070A47..070A48|spc_song_03_voice_0_pointer_hi|game.asm:26350|
|region-482|070A48..070A49|spc_song_04_voice_0_pointer_hi|game.asm:26375|
|region-1150|070A49..070A51|spc_song_05_12_voice_0_high|game.asm:26399|
|region-483|070A51..070A52|spc_song_13_voice_0_pointer_hi|game.asm:26428|
|region-484|070A52..070A53|spc_song_14_voice_0_pointer_hi|game.asm:26453|
|region-485|070A53..070A54|spc_song_15_voice_0_pointer_hi|game.asm:26478|
|region-1352|070A54..070A64|spc_reserved_voice0_high_slice_0e50|game.asm:26502|
|region-486|070A64..070A65|spc_song_00_voice_1_pointer_lo|game.asm:26516|
|region-487|070A65..070A66|spc_song_01_voice_1_pointer_lo|game.asm:26541|
|region-488|070A66..070A67|spc_song_02_voice_1_pointer_lo|game.asm:26566|
|region-489|070A67..070A68|spc_song_03_voice_1_pointer_lo|game.asm:26591|
|region-490|070A68..070A69|spc_song_04_voice_1_pointer_lo|game.asm:26616|
|region-1151|070A69..070A71|spc_song_05_12_voice_1_low|game.asm:26640|
|region-491|070A71..070A72|spc_song_13_voice_1_pointer_lo|game.asm:26669|
|region-492|070A72..070A73|spc_song_14_voice_1_pointer_lo|game.asm:26694|
|region-493|070A73..070A74|spc_song_15_voice_1_pointer_lo|game.asm:26719|
|region-1353|070A74..070A84|spc_reserved_voice1_low_slice_0e70|game.asm:26743|
|region-494|070A84..070A85|spc_song_00_voice_1_pointer_hi|game.asm:26757|
|region-495|070A85..070A86|spc_song_01_voice_1_pointer_hi|game.asm:26782|
|region-496|070A86..070A87|spc_song_02_voice_1_pointer_hi|game.asm:26807|
|region-497|070A87..070A88|spc_song_03_voice_1_pointer_hi|game.asm:26832|
|region-498|070A88..070A89|spc_song_04_voice_1_pointer_hi|game.asm:26857|
|region-1152|070A89..070A91|spc_song_05_12_voice_1_high|game.asm:26881|
|region-499|070A91..070A92|spc_song_13_voice_1_pointer_hi|game.asm:26910|
|region-500|070A92..070A93|spc_song_14_voice_1_pointer_hi|game.asm:26935|
|region-501|070A93..070A94|spc_song_15_voice_1_pointer_hi|game.asm:26960|
|region-1354|070A94..070AA4|spc_reserved_voice1_high_slice_0e90|game.asm:26984|
|region-502|070AA4..070AA5|spc_song_00_voice_2_pointer_lo|game.asm:26998|
|region-503|070AA5..070AA6|spc_song_01_voice_2_pointer_lo|game.asm:27023|
|region-504|070AA6..070AA7|spc_song_02_voice_2_pointer_lo|game.asm:27048|
|region-505|070AA7..070AA8|spc_song_03_voice_2_pointer_lo|game.asm:27073|
|region-506|070AA8..070AA9|spc_song_04_voice_2_pointer_lo|game.asm:27098|
|region-1153|070AA9..070AB1|spc_song_05_12_voice_2_low|game.asm:27122|
|region-507|070AB1..070AB2|spc_song_13_voice_2_pointer_lo|game.asm:27151|
|region-508|070AB2..070AB3|spc_song_14_voice_2_pointer_lo|game.asm:27176|
|region-509|070AB3..070AB4|spc_song_15_voice_2_pointer_lo|game.asm:27201|
|region-1355|070AB4..070AC4|spc_reserved_voice2_low_slice_0eb0|game.asm:27225|
|region-510|070AC4..070AC5|spc_song_00_voice_2_pointer_hi|game.asm:27239|
|region-511|070AC5..070AC6|spc_song_01_voice_2_pointer_hi|game.asm:27264|
|region-512|070AC6..070AC7|spc_song_02_voice_2_pointer_hi|game.asm:27289|
|region-513|070AC7..070AC8|spc_song_03_voice_2_pointer_hi|game.asm:27314|
|region-514|070AC8..070AC9|spc_song_04_voice_2_pointer_hi|game.asm:27339|
|region-1154|070AC9..070AD1|spc_song_05_12_voice_2_high|game.asm:27363|
|region-515|070AD1..070AD2|spc_song_13_voice_2_pointer_hi|game.asm:27392|
|region-516|070AD2..070AD3|spc_song_14_voice_2_pointer_hi|game.asm:27417|
|region-517|070AD3..070AD4|spc_song_15_voice_2_pointer_hi|game.asm:27442|
|region-1356|070AD4..070AE4|spc_reserved_voice2_high_slice_0ed0|game.asm:27466|
|region-518|070AE4..070AE5|spc_song_00_voice_3_pointer_lo|game.asm:27480|
|region-519|070AE5..070AE6|spc_song_01_voice_3_pointer_lo|game.asm:27505|
|region-520|070AE6..070AE7|spc_song_02_voice_3_pointer_lo|game.asm:27530|
|region-521|070AE7..070AE8|spc_song_03_voice_3_pointer_lo|game.asm:27555|
|region-522|070AE8..070AE9|spc_song_04_voice_3_pointer_lo|game.asm:27580|
|region-1155|070AE9..070AF1|spc_song_05_12_voice_3_low|game.asm:27604|
|region-523|070AF1..070AF2|spc_song_13_voice_3_pointer_lo|game.asm:27633|
|region-524|070AF2..070AF3|spc_song_14_voice_3_pointer_lo|game.asm:27658|
|region-525|070AF3..070AF4|spc_song_15_voice_3_pointer_lo|game.asm:27683|
|region-1357|070AF4..070B04|spc_reserved_voice3_low_slice_0ef0|game.asm:27707|
|region-526|070B04..070B05|spc_song_00_voice_3_pointer_hi|game.asm:27721|
|region-527|070B05..070B06|spc_song_01_voice_3_pointer_hi|game.asm:27746|
|region-528|070B06..070B07|spc_song_02_voice_3_pointer_hi|game.asm:27771|
|region-529|070B07..070B08|spc_song_03_voice_3_pointer_hi|game.asm:27796|
|region-530|070B08..070B09|spc_song_04_voice_3_pointer_hi|game.asm:27821|
|region-1156|070B09..070B11|spc_song_05_12_voice_3_high|game.asm:27845|
|region-531|070B11..070B12|spc_song_13_voice_3_pointer_hi|game.asm:27874|
|region-532|070B12..070B13|spc_song_14_voice_3_pointer_hi|game.asm:27899|
|region-533|070B13..070B14|spc_song_15_voice_3_pointer_hi|game.asm:27924|
|region-1358|070B14..070B24|spc_reserved_voice3_high_slice_0f10|game.asm:27948|
|region-534|070B24..070B25|spc_song_00_voice_4_pointer_lo|game.asm:27962|
|region-535|070B25..070B26|spc_song_01_voice_4_pointer_lo|game.asm:27987|
|region-536|070B26..070B27|spc_song_02_voice_4_pointer_lo|game.asm:28012|
|region-537|070B27..070B28|spc_song_03_voice_4_pointer_lo|game.asm:28037|
|region-538|070B28..070B29|spc_song_04_voice_4_pointer_lo|game.asm:28062|
|region-1157|070B29..070B31|spc_song_05_12_voice_4_low|game.asm:28086|
|region-539|070B31..070B32|spc_song_13_voice_4_pointer_lo|game.asm:28115|
|region-540|070B32..070B33|spc_song_14_voice_4_pointer_lo|game.asm:28140|
|region-541|070B33..070B34|spc_song_15_voice_4_pointer_lo|game.asm:28165|
|region-1359|070B34..070B44|spc_reserved_voice4_low_slice_0f30|game.asm:28189|
|region-542|070B44..070B45|spc_song_00_voice_4_pointer_hi|game.asm:28203|
|region-543|070B45..070B46|spc_song_01_voice_4_pointer_hi|game.asm:28228|
|region-544|070B46..070B47|spc_song_02_voice_4_pointer_hi|game.asm:28253|
|region-545|070B47..070B48|spc_song_03_voice_4_pointer_hi|game.asm:28278|
|region-546|070B48..070B49|spc_song_04_voice_4_pointer_hi|game.asm:28303|
|region-1158|070B49..070B51|spc_song_05_12_voice_4_high|game.asm:28327|
|region-547|070B51..070B52|spc_song_13_voice_4_pointer_hi|game.asm:28356|
|region-548|070B52..070B53|spc_song_14_voice_4_pointer_hi|game.asm:28381|
|region-549|070B53..070B54|spc_song_15_voice_4_pointer_hi|game.asm:28406|
|region-1360|070B54..070B64|spc_reserved_voice4_high_slice_0f50|game.asm:28430|
|region-550|070B64..070B65|spc_song_00_voice_5_pointer_lo|game.asm:28444|
|region-551|070B65..070B66|spc_song_01_voice_5_pointer_lo|game.asm:28469|
|region-552|070B66..070B67|spc_song_02_voice_5_pointer_lo|game.asm:28494|
|region-553|070B67..070B68|spc_song_03_voice_5_pointer_lo|game.asm:28519|
|region-554|070B68..070B69|spc_song_04_voice_5_pointer_lo|game.asm:28544|
|region-1159|070B69..070B71|spc_song_05_12_voice_5_low|game.asm:28568|
|region-555|070B71..070B72|spc_song_13_voice_5_pointer_lo|game.asm:28597|
|region-556|070B72..070B73|spc_song_14_voice_5_pointer_lo|game.asm:28622|
|region-557|070B73..070B74|spc_song_15_voice_5_pointer_lo|game.asm:28647|
|region-1361|070B74..070B84|spc_reserved_voice5_low_slice_0f70|game.asm:28671|
|region-558|070B84..070B85|spc_song_00_voice_5_pointer_hi|game.asm:28685|
|region-559|070B85..070B86|spc_song_01_voice_5_pointer_hi|game.asm:28710|
|region-560|070B86..070B87|spc_song_02_voice_5_pointer_hi|game.asm:28735|
|region-561|070B87..070B88|spc_song_03_voice_5_pointer_hi|game.asm:28760|
|region-562|070B88..070B89|spc_song_04_voice_5_pointer_hi|game.asm:28785|
|region-1160|070B89..070B91|spc_song_05_12_voice_5_high|game.asm:28809|
|region-563|070B91..070B92|spc_song_13_voice_5_pointer_hi|game.asm:28838|
|region-564|070B92..070B93|spc_song_14_voice_5_pointer_hi|game.asm:28863|
|region-565|070B93..070B94|spc_song_15_voice_5_pointer_hi|game.asm:28888|
|region-1362|070B94..070BA4|spc_reserved_voice5_high_slice_0f90|game.asm:28912|
|region-566|070BA4..070BA5|spc_song_00_voice_6_pointer_lo|game.asm:28926|
|region-567|070BA5..070BA6|spc_song_01_voice_6_pointer_lo|game.asm:28951|
|region-568|070BA6..070BA7|spc_song_02_voice_6_pointer_lo|game.asm:28976|
|region-569|070BA7..070BA8|spc_song_03_voice_6_pointer_lo|game.asm:29001|
|region-570|070BA8..070BA9|spc_song_04_voice_6_pointer_lo|game.asm:29026|
|region-1161|070BA9..070BB1|spc_song_05_12_voice_6_low|game.asm:29050|
|region-571|070BB1..070BB2|spc_song_13_voice_6_pointer_lo|game.asm:29079|
|region-572|070BB2..070BB3|spc_song_14_voice_6_pointer_lo|game.asm:29104|
|region-573|070BB3..070BB4|spc_song_15_voice_6_pointer_lo|game.asm:29129|
|region-1363|070BB4..070BC4|spc_reserved_voice6_low_slice_0fb0|game.asm:29153|
|region-574|070BC4..070BC5|spc_song_00_voice_6_pointer_hi|game.asm:29167|
|region-575|070BC5..070BC6|spc_song_01_voice_6_pointer_hi|game.asm:29192|
|region-576|070BC6..070BC7|spc_song_02_voice_6_pointer_hi|game.asm:29217|
|region-577|070BC7..070BC8|spc_song_03_voice_6_pointer_hi|game.asm:29242|
|region-578|070BC8..070BC9|spc_song_04_voice_6_pointer_hi|game.asm:29267|
|region-1162|070BC9..070BD1|spc_song_05_12_voice_6_high|game.asm:29291|
|region-579|070BD1..070BD2|spc_song_13_voice_6_pointer_hi|game.asm:29320|
|region-580|070BD2..070BD3|spc_song_14_voice_6_pointer_hi|game.asm:29345|
|region-581|070BD3..070BD4|spc_song_15_voice_6_pointer_hi|game.asm:29370|
|region-1364|070BD4..070BE4|spc_reserved_voice6_high_slice_0fd0|game.asm:29394|
|region-582|070BE4..070BE5|spc_song_00_voice_7_pointer_lo|game.asm:29408|
|region-583|070BE5..070BE6|spc_song_01_voice_7_pointer_lo|game.asm:29433|
|region-584|070BE6..070BE7|spc_song_02_voice_7_pointer_lo|game.asm:29458|
|region-585|070BE7..070BE8|spc_song_03_voice_7_pointer_lo|game.asm:29483|
|region-586|070BE8..070BE9|spc_song_04_voice_7_pointer_lo|game.asm:29508|
|region-1163|070BE9..070BF1|spc_song_05_12_voice_7_low|game.asm:29532|
|region-587|070BF1..070BF2|spc_song_13_voice_7_pointer_lo|game.asm:29561|
|region-588|070BF2..070BF3|spc_song_14_voice_7_pointer_lo|game.asm:29586|
|region-589|070BF3..070BF4|spc_song_15_voice_7_pointer_lo|game.asm:29611|
|region-1365|070BF4..070C04|spc_reserved_voice7_low_slice_0ff0|game.asm:29635|
|region-590|070C04..070C05|spc_song_00_voice_7_pointer_hi|game.asm:29649|
|region-591|070C05..070C06|spc_song_01_voice_7_pointer_hi|game.asm:29674|
|region-592|070C06..070C07|spc_song_02_voice_7_pointer_hi|game.asm:29699|
|region-593|070C07..070C08|spc_song_03_voice_7_pointer_hi|game.asm:29724|
|region-594|070C08..070C09|spc_song_04_voice_7_pointer_hi|game.asm:29749|
|region-1164|070C09..070C11|spc_song_05_12_voice_7_high|game.asm:29773|
|region-595|070C11..070C12|spc_song_13_voice_7_pointer_hi|game.asm:29802|
|region-596|070C12..070C13|spc_song_14_voice_7_pointer_hi|game.asm:29827|
|region-597|070C13..070C14|spc_song_15_voice_7_pointer_hi|game.asm:29852|
|region-1366|070C14..070C24|spc_reserved_voice7_high_slice_1010|game.asm:29876|
|region-598|070C24..070C25|spc_song_01_config_pointer_lo|game.asm:29890|
|region-599|070C25..070C26|spc_song_02_config_pointer_lo|game.asm:29915|
|region-600|070C26..070C27|spc_song_03_config_pointer_lo|game.asm:29940|
|region-601|070C27..070C28|spc_song_04_config_pointer_lo|game.asm:29965|
|region-1165|070C28..070C30|spc_song_05_12_configuration_low|game.asm:29989|
|region-602|070C30..070C31|spc_song_13_config_pointer_lo|game.asm:30018|
|region-603|070C31..070C32|spc_song_14_config_pointer_lo|game.asm:30043|
|region-604|070C32..070C33|spc_song_15_config_pointer_lo|game.asm:30068|
|region-1367|070C33..070C44|spc_reserved_config_low_slice_102f|game.asm:30092|
|region-605|070C44..070C45|spc_song_01_config_pointer_hi|game.asm:30107|
|region-606|070C45..070C46|spc_song_02_config_pointer_hi|game.asm:30132|
|region-607|070C46..070C47|spc_song_03_config_pointer_hi|game.asm:30157|
|region-608|070C47..070C48|spc_song_04_config_pointer_hi|game.asm:30182|
|region-1166|070C48..070C50|spc_song_05_12_configuration_high|game.asm:30206|
|region-609|070C50..070C51|spc_song_13_config_pointer_hi|game.asm:30235|
|region-610|070C51..070C52|spc_song_14_config_pointer_hi|game.asm:30260|
|region-611|070C52..070C53|spc_song_15_config_pointer_hi|game.asm:30285|
|region-1368|070C53..070C64|spc_reserved_config_high_slice_104f|game.asm:30309|
|region-1332|070C64..070C66|spc_effect_pointer_low_slice_1060|game.asm:30327|
|region-857|070C66..070C67|spc_effect_02_low|game.asm:30344|
|region-858|070C67..070C68|spc_effect_03_low|game.asm:30369|
|region-1333|070C68..070C69|spc_effect_pointer_low_slice_1064|game.asm:30393|
|region-859|070C69..070C6A|spc_effect_05_low|game.asm:30410|
|region-860|070C6A..070C6B|spc_effect_06_low|game.asm:30435|
|region-861|070C6B..070C6C|spc_effect_07_low|game.asm:30460|
|region-862|070C6C..070C6D|spc_effect_08_low|game.asm:30485|
|region-1334|070C6D..070C74|spc_effect_pointer_low_slice_1069|game.asm:30509|
|region-1335|070C74..070C76|spc_effect_pointer_high_slice_1070|game.asm:30528|
|region-863|070C76..070C77|spc_effect_02_high|game.asm:30545|
|region-864|070C77..070C78|spc_effect_03_high|game.asm:30570|
|region-1336|070C78..070C79|spc_effect_pointer_high_slice_1074|game.asm:30594|
|region-865|070C79..070C7A|spc_effect_05_high|game.asm:30611|
|region-866|070C7A..070C7B|spc_effect_06_high|game.asm:30636|
|region-867|070C7B..070C7C|spc_effect_07_high|game.asm:30661|
|region-868|070C7C..070C7D|spc_effect_08_high|game.asm:30686|
|region-1337|070C7D..070C84|spc_effect_pointer_high_slice_1079|game.asm:30710|
|region-1338|070C84..070C86|spc_effect_priority_slice_1080|game.asm:30729|
|region-869|070C86..070C87|spc_effect_02_priority|game.asm:30746|
|region-870|070C87..070C88|spc_effect_03_priority|game.asm:30771|
|region-1339|070C88..070C89|spc_effect_priority_slice_1084|game.asm:30795|
|region-871|070C89..070C8A|spc_effect_05_priority|game.asm:30812|
|region-872|070C8A..070C8B|spc_effect_06_priority|game.asm:30837|
|region-873|070C8B..070C8C|spc_effect_07_priority|game.asm:30862|
|region-874|070C8C..070C8D|spc_effect_08_priority|game.asm:30887|
|region-1340|070C8D..070C94|spc_effect_priority_slice_1089|game.asm:30911|
|region-1343|070C94..070C9B|spc_default_envelope_record|game.asm:30931|
|region-1344|070C9B..070D28|spc_initial_song_configuration_workspace|game.asm:30950|
|region-1345|070D28..070D58|spc_initial_envelope_record21_tail_and_later|game.asm:30977|
|region-612|070D58..070DC1|spc_configuration_slice_1154|game.asm:30995|
|region-613|070DC1..070DE5|spc_configuration_slice_11bd|game.asm:31002|
|region-614|070DE5..070E15|spc_configuration_slice_11e1|game.asm:31009|
|region-615|070E15..070E4E|spc_configuration_slice_1211|game.asm:31016|
|region-616|070E4E..070E54|spc_configuration_slice_124a|game.asm:31023|
|region-617|070E54..070EA2|spc_configuration_slice_1250|game.asm:31030|
|region-618|070EA2..070EE1|spc_configuration_slice_129e|game.asm:31037|
|region-1167|070EE1..07106F|spc_song_configuration_slice_12dd|game.asm:31045|
|region-619|07106F..0710FC|spc_configuration_slice_146b|game.asm:31091|
|region-1168|0710FC..071110|spc_song_configuration_slice_14f8|game.asm:31099|
|region-620|071110..07113A|spc_configuration_slice_150c|game.asm:31122|
|region-621|07113A..07116B|spc_configuration_slice_1536|game.asm:31129|
|region-622|07116B..07116C|spc_song_0_sequence_slice_1567|game.asm:31136|
|region-623|07116C..07119D|spc_song_1_sequence_slice_1568|game.asm:31161|
|region-624|07119D..0711BB|spc_song_1_sequence_slice_1599|game.asm:31209|
|region-625|0711BB..0711BF|spc_configuration_slice_15b7|game.asm:31249|
|region-626|0711BF..0711C7|spc_song_1_sequence_slice_15bb|game.asm:31256|
|region-627|0711C7..07124D|spc_song_1_sequence_slice_15c3|game.asm:31285|
|region-1458|07124D..07124E|Data|game.asm:31366|
|region-628|07124E..071273|spc_song_1_sequence_slice_164a|game.asm:31373|
|region-1459|071273..071274|Data|game.asm:31404|
|region-629|071274..0712AB|spc_song_1_sequence_slice_1670|game.asm:31411|
|region-1460|0712AB..0712AC|Data|game.asm:31450|
|region-630|0712AC..071358|spc_song_1_sequence_slice_16a8|game.asm:31457|
|region-1461|071358..071359|Data|game.asm:31558|
|region-631|071359..0713A8|spc_song_1_sequence_slice_1755|game.asm:31565|
|region-1462|0713A8..0713A9|Data|game.asm:31618|
|region-632|0713A9..07140A|spc_song_1_sequence_slice_17a5|game.asm:31625|
|region-1463|07140A..07140B|Data|game.asm:31687|
|region-633|07140B..07143E|spc_song_1_sequence_slice_1807|game.asm:31699|
|region-1372|07143E..071443|conditional_voice_restart_transfer_183a|game.asm:31753|
|region-1464|071443..071444|Data|game.asm:31758|
|region-634|071444..07168D|spc_song_1_sequence_slice_1840|game.asm:31765|
|region-1371|07168D..071696|conditional_song_slur_phrase_1a89|game.asm:32120|
|region-635|071696..071746|spc_song_1_sequence_slice_1a92|game.asm:32136|
|region-636|071746..071790|spc_song_2_sequence_slice_1b42|game.asm:32258|
|region-1465|071790..071791|Data|game.asm:32302|
|region-637|071791..071819|spc_song_2_sequence_slice_1b8d|game.asm:32309|
|region-1466|071819..07181A|Data|game.asm:32396|
|region-638|07181A..07189F|spc_song_2_sequence_slice_1c16|game.asm:32403|
|region-1467|07189F..0718A0|Data|game.asm:32489|
|region-639|0718A0..0718B5|spc_song_2_sequence_slice_1c9c|game.asm:32496|
|region-1468|0718B5..0718B6|Data|game.asm:32517|
|region-640|0718B6..071957|spc_song_2_sequence_slice_1cb2|game.asm:32524|
|region-1469|071957..071958|Data|game.asm:32618|
|region-641|071958..071A07|spc_song_2_sequence_slice_1d54|game.asm:32625|
|region-1470|071A07..071A0A|Data|game.asm:32731|
|region-642|071A0A..071A4B|spc_song_2_sequence_slice_1e06|game.asm:32738|
|region-1471|071A4B..071A4E|Data|game.asm:32790|
|region-643|071A4E..071AEF|spc_song_2_sequence_slice_1e4a|game.asm:32797|
|region-1472|071AEF..071AF0|Data|game.asm:32929|
|region-644|071AF0..071DC7|spc_song_3_sequence_slice_1eec|game.asm:32936|
|region-645|071DC7..071DFD|spc_song_4_sequence_slice_21c3|game.asm:33356|
|region-1473|071DFD..071DFE|Data|game.asm:33397|
|region-646|071DFE..071E5D|spc_song_4_sequence_slice_21fa|game.asm:33404|
|region-1474|071E5D..071E5E|Data|game.asm:33467|
|region-647|071E5E..071E97|spc_song_4_sequence_slice_225a|game.asm:33474|
|region-1475|071E97..071E98|Data|game.asm:33514|
|region-648|071E98..071F0B|spc_song_4_sequence_slice_2294|game.asm:33521|
|region-1476|071F0B..071F0C|Data|game.asm:33593|
|region-649|071F0C..071F71|spc_song_4_sequence_slice_2308|game.asm:33600|
|region-1477|071F71..071F72|Data|game.asm:33666|
|region-650|071F72..071FD7|spc_song_4_sequence_slice_236e|game.asm:33673|
|region-1478|071FD7..071FD8|Data|game.asm:33739|
|region-651|071FD8..071FDA|spc_song_4_sequence_slice_23d4|game.asm:33746|
|region-1169|071FDA..072016|spc_race_song_sequence_slice_23d6|game.asm:33772|
|region-1479|072016..072017|Data|game.asm:33811|
|region-1170|072017..07209D|spc_race_song_sequence_slice_2413|game.asm:33820|
|region-1480|07209D..07209E|Data|game.asm:33901|
|region-1171|07209E..0721A1|spc_race_song_sequence_slice_249a|game.asm:33910|
|region-1481|0721A1..0721A2|Data|game.asm:34052|
|region-1172|0721A2..0721C9|spc_race_song_sequence_slice_259e|game.asm:34061|
|region-1173|0721C9..0721DB|spc_race_song_sequence_slice_25c5|game.asm:34105|
|region-1174|0721DB..072215|spc_race_song_sequence_slice_25d7|game.asm:34140|
|region-1482|072215..072216|Data|game.asm:34183|
|region-1175|072216..0722A3|spc_race_song_sequence_slice_2612|game.asm:34192|
|region-1483|0722A3..0722A4|Data|game.asm:34290|
|region-1176|0722A4..072303|spc_race_song_sequence_slice_26a0|game.asm:34299|
|region-1484|072303..072304|Data|game.asm:34370|
|region-1177|072304..072306|spc_race_song_sequence_slice_2700|game.asm:34379|
|region-1178|072306..072381|spc_race_song_sequence_slice_2702|game.asm:34404|
|region-1485|072381..072382|Data|game.asm:34470|
|region-1179|072382..0723D3|spc_race_song_sequence_slice_277e|game.asm:34479|
|region-1486|0723D3..0723D4|Data|game.asm:34531|
|region-1180|0723D4..072404|spc_race_song_sequence_slice_27d0|game.asm:34540|
|region-1487|072404..072405|Data|game.asm:34575|
|region-1181|072405..072463|spc_race_song_sequence_slice_2801|game.asm:34584|
|region-1488|072463..072464|Data|game.asm:34649|
|region-1182|072464..0724A5|spc_race_song_sequence_slice_2860|game.asm:34658|
|region-1489|0724A5..0724A6|Data|game.asm:34712|
|region-1183|0724A6..0724E5|spc_race_song_sequence_slice_28a2|game.asm:34721|
|region-1490|0724E5..0724E6|Data|game.asm:34762|
|region-1184|0724E6..0725A6|spc_race_song_sequence_slice_28e2|game.asm:34771|
|region-652|0725A6..0725CA|spc_song_15_sequence_slice_29a2|game.asm:34896|
|region-1185|0725CA..072650|spc_race_song_sequence_slice_29c6|game.asm:34944|
|region-1491|072650..072651|Data|game.asm:35020|
|region-1186|072651..072718|spc_race_song_sequence_slice_2a4d|game.asm:35029|
|region-1492|072718..072719|Data|game.asm:35151|
|region-1187|072719..07272C|spc_auxiliary_stream_2b15|game.asm:35160|
|region-1188|07272C..0727CE|spc_race_song_sequence_slice_2b28|game.asm:35191|
|region-1493|0727CE..0727CF|Data|game.asm:35282|
|region-1189|0727CF..072871|spc_race_song_sequence_slice_2bcb|game.asm:35291|
|region-1494|072871..072872|Data|game.asm:35382|
|region-1190|072872..0728C3|spc_race_song_sequence_slice_2c6e|game.asm:35391|
|region-1495|0728C3..0728C4|Data|game.asm:35441|
|region-1191|0728C4..072919|spc_race_song_sequence_slice_2cc0|game.asm:35450|
|region-1496|072919..07291A|Data|game.asm:35506|
|region-1192|07291A..07291C|spc_race_song_sequence_slice_2d16|game.asm:35515|
|region-1193|07291C..07295A|spc_race_song_sequence_slice_2d18|game.asm:35540|
|region-1497|07295A..07295B|Data|game.asm:35583|
|region-1194|07295B..07297E|spc_race_song_sequence_slice_2d57|game.asm:35592|
|region-1498|07297E..07297F|Data|game.asm:35623|
|region-1195|07297F..0729A3|spc_race_song_sequence_slice_2d7b|game.asm:35632|
|region-1499|0729A3..0729A4|Data|game.asm:35660|
|region-1196|0729A4..0729D0|spc_race_song_sequence_slice_2da0|game.asm:35669|
|region-1500|0729D0..0729D1|Data|game.asm:35700|
|region-1197|0729D1..072AF6|spc_race_song_sequence_slice_2dcd|game.asm:35709|
|region-1501|072AF6..072AF7|Data|game.asm:35895|
|region-1198|072AF7..072B2E|spc_race_song_sequence_slice_2ef3|game.asm:35904|
|region-1502|072B2E..072B2F|Data|game.asm:35942|
|region-1199|072B2F..072BC7|spc_race_song_sequence_slice_2f2b|game.asm:35951|
|region-1200|072BC7..072C4E|spc_race_song_sequence_slice_2fc3|game.asm:36090|
|region-1503|072C4E..072C4F|Data|game.asm:36170|
|region-1201|072C4F..072D61|spc_race_song_sequence_slice_304b|game.asm:36179|
|region-1504|072D61..072D62|Data|game.asm:36333|
|region-1202|072D62..072D6B|spc_race_song_sequence_slice_315e|game.asm:36342|
|region-1203|072D6B..072D7E|spc_auxiliary_stream_3167|game.asm:36375|
|region-1204|072D7E..072E9F|spc_race_song_sequence_slice_317a|game.asm:36406|
|region-1505|072E9F..072EA0|Data|game.asm:36561|
|region-1205|072EA0..07302F|spc_race_song_sequence_slice_329c|game.asm:36570|
|region-1206|07302F..07309D|spc_race_song_sequence_slice_342b|game.asm:36808|
|region-1506|07309D..07309E|Data|game.asm:36868|
|region-1207|07309E..0730E2|spc_race_song_sequence_slice_349a|game.asm:36877|
|region-1507|0730E2..0730E3|Data|game.asm:36923|
|region-1208|0730E3..07310C|spc_race_song_sequence_slice_34df|game.asm:36932|
|region-1508|07310C..07310D|Data|game.asm:36964|
|region-1209|07310D..0731A1|spc_race_song_sequence_slice_3509|game.asm:36973|
|region-1509|0731A1..0731A2|Data|game.asm:37059|
|region-1210|0731A2..073237|spc_race_song_sequence_slice_359e|game.asm:37068|
|region-1510|073237..073238|Data|game.asm:37155|
|region-1211|073238..0732CC|spc_race_song_sequence_slice_3634|game.asm:37164|
|region-1511|0732CC..0732CD|Data|game.asm:37250|
|region-1212|0732CD..07330E|spc_race_song_sequence_slice_36c9|game.asm:37259|
|region-653|07330E..073382|spc_song_13_sequence_slice_370a|game.asm:37319|
|region-1213|073382..0733CB|spc_race_song_sequence_slice_377e|game.asm:37399|
|region-1512|0733CB..0733CC|Data|game.asm:37446|
|region-1214|0733CC..073435|spc_race_song_sequence_slice_37c8|game.asm:37455|
|region-1513|073435..073436|Data|game.asm:37540|
|region-1215|073436..0734E6|spc_race_song_sequence_slice_3832|game.asm:37549|
|region-1514|0734E6..0734E7|Data|game.asm:37647|
|region-1216|0734E7..0735AE|spc_race_song_sequence_slice_38e3|game.asm:37656|
|region-1515|0735AE..0735AF|Data|game.asm:37767|
|region-1217|0735AF..07374E|spc_race_song_sequence_slice_39ab|game.asm:37776|
|region-1516|07374E..07374F|Data|game.asm:38009|
|region-1218|07374F..0737B0|spc_race_song_sequence_slice_3b4b|game.asm:38018|
|region-1517|0737B0..0737B1|Data|game.asm:38098|
|region-1219|0737B1..0737E0|spc_race_song_sequence_slice_3bad|game.asm:38107|
|region-1220|0737E0..073857|spc_race_song_sequence_slice_3bdc|game.asm:38160|
|region-1518|073857..073858|Data|game.asm:38229|
|region-1221|073858..0738E1|spc_race_song_sequence_slice_3c54|game.asm:38238|
|region-1519|0738E1..0738E2|Data|game.asm:38322|
|region-1222|0738E2..073985|spc_race_song_sequence_slice_3cde|game.asm:38331|
|region-1520|073985..073986|Data|game.asm:38429|
|region-1223|073986..073A3B|spc_race_song_sequence_slice_3d82|game.asm:38438|
|region-1521|073A3B..073A3C|Data|game.asm:38549|
|region-1224|073A3C..073ABD|spc_race_song_sequence_slice_3e38|game.asm:38558|
|region-1522|073ABD..073ABE|Data|game.asm:38635|
|region-1225|073ABE..073BD7|spc_race_song_sequence_slice_3eba|game.asm:38644|
|region-1523|073BD7..073BD8|Data|game.asm:38799|
|region-1226|073BD8..073CB5|spc_race_song_sequence_slice_3fd4|game.asm:38808|
|region-654|073CB5..073DF9|spc_song_15_sequence_slice_40b1|game.asm:38970|
|region-655|073DF9..073F3B|spc_song_14_sequence_slice_41f5|game.asm:39152|
|region-656|073F3B..073F4E|spc_song_3_sequence_slice_4337|game.asm:39341|
|region-1341|073F4E..073F5B|spc_effect_01_sequence|game.asm:39377|
|region-875|073F5B..073F6E|spc_effect_bytes_073f5b|game.asm:39401|
|region-876|073F6E..073F7F|spc_effect_bytes_073f6e|game.asm:39435|
|region-1342|073F7F..073F90|spc_effect_04_sequence|game.asm:39467|
|region-877|073F90..073FA7|spc_effect_bytes_073f90|game.asm:39493|
|region-1346|073FA7..073FBE|spc_conditional_standalone_effect_sequence|game.asm:39527|
|region-878|073FBE..073FF3|spc_effect_bytes_073fbe|game.asm:39554|
|region-879|073FF3..074029|spc_effect_bytes_073ff3|game.asm:39613|
|region-1524|074029..074C04|Data|game.asm:39658|
|region-657|074C04..074C08|spc_sample_00_directory|game.asm:39665|
|region-1318|074C08..074C0C|spc_sample_directory_slice_5004|game.asm:39695|
|region-658|074C0C..074C10|spc_sample_02_directory|game.asm:39711|
|region-659|074C10..074C14|spc_sample_03_directory|game.asm:39739|
|region-1227|074C14..074C18|spc_sample_04_directory|game.asm:39766|
|region-660|074C18..074C1C|spc_sample_05_directory|game.asm:39788|
|region-1228|074C1C..074C20|spc_sample_06_directory|game.asm:39815|
|region-661|074C20..074C24|spc_sample_07_directory|game.asm:39837|
|region-662|074C24..074C28|spc_sample_08_directory|game.asm:39865|
|region-1319|074C28..074C2C|spc_sample_directory_slice_5024|game.asm:39892|
|region-663|074C2C..074C30|spc_sample_10_directory|game.asm:39908|
|region-1229|074C30..074C34|spc_sample_11_directory|game.asm:39935|
|region-664|074C34..074C38|spc_sample_12_directory|game.asm:39957|
|region-665|074C38..074C3C|spc_sample_13_directory|game.asm:39985|
|region-666|074C3C..074C40|spc_sample_14_directory|game.asm:40013|
|region-667|074C40..074C44|spc_sample_15_directory|game.asm:40041|
|region-668|074C44..074C48|spc_sample_16_directory|game.asm:40069|
|region-1320|074C48..074C54|spc_sample_directory_slice_5044|game.asm:40096|
|region-669|074C54..074C58|spc_sample_20_directory|game.asm:40114|
|region-670|074C58..074C5C|spc_sample_21_directory|game.asm:40142|
|region-671|074C5C..074C60|spc_sample_22_directory|game.asm:40170|
|region-672|074C60..074C64|spc_sample_23_directory|game.asm:40198|
|region-673|074C64..074C68|spc_sample_24_directory|game.asm:40226|
|region-674|074C68..074C6C|spc_sample_25_directory|game.asm:40254|
|region-1321|074C6C..074C70|spc_sample_directory_slice_5068|game.asm:40281|
|region-880|074C70..074C74|spc_sample_27_directory|game.asm:40298|
|region-881|074C74..074C78|spc_sample_28_directory|game.asm:40326|
|region-675|074C78..074C7C|spc_sample_29_directory|game.asm:40353|
|region-676|074C7C..074C80|spc_sample_30_directory|game.asm:40381|
|region-677|074C80..074C84|spc_sample_31_directory|game.asm:40409|
|region-1322|074C84..074C88|spc_sample_directory_slice_5080|game.asm:40436|
|region-1230|074C88..074C89|spc_instrument_note_base_index_00_slice|game.asm:40454|
|region-1323|074C89..074C8A|spc_instrument_note_base_slice_5085|game.asm:40477|
|region-1231|074C8A..074C8B|spc_instrument_note_base_index_02_slice|game.asm:40495|
|region-1232|074C8B..074C91|spc_instrument_note_base_index_03_slice|game.asm:40519|
|region-1324|074C91..074C92|spc_instrument_note_base_slice_508d|game.asm:40542|
|region-1233|074C92..074C99|spc_instrument_note_base_index_10_slice|game.asm:40560|
|region-1325|074C99..074C9C|spc_instrument_note_base_slice_5095|game.asm:40583|
|region-1234|074C9C..074C9F|spc_instrument_note_base_index_20_slice|game.asm:40601|
|region-1235|074C9F..074CA2|spc_instrument_note_base_index_23_slice|game.asm:40625|
|region-1326|074CA2..074CA3|spc_instrument_note_base_slice_509e|game.asm:40648|
|region-1267|074CA3..074CA4|spc_instrument_27_note_base_cell|game.asm:40665|
|region-1268|074CA4..074CA5|spc_instrument_28_note_base_cell|game.asm:40686|
|region-1236|074CA5..074CA8|spc_instrument_note_base_index_29_slice|game.asm:40708|
|region-1237|074CA8..074CA9|spc_instrument_pitch_scale_index_00_slice|game.asm:40732|
|region-1327|074CA9..074CAA|spc_instrument_pitch_scale_slice_50a5|game.asm:40755|
|region-1238|074CAA..074CAB|spc_instrument_pitch_scale_index_02_slice|game.asm:40773|
|region-1239|074CAB..074CB1|spc_instrument_pitch_scale_index_03_slice|game.asm:40797|
|region-1328|074CB1..074CB2|spc_instrument_pitch_scale_slice_50ad|game.asm:40820|
|region-1240|074CB2..074CB9|spc_instrument_pitch_scale_index_10_slice|game.asm:40838|
|region-1329|074CB9..074CBC|spc_instrument_pitch_scale_slice_50b5|game.asm:40861|
|region-1241|074CBC..074CBF|spc_instrument_pitch_scale_index_20_slice|game.asm:40879|
|region-1242|074CBF..074CC2|spc_instrument_pitch_scale_index_23_slice|game.asm:40903|
|region-1330|074CC2..074CC3|spc_instrument_pitch_scale_slice_50be|game.asm:40926|
|region-1269|074CC3..074CC4|spc_instrument_27_pitch_scale_cell|game.asm:40943|
|region-1270|074CC4..074CC5|spc_instrument_28_pitch_scale_cell|game.asm:40964|
|region-1243|074CC5..074CC8|spc_instrument_pitch_scale_index_29_slice|game.asm:40986|
|region-1331|074CC8..074CD1|spc_silent_loop_brr|game.asm:41009|
|region-678|074CD1..074DE8|spc_sample_00_brr|game.asm:41026|
|region-679|074DE8..074E81|spc_sample_02_brr|game.asm:41034|
|region-680|074E81..074EDB|spc_sample_03_brr|game.asm:41042|
|region-1244|074EDB..07508B|spc_sample_04_brr|game.asm:41051|
|region-681|07508B..0751BD|spc_sample_05_brr|game.asm:41058|
|region-682|0751BD..075742|spc_sample_07_brr|game.asm:41066|
|region-683|075742..075A51|spc_sample_08_brr|game.asm:41074|
|region-684|075A51..075DA8|spc_sample_10_brr|game.asm:41082|
|region-1245|075DA8..0764CB|spc_sample_11_brr|game.asm:41091|
|region-685|0764CB..076822|spc_sample_12_brr|game.asm:41098|
|region-686|076822..07688E|spc_sample_13_brr|game.asm:41106|
|region-687|07688E..0768FA|spc_sample_14_brr|game.asm:41114|
|region-688|0768FA..0774DC|spc_sample_15_brr|game.asm:41122|
|region-689|0774DC..077548|spc_sample_16_brr|game.asm:41130|
|region-690|077548..077E04|spc_sample_20_brr|game.asm:41138|
|region-89|077E04..077E08|spc_upload_block1_terminator|game.asm:41145|
|region-1526|077E08..078000|Data|game.asm:41148|
|region-90|078000..078004|spc_upload_block2_header|game.asm:41157|
|region-691|078004..078597|spc_sample_20_brr_continued|game.asm:41164|
|region-692|078597..078C72|spc_sample_21_brr|game.asm:41172|
|region-693|078C72..079E0F|spc_sample_22_brr|game.asm:41180|
|region-694|079E0F..07A1E4|spc_sample_23_brr|game.asm:41188|
|region-695|07A1E4..07AA0C|spc_sample_24_brr|game.asm:41196|
|region-696|07AA0C..07BD62|spc_sample_25_brr|game.asm:41204|
|region-882|07BD62..07C63E|spc_sample_27_brr|game.asm:41213|
|region-697|07C63E..07D4D5|spc_sample_29_brr|game.asm:41223|
|region-698|07D4D5..07E8BB|spc_sample_30_brr|game.asm:41231|
|region-699|07E8BB..07FA2B|spc_sample_31_brr|game.asm:41239|
|region-1525|07FA2B..07FE04|Data|game.asm:41242|
|region-91|07FE04..07FE08|spc_upload_block2_terminator|game.asm:41249|
|region-1527|07FE08..080000|Data|game.asm:41252|

## Stage4 upload terminator correction
The Stage3 table/test both-entry0400 claim is superseded only for the first stream. Actual ROM77E04 bytes0000C0FF and original game.asm41145 establish first execution handoffFFC0 (IPL); second7FE04 bytes00000004 remains0400. Decoder unchanged, fixture tuple now supplies each source entry. Initial definitions14/15 retained. Both payloads remainexact7E00 and consumed7E08; no gap/header enters sample data.

Stage4 auxiliary normal-stream fixture correction: its default duration0 equaled defaultrelease_threshold0, naturally invoking the unconditional source release scan (game.asm20870) after the assertednormal token read. Added duration1 to isolate intended ordinary/restart/hold path, preserving every assertion. Separate release-scan fixture unchanged; no artificial nonzero guard added. Initialvoice27/28 retained.
