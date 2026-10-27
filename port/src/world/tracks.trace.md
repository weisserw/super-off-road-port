# Tracks independent trace / executable contract evidence

Worker `/root/behavior_contracts/tracks`; source frozen Round29
`reference/source/game.asm` SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
Adjacent tracks.rs contains pending domain interfaces, not implemented behavior.
Every tracks-assigned region in planning/source-ownership.json was reviewed:
selection/order, all code $055F..$068B, all 64 heading bytes $068B..$06CB,
parameter readers/tables/loading $11FE..$141F and route bundle $57FA..$5FFE.
CPU bank00 code addresses = physical+$8000; asset pointers use full LoROM bank
translation. References consulted: toolkit docs/development.md platform table,
docs/snes.md address/width rules, docs/snes-native.md rendering boundaries;
W65C816 width/carry semantics as linked in runtime.trace.md. No CPU/core source
was used as an implementation or test oracle.

## Selection and descriptor columns

`select_track_descriptor` game.asm:345, physical $0200..$0216, PHP/REP20 clears
track_reversed as a WORD, then SEP30 uses byte track_order_slot with no mask.
Sign of the order byte sets reverse to1; AND7F stores selected ID. Thus clearing
reverse also clears adjacent scratch high byte, not a second semantic field.
`decode_order` validates original bounds once; `select_order` rejects out-of-range
indices rather than silently masking them. Race owns increment-and-mask63 at
`finish_race_and_increment_order` (:309), not this selector. All64 metadata
entries are asserted from the local ROM table $0216..$0256, including normal15,
reverse15 and no natural reverse09. Absence from order never removes reverse09.

`load_track_byte_parameters` and `load_track_word_parameters` :4349..4390,
physical $11FE..$123A, begin SEP30 with byte track ID. Single-byte bank/scale
columns use index t; TXA/ASL then REP20 uses word columns index2t. Parameters:
X offsets $123A, Y offsets $125A, Y scale $127A, X scale $128A,
BG1 low $129A/bank $12BA, BG2 low $12CA/bank $12EA,
terrain bank $12FA/low $130A. Exactly16 entries each =>240-byte table.
Graphics triples $13EF..$141F use 3t (byte ASL+ADC track ID) and bank at +2.
All16 independently inspected expected descriptor tuples appear in the test;
raw word offsets FFFF/FFFE remain their source bit patterns, signed interpretation
belongs to terrain projection. Scale axis order is [X,Y], not storage order.
Malformed partial tables fail before use. No invented track names are asserted.

## Routes: complete stream views, not archive region ownership

`load_route_pointer` :985 masks selected ID to0F in16-bit A; ASL indexes a word
pointer. Reversed nonzero word adds $20. Table $57FA..$583A has32 bank00 CPU
pointers, resolved physical=pointer-$8000. Copy at :1015 reads/stores a WORD,
then ANDFF00/CMPFF00. The terminating word itself is copied; only afterward
SEP20 extracts its low byte to checkpoint count and writes each of four checkpoint
indices=1. The route owner exposes terminal/count, while race initialization
owns writing truck checkpoint records, so there is no duplicate state owner.
Control words and packed coordinates are preserved for race/AI consumers
(:2019, :2383, :8601), not normalized into a guessed pair array.

All32 pointer-selected streams were independently walked in the ROM. Word counts
and terminal low bytes are explicit metadata expectations. First high-FF word
ends even when low byte differs; lowFF alone does not end. Thus no fixed pair
count can replace source termination. Forward08 begins $5A60 and reaches FF0F
after61 words, while reverse08 begins $5ADA and reaches FF0F after63 words.
Forward13/reverse13 have47/55 words and overlap named consumer windows. Reverse14
points earlier to $583A (21 words/count9), unlike forward14 (25/count11).
Reverse09 at $5B76 remains loaded (15 words/count6), despite no natural order.
Source has no malformed-input guard; bounded truncation/error is native asset
validation, not a source branch. Tests reject incomplete words, absent sentinel,
truncated ROM and unsupported original ID without embedding original route data.
The generic decoder permits trailing resource bytes after its sentinel, necessary
for overlapping consumer windows. Catalogue validation separately requires valid
route semantics for added content.

## Wheel/grid setup: branch is heading, not reverse flag

`clear_wheel_state_and_select_route` :975 actually returns before route loading:
SEP30 loop X=15..0 clears exactly16 bytes of sampled/integrated/velocity arrays.
It does NOT clear truck_surface, which later grid initialization sets. Race calls
clear wheels, clears sprite animation, loads route, loads track assets, initializes
grid in that order (:110..200). This distinction is contracted directly.

`initialize_start_grid` :1041 uses byte index2*(track&0F), +$20 when reversed.
`track_start_heading_lookup` symbol $868B points physical $068B; only EVEN bytes
are consumed, all odd interleaves are zero but their meaning is not invented.
The32 consumed bytes comprise forward16 then reverse16. Synthetic nonzero odd
bytes prove two-byte stride rather than treating heading as a word. Heading low
byte is written to each truck word; race already initialized high bytes zero.
Domain Grid.heading is that byte; no whole-word overwrite claim is made for
arbitrary high-byte preconditions not established by the native interface.

After REP20/AND00FF the source branches on heading !=0. Nonzero heading leaves
A equal to heading itself ($80 in original table); ZERO heading falls through
`reverse_grid_step` to A=$FC00. Thus the branch is NOT simply direction=reverse:
forward track03 has heading0, reverse03 has heading80, reverse14 also80.
`scratch_word_54` keeps this signed displacement. X base=(first route word&BFE0)
+displacement wrapping16; X columns are base/base+$0300. Ybase=second word&FFE0,
rows Ybase-$0100 and that+$01E0. Initial terrain probe **origin** uses [Xbase,Ybase],
not the first truck's row and not its later permuted column. Grid commit initially
writes source truck order; if displacement==$FC00 it swaps X columns by writing
prepared0→truck2,1→truck3,2→truck0,3→truck1. `alternate_start_grid_rows` :1192
loads track_order_slot as WORD, but LSR carry only depends on low bit: odd slots
swap rows within0/1 and2/3, independent of X swap/direction. Tests cover both
heading paths, odd/even order, BFE0 control-bit clearing, underflow/wrap, non-table
nonzero heading1 (source tests zero, not equality80). Fixed expectations were
calculated directly from these masks/add/subtract steps, not game output.

After commit, `calculate_wheel_probe_position` and `sample_terrain_height_or_block`
are terrain-owned cross-calls (:6773/:6411).
The wheel helper uses inherited working_heading_or_pose_angle and Y&3 to add signed sine offsets to that origin; Grid does not pretend its origin is the final probe. load_track_assets ends Y=$0600 and grid SEP30 truncates Y to0, selecting wheel0 on this natural call; it does not assign the working pose angle. Terrain owns that inherited-state/offset contract. Returned byte fills all16 integrated heights and four truck_surface bytes;
sampled/velocity remain unchanged by these final stores. `apply_initial_surface`
models only that result application; terrain worker must contract actual probe
and height computation and any helper side effects. Grid is not an alternate
terrain implementation. Truck checkpoint initial1 and race update ordering are
race contracts, linked above, rather than reimplemented here.

## ROM content loading / presentation boundary

`copy_track_terrain_and_upload_video` :4437, physical $132A, has a separately
callable terrain-copy entry, PHP/REP10/SEP20; Y=X=$16B0 decrement through0 copies
**$16B1** bytes, not the frequently inferred $16B0 stride between most tracks.
Adjacent tracks overlap by one source byte; preserve that consumer view.
24-bit indirect-long,Y carries across address banks. Original views all remain
within their selected source bank, but physical bank identity must be retained.
`load_track_assets` :4451 first loads all parameters, then terrain, graphics,
palette, map clears and map transfers. Graphics DMA length=$8000, source three
bytes; all original graphics pointers are $03:8000 or $0C:8000, so transfers end
FFFF and no wrap ambiguity exists for these original assets. CPU LoROM pointers
must NOT be used as physical offsets ($04:8000→$020000, $0A:96AA→$0516AA,
$0D:9212→$069212). Added malformed/unmapped pointers are rejected by loaders.

Palette copies512 bytes from race_palette_transfer_source. Three map clears call
presentation-owned fill_vram_with_caller_word (inclusive clear semantics), then
BG1 transfer1536 bytes to word VRAM4040, BG2 transfer1536 to4440, OR20 on each
BG2 high byte (tile attribute2000). **TrackContent stores raw BG bytes**;
presentation must apply that attribute during staging, not mutate immutable ROM
content. Graphic planar decode/composition, palette, layer configuration, DMA bus
edge semantics and inclusive map fills remain rendering contracts; terrain
interpolation remains terrain contracts. `load_content` assembles raw semantic
consumer windows, headings and both routes; descriptor metadata remains separate.
ROM slice equality tests sample three distinct pointer/bank/stride cases with
exact lengths and direct original storage, never learn data from native output.

## Extensible named track contract (native requirement, not original format)

Game-owned `tracks.json` MemoryAssets manifest merges ROM-backed NamedTrack
entries with added/replacement files via collection validation; its order may
reference originals and extensions, without fixing total length16. Proposed
track file JSON version1 carries projection_offset word pair/projection_scale byte pair, terrain byte array ($16B1), two background byte
arrays ($600 each), graphics byte array ($8000), two u16 route arrays including
highFF terminal and two heading bytes. Source original bounds remain16 and64;
these do not limit the catalogue size or manifest order. Tests use one original
plus an added synthetic track, check actual contents/order, malformed terrain,
missing order IDs and missing route sentinel. No source assets are embedded.

This file format is an explicit native interface choice for extension support;
it is not a claim the SNES reads JSON. Additional track metadata (object definitions, countdown/title) must accompany this definition during stage4
when the associated area interfaces exist. Current fixture targets structural
raw windows only; **these contracts alone do not establish complete playable
extension support**, nor do loaded raw resources implement game behavior.

## Evidence status

All interfaces remain explicit unimplemented stubs; no game behavior or test
oracle was implemented. See checks/contracts/tracks-results.json for exact pinned
compile/discovery/area and infrastructure commands, counts and exit codes.
No QA, toolkit edit, reference edit, ledger disposition or emulator experiment.

Final execution: Rust1.90 locked compile/list passed, 34 suite tests discovered
(17 tracks +13 runtime +4 infrastructure). Tracks run0 passed/17 expected
unimplemented failures/0 ignored; assets2 and game2 infrastructure tests passed.
Replacement and17-slot catalogue contracts were included in this final run.

## Stage 4 fixture correction

The native CountdownLayout became required during the later transitions worker. The earlier tracks custom_json fixture omitted its countdown field while content() expected center[91,139]/destination0x4900. Stage4 added that exact explicit field to the synthetic JSON rather than defaulting missing file metadata. This repairs fixture consistency without changing a source-game expectation or relaxing extension validation. Original Stage3 logs remain in checks/contracts/.

## Stage4 playable added-route correction
Game-selected added contract initially4/5 exposed a source-format hole: synthetic count2 had onlyrecords0/1 thenFF02, but AI next_target (game.asm next-target body and ai.rs) advances to fullrecord2; original35word route has17coordinatepairs0..16 plusFF10. File route count names the LAST checkpoint, requiringcount+1 fullpairs. Added native validation for this minimum pluscompletepair framing/controlredirect targets/human checkpoint presence; original variablelengthsource views remain unchanged. Earlier tiny added count1 fixture likewise onlyhadrecord0; appendedrequiredrecord1 to both semanticfixture andJSON (retains existingequalcontent/order/addition assertions). InitialGame4/1 and priortracks19pass preserved.
