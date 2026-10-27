# Controls independent trace and pending executable contracts

Worker `/root/behavior_contracts/controls`; frozen Round29
`reference/source/game.asm` SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
Primary evidence reread: maintained source instructions, symbols and matching local
ROM bytes. `reference/reading.asm` carries the same execution/storage views.
Consulted toolkit docs/development.md platform table, docs/snes.md address/context
rules and native input.rs/snes.rs/`Game::tick_players` interfaces. As documented
there, native E=0 operand widths are M-dependent for A and X-dependent for index
registers; SEP10 discards index high bytes and PLP cannot reconstruct them.
These are domain contracts, not a CPU or address-indexed state model.

All 67 controls-index regions were reviewed: region-1280 physical
$00141F..$001438, 64 byte regions covering paired tables $001438..$001478,
region-48 $00405B..$00409A, region-49 $00409A..$004123. Shared source consumers
and the controller-copy subentries of race-owned $000743..$0007B0 were also read.
The tables split low/high bytes across ledger identities; this is one signed-word
layout, not 64 unrelated game concepts. No ledger identity/disposition changed.

## Direction selectors, widths and original expectations

`decode_controller_direction_words` game.asm:4545, physical $141F, CPU $00941F:
PHP saves P; REP20 forces word A; SEP10 makes indices byte-wide. LDA DP78 loads
working_controller_word. XBA then AND000F selects bits8..11 of the original word;
ASL gives even byte index0..30. LDA absolute $9438,X and $9458,X reads full
little-endian words, stored into DP11/13. PLP restores incoming status, including
carry; A/X are scratch. Y low is unchanged, but SEP10 already discarded Y high
(and X high) on X16 entry. Restoring X16 width does not restore those high bytes.
DB00/DP0000 and an ordinary same-bank JSR are conditional calling assumptions,
independently supported by paired table layout and byte consumer; there is no
established natural caller. No test asserts reachability or creates a player mode
for this helper. `word_delta` expresses the supported semantic result, not CPU
register side effects. `controls-word-helper-entry` retains that uncertainty.

`decode_frontend_direction` game.asm:14370, physical $4097, CPU $00C097, saves
P and SEP10 only. Its continuation at $409A performs AND0F/ASL/TAX then reads
those same table bases and writes DP11 and13 in the **caller's A width**.
The emitted AND immediate is byte-sized: all natural callers below establish M8.
Do not pretend arbitrary M16 entry is another decoded variant. Natural execution
reads one low byte from each even-indexed word, leaving the neighboring scratch
bytes alone. Cursor arithmetic uses that byte as signed via BMI after ADC.
Native `MenuDelta` is therefore signed i8, while conditional `WordDelta` is i16;
the synthetic 1281/FE81 test distinguishes low-byte interpretation from word sign
or clamping. PHP/PLP preserves caller flags despite internal ASL carry and LDA
N/Z; consumers explicitly CLC before coordinate additions. Neither delta helper
changes game-owned held state or repeat timers.

Local ROM inspection of the two16-word tables, not future implementation output,
yields these ordered delta pairs for selector bits R=1,L=2,D=4,U=8:

| Nibble | Column | Row |
| --- | --- | --- |
| 0 | 0 | 0 |
| 1 | +1 | 0 |
| 2 | -1 | 0 |
| 3 | 0 | 0 |
| 4 | 0 | +1 |
| 5 | +1 | +1 |
| 6 | -1 | +1 |
| 7 | 0 | 0 |
| 8 | 0 | -1 |
| 9 | +1 | -1 |
| A | -1 | -1 |
| B | 0 | 0 |
| C..F | 0 | 0 |

Importantly simultaneous left/right suppresses *both* axes even with up/down,
and simultaneous up/down also suppresses both axes. This is not independently
cancelling opposite axes. Decode from ROM-backed tables; do not replace the table
with a guessed steering algorithm. Selector mask excludes B/Y/Select/Start in
the high byte and excludes all low controller-byte buttons. All64 table bytes
are consumed: high bytes00 for zero/+1 and FF for -1 under word-helper context.
Loader tests use local ROM, signed domain expectations and synthetic parser
fixtures; malformed table lengths fail rather than reading partial/trailing data.
The original table read is a named checked ROM location, not an arbitrary tile demo.

Natural callers independently followed:

- Name edit player1 :13995..14045 and player2 :14136..14195 establish SEP30,
  guard joined/unfinished state, maintain eight-count repeat and add decoded
  column/row with CLC. Grid wrap/erase/EXIT and repeat rules belong to names.
- Shop player1 :10700..10760 and player2 :11090..11135 enter M8/X8 from done
  guards; direction-nibble BIT gates the helper, then explicit cursor wrap and
  geometry run. Confirm edges/history and eight-count repeat belong to economy.
- Word helper has no labelled direct source caller found. That is a search lead,
  not proof of unreachability, and does not authorize exclusion.

## Frontend automatic-controller sampling

`poll_frontend_controllers` game.asm:14344, physical $405B..$4097, CPU $00C05B:
PHP/SEP30 makes all reads/writes byte width. Page0 takes the first branch and
**every nonzero page** takes the alternate branch. Both read JOY1H→player1 high,
JOY2H→player2 high, JOY1L→player1 low, JOY2L→player2 low, with identical devices
and ordering. Source byte destinations are split at055A/055B/055C/055D; native
`FrontendInputs` reassembles whole words without losing low-byte A/X/L/R buttons.
The sampler does not filter for joined/done participants and does not read JOY3/4
on later pages. Page selects participant record ownership elsewhere. PHP/PLP
preserves status but SEP30 truncates X/Y highs even though lows are untouched.
No callback or helper inside this routine changes carry between the copies.

Name frame services (:13647..13705) update join countdown, sample controllers,
pack sprites, test both joins, then edit both names in the same frame. Shop
(:10565..10580) waits a frame, redraws money/one upgrade field, samples controllers,
then runs both done/input handlers. They need the just-sampled frontend words,
not race's one-motion-frame delay. Source owner-specific edge/repeat calculations
must use these held words, not host `pressed` events whose cadence can differ.
The page tests use four different full hardware words and pages0/1/2/FF to expose
both branch behavior and accidental JOY3/4 substitution.

## Enabled race input copies and actual dispatcher latency

`copy_enabled_controller_words`, game.asm:2058..2145, physical $0782..$07B0,
CPU $008782: REP30 establishes word A/X. It loads active_human_mask as a word,
AND00FF excludes the adjacent byte, and BEQ returns for zero. Four successive LSR
instructions test original bits0,1,2,3 in carry. LDX JOYnL/STX truck word are full
16-bit operations and do not alter carry/A. Thus each shifted bit remains correct
across its conditional copy. Disabled slots retain previous synthesized/AI words;
there is no blanket zeroing and no direction-only mask. Bits4..7 can make the
initial BEQ false but enable none of the four tested slots. Return leaves widths
word-sized; `check_enabled_route_checkpoints` immediately SEP30 and constructs
its own shifted mask and descending slot scan, so no helper-register purity is
assumed. Route rectangle arithmetic/flags and lap progression belong to race,
not controls; its carry-clear acceptance contract must be traced there.

`run_race_frame` (:227, physical0198) waits for completed NMI polling, handles
pause/attract Start, then `update_four_trucks` (:3391) loads **retained** truck
controller_words into working_controller_word for acceleration and motion.
Wheel/suspension/pose/sort/OAM/animation services follow. Only then does
`poll_controls_and_check_routes` copy new enabled hardware words, followed by
checkpoint tests. Objects/banner/HUD follow; AI updates are last and can replace
non-human retained words. Thus this frame's motion uses the previous retained
sample, checkpoint processing follows this frame's human copy, and next motion
uses those human words plus the newest AI generation. `RaceInputPhases` makes
that semantic boundary explicit for the future race coordinator; do not run this
sampling before motion simply because the native function returns both values.
The two-frame test uses distinct old/hardware/AI words, human mask05 and release
on the next frame to catch reordered polling or loss of disabled AI words.
Race owns actual call order and must consume this boundary in its implementation;
this contract does not implement/mimic vehicle motion as an oracle.

## Shared native action boundary

Toolkit `snes::default_settings` supplies12 held action names and independent
`MultiInputState.players`. `encode_players` maps current held actions into source
SNES standard automatic words B8000/Y4000/Select2000/Start1000/Up0800/Down0400/
Left0200/Right0100/A0080/X0040/L0020/R0010. Reserved low nibble remains zero.
These encodings agree with source direction reads above, Start high-byte10,
Select high-byte20 joins, B high-byte80 shop confirm and working-controller
acceleration/nitro consumers; exact game use remains those owners' contracts.
Host pressed/released sets do not override held state; all four players remain
isolated, and opposite directions are preserved until the source table consumer.
Unknown action names contribute nothing. This is the common desktop/browser
input adapter requirement, not a new game rule. Game::tick must adapt its primary
single-player InputState to player0 and Game::tick_players must use independent
players; neither frontend should own a second console-word translator.

## Mixed-region rendering handoff and pause boundaries

Primary region49 includes unrelated rendering tails after the delta RTS; none
may disappear due to primary controls index ownership. Independently read:
`hide_frontend_sprites` physical40AA..40DB starts SEP20/REP10, Y01FC and X007F,
writing F0 to both each128 OAM entries' first byte **and Y byte**, decrementing Y
by4 each iteration. It sets all16 native sprite Y bytes toF0, all32 OAM high
bytesAA, then calls pack_sorted_sprites_to_oam (which can further change the
shadow). Do not assume the final packed first entries retain both prepack bytes.
`upload_frontend_fonts` physical40DB..4123 saves P, chooses byte VMAIN80, word
VRAM destination5000, and sequentially streams two0800-byte long-addressed ROM
fonts08D440 and08DA20. It then sets X4800 and calls clear_vram_and_reset_ppu;
that helper's inclusive clear and PPU side effects are **not pure**, so rendering
must trace/test final effects and source order. PLP restores caller status.
Rendering worker must add substantive hide/font/upload contracts with these
source entry points; this handoff is not an exclusion or a claim of tested tails.

Pause owner/release belongs to transitions: source fourth candidate tests JOY3H
again even though subsequent indexed owner read uses selected slot's JOYnH.
No generic sampler should silently "fix" that asymmetry or own pause edges.
Controls does not duplicate transition state. Full game snapshots must eventually
include retained controls and every owner-specific edge/repeat state separately.

## Actual contract execution

`port/src/controls.rs` defines pending native interfaces and15 executable tests.
Rust1.90.0 locked --no-run and --list succeed (74 total suite tests). All15 area
tests execute and fail specifically at pending unimplemented stubs; no compile,
fixture, hang or ignored-test failure. Both assets infrastructure tests and both
Game infrastructure tests pass unchanged. Commands, return codes and logs are in
checks/contracts/controls-results.json. No source behavior, QA, emulator run,
toolkit change, reference edit, or ledger completion claim was made.
