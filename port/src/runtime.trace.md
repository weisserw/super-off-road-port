# Runtime source trace and contract evidence

Worker: `/root/behavior_contracts/runtime`. Frozen Round29 source
`reference/source/game.asm`, SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
All bank-zero addresses below are CPU `$008000 + physical offset`, not arbitrary
ROM indexing in rules. Tests live in adjacent runtime.rs and are native-discovered.
No behavior is implemented by these interfaces.

CPU reference consulted: WDC W65C816S datasheet, March 13 2024,
https://www.westerndesigncenter.com/wdc/documentation/w65c816s.pdf,
sections 2.4/2.7/2.8, operation tables and 7.20/7.23. Destination width controls
transfers; A's hidden high byte survives byte-width operations. Toolkit context:
`/root/romatlas/docs/development.md`, `docs/snes.md`, `docs/snes-native.md`.
The native port models domain outcomes and ordering, never CPU execution/RAM.

## Boot: region-1, physical $000000..$0000AA, CPU $008000

Independently read game.asm:19..102. CLC/XCE leaves emulation mode; REP #$20
allows DP=0 and stack=$01FF; DB=0 is installed with byte PHA/PLB. Native hardware
register details are an implementation mechanism, not fabricated player state.
The externally relevant call/effect sequence is reset video, disable NMI, upload
audio, force blank/inhibit, configure Mode1 maps/font base, font load/background
clear, explicit initial state writes, sprite transfer setup, frontend command
$0201 to APUIO2/3, force blank/inhibit, frontend dispatcher, video/sprite reset,
then first track descriptor selection. The dispatch loop also writes inhibit=$80,
active human mask=1 and order slot=0 before entering the frontend. Domain
BootEffect batches repeated adjacent blank/inhibit writes as one semantic effect;
it does not omit a frame wait or call. No wait occurs between those repeated writes.
`boot_cross_owner_effects_retain_source_call_order` fixes this orchestration.

SEP #$30 establishes byte A/X. STZ steering/speed/acceleration each clear only
slot0. Four explicit suspension STZs clear all four slots. Reentry byte clears.
REP #$20 with X still byte runs X=6,4,2,0: each word STZ clears one acceleration
fraction, all four words. Six word stores clear interpolation base, A, B, center,
C and D. These named scratch inputs are represented as terrain_corners in the
boot preparation interface, not source-address fields. No boot store addresses
RNG history or the frontend signature. The boot-write contract initializes
nonzero retained state so accidental full-array clearing is observable. Later
frontend clearing all20 upgrade/stock bytes is a distinct owner effect.

Caller/callee boundaries: presentation owns reset-video/font/OAM and graphics;
audio owns upload and command handling; attract owns frontend dispatch, splashes,
session initialization and Start stack unwinding; tracks owns descriptor decode.
Runtime BootEffect is the integration request ordering, not duplicate algorithms.

## RNG: region-174, physical $0019C2..$0019F3, CPU $0099C2

Read game.asm:6900..6933; symbol bytes are contiguous $4D..$51. Let old bytes be
s0..s4. PHP preserves entry status; SEP #$20 affects A only. First ROR s4 sets
carry=old s4 bit0. Its rotated value is overwritten, so **entry carry is irrelevant**.
LDA s3/PHA/ROR A then yields feedback `(s3>>1)|((s4&1)<<7)` in temporary s4.
Four ASLs of s2 yield low8(s2<<4) in temporary s3. Original s1 moves to s2;
four LSRs produce s1>>4, OR temporary s3 and XOR feedback produce new s0.
STX restores original s2 to s3; LDX s0/STX s1 moves old s0; PLA restores old s3
into s4. Final domain transformation:

- new0 = `((s1>>4) | low8(s2<<4)) XOR ((s3>>1)|((s4&1)<<7))`
- new1=s0, new2=s1, new3=s2, new4=s3.

For **16-bit X**, TAX transfers A+B: intermediate STX s3 writes a second byte
into s4, but the later PLA/STA overwrites it. LDX s0 reads old s0/old s1;
STX s1 writes those to new1/new2, matching the byte-index result. Thus no hidden
A high byte survives in the final history. For **16-bit entry A**, PLP restores
word width before final LDA s0, so return = new0 | (new1<<8), not just a byte.
X is restored by PLX. PLP restores carry/overflow/decimal/M/X, while PLX and final
LDA update N/Z; do not claim all final condition flags are preserved.

Independently derived fixed vectors:
`12 34 56 78 9B`: nibble combine=$63, feedback=$BC => $DF;
next combine=$41, feedback=$2B => $6A.
`A5 F0 0F 81 00`: $FF XOR $40=$BF, word return=$A5BF.
`01 FF FF FF FE`: $FF XOR $7F=$80; same with last byte FF gives $00.
All zero stays zero; reseeding inside the transform would contradict source.
These constants are fixtures, not an executable duplicate RNG oracle.

Call contexts inspected: game.asm:2720..2795 (AI route advance establishes
SEP #$30), :8930..9030 (rank adjustment byte context), :15040..15115 (pickup
PHP/REP #$30, masks word return with $00FF or $0003), :15545..15605 (sequence
object initializer REP #$30 and masks word return with $0007). All consume the
low result; native domain callers should use Byte width unless explicitly testing
the original word contract. Object/AI workers must preserve RNG call counts and
ordering, including the zero-spawn gate. No seed write is established in the
boot/frontend paths; deterministic native power-on seed is explicitly unresolved
in uncertainties.json, not silently inferred from a helper name.

## NMI: physical $001BCA..$001C07, CPU $009BCA

Read game.asm:7214..7250, cross-calls upload_oam_shadow and
upload_four_truck_pose_tiles (presentation owned, including latter's SEP #$30).
REP #$30/CLD and word pushes preserve A/X/Y across the interrupt; hardware status
is restored by RTI. Inhibit is tested as a byte: **every nonzero** value bypasses
uploads. Zero invokes staged OAM then staged truck pose transfer; it reinstalls
word VRAM destination=$4820 and byte VMAIN=$80 after helpers change widths.
Only then SEP #$20 makes all three counter INCs byte width: NMI count,
animation count and audio heartbeat wrap independently modulo256. The incremented
heartbeat is written to APUIO0. Finally wait while HVBJOY bit0 is set before
restoring registers/RTI. FrameEffect records this order with a completed controller
poll boundary; it does not emulate a host busy wait. Tests cover normal visible
work, all meaningful nonzero inhibit classes and independent wrap endpoints.

This is a **cross-area contract**, not an ownership reassignment: the navigation
index places NMI in presentation; runtime owns counters/cadence and asks
presentation to consume previously staged objects/poses. Pure rendering and
observation must not advance counters. Transition wait/fade contracts depend on
these independent NMI/animation counters. Audio owns interpreting heartbeat.
Do not move race controller copies ahead of movement (race owns that dispatcher).

## Persistence/reset and full native state

Read frontend initializer game.asm:7775..7860, signature copy :8030..8045,
signature definition :8050 and default-record copy :12580..12625.
The frontend compares **all five** bytes descending 4..0 against
`12 0F 16 16 19`; any mismatch clears signature byte0, runs licensed splash and
copies record defaults. Valid signature skips that copy. Both routes initialize
session money, but do not copy record defaults on warm reentry. The later splash
sequence copies the five-byte signature. Attract/results workers own executable
signature and record-copy rules; runtime's boot test protects the preserved
signature boundary. A full cold native Game::reset must be deterministic; a
warm in-game frontend reentry is distinct and must not be replaced with reset.

Game integration contracts require read-only `runtime.nmi_frames` and
`runtime.random_history` semantic observations, deterministic reset, clone replay,
full snapshot restoration of runtime and pending transition state, and transactional
rejection of wrong-ROM snapshots. Existing four infrastructure tests remain
unchanged and passing; current Game lacks these behavior observations, so those
two contracts correctly fail. Runtime history exposure is a five-byte domain
sequence, not an opaque RAM dump. Counter JSON naming uses `nmi_frames` even
though source byte wraps; documentation must state modulo256 when implemented.

## Metadata, padding, and unknown entry assessment

Primary review index has14 runtime regions (earlier prose count13 was incorrect):
boot/RNG, one unknown gap, header and10 padding allocations. Read every assigned
padding declaration and underlying local ROM: all10 ranges contain only $FF.
They are allocation tails with preceding bounded data/code, not game behavior;
no tests of trivial FF preservation replace a behavior. Ledger classification
remains pending for implementation accounting; no regions were excluded here.
Header physical $007FC0..$008000 has cartridge type0, SRAM size0, region1,
reset vector $8000 and native NMI $9BCA. Metadata decoder contracts read the
owner's ROM (no copied original assets), assert those domain facts and reject
truncation. No-battery hook contract passes with current default unsupported
save hooks. Session records must not manufacture original battery persistence.

Unknown $0019C0..$0019C2 bytes are `01 FF`, immediately after OAM helper's PLP/RTS
and before the separately called RNG. No established direct consumer or branch
entry to either byte was found in read source. Neither an instruction nor an
unused-padding claim is justified; preserve unknown identity and pending impact
assessment. No native state or seeded constant is invented from them.

## Executed evidence

Pinned `cargo +1.90.0 test --locked --manifest-path port/Cargo.toml --no-run`
passes. `-- --list` discovers17 tests:13 runtime plus4 infrastructure. Runtime
run: **1 pass,12 expected pending failures,0 ignored**; ten unimplemented domain
contracts and two missing semantic-observation assertions. Assets and Game
infrastructure separately pass2+2. Logs and exact commands/exit codes:
`checks/contracts/runtime-results.json` and runtime-*.log. No compile failures,
hangs, behavior implementation, QA, toolkit edit, audit or reference modification.

Superseded Stage4 integration policy (Stage5 correction below): source boot does not establish power-on RNG history. Game cold/reset now uses named COLD_RANDOM_HISTORY12,34,56,78,9B hex, explicitly a deterministic native assumption to avoid the feedback transform's absorbing zero state. prepare_boot still preserves caller-supplied history; the original zero-transform contract is unchanged. Warm frontend paths retain the existing history.

Stage5 global cold policy now config::COLD_RANDOM_HISTORY=[0;5], matching the deterministic original zero-WRAM reference baseline. Initial originalgrid1573 versus native ready-delay10 isolated71pixels of pickup presence from the prior nonzero assumption; no other pixels differed after BG/OBJ coordinate repair. Physical console history remains unestablished; no source seed write is claimed. Warm retained history and source zero-feedback remain unchanged. Prior policy/captures retained in uncertainties.json and QA evidence.

## Follow-up: randomized player startup

The Stage5 zero-WRAM default above is superseded: zero is an absorbing state and disables gameplay randomness. Player-facing Port loading now samples five bytes using getrandom (native OS / browser Web Crypto), rejecting histories whose only set bits would be discarded from byte4. The source transform is unchanged, including its zero-input contract. Explicit seeded loading and the headless --rng-seed option preserve deterministic reference/setup tests without choosing zero for players. The initial history is saved/restored separately from advancing history, and reset within the loaded game is reproducible.
