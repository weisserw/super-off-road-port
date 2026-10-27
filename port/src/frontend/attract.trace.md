# Attract and session-entry source trace

Independent worker `/root/behavior_contracts/attract`, Stage 3. All native rule
bodies remain explicit pending stubs. Tests are native-discovered in
`frontend::attract::tests`. No runtime/emulator evidence was used as an oracle.
Adopted source: frozen Round29 `reference/source/game.asm`, SHA-256
4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f;
`reference/reading.asm` provides the same maintained view. Source CPU bank00
$8000+offset corresponds to physical offset for routines in this area; asset
banks use LoROM mapping `(bank & 0x7f)*0x8000+(address & 0x7fff)`.
Toolkit docs/development.md platform table, docs/snes.md CPU width/address rules
and docs/snes-native.md independent mechanisms consulted. No CPU/RAM interpreter
is introduced. Native Session/Truck/VideoMemory/Display are actual owners, not a
phase/effect mock oracle.

## Entry, cold signature and common session initialization

`initialize_frontend_and_attract` CPU00:A037 physical2037 (game.asm7775)
first REP30, sends APUIO2 word0201, clears continuation word078F..0790, SEP30,
clears BG1 scroll latches, shape byte054B and companion054C, CGWSEL/CGADSUB,
enables NMI+auto joypad81, BG3SC4A and frontend_page_limit0. Only lower BG3 map
configuration is touched here; renderer owns its map-size interpretation.
Source COLDATA is not written, so retained fixed color must not reset. Existing
Display retains no CPU registers; scroll/math source effects are semantic fields.
`begin_entry` asserts these writes and does not prematurely clear wallets/records.

The five-byte signature comparison scans index4 down0 against physical21AE
`12 0F 16 16 19`. All five must match for the warm path. On ANY mismatch,
STZ frontend_ram_signature clears ONLY byte0, calls show_licensed_splash (which
falls through the complete licensed/software timed sequence), and only AFTER
that returns calls copy_frontend_record_defaults (game.asm12580 physical363F).
That helper copies48 track-name,32 time,60 score-digit,30 score-name,10 score-order,
12 truck-name bytes, then restores status. SessionRecords is one shared value;
results worker must refine/reuse it, never create an independent preserved copy.
The cold record copy deliberately follows completed first splash/signature install.

Warm signature does not preserve the wallets: both branches call
initialize_money_and_record_defaults at361F (game.asm12564). It clears24 money
bytes by descending word stores22..0, then writes1 to offsets2,8,14,20;
selected-money truck becomes0. Despite the helper label, its natural entry does
NOT fall through record defaults (RTS separates the internal record-copy entry).
Thus every wallet is [0,0,1,0,0,0] while valid records/names survive. No battery
save is implied: source header SRAM0, documented separately by runtime.

After money init REP10 saves stack for nonlocal reentry, SEP20 increments the
video-inhibit BYTE, sends1F to enabled-mask/TM/TMW, then SEP30 changes only retained
enabled-mask to05. Loop X=13h..0 clears20 contiguous bytes at061B..062E:
four nitro, acceleration, steering, speed, suspension bytes. No speed, heading,
fraction, wheel, button-history or cooldown array is cleared by that loop.
The native initialize_session contracts actual Truck upgrades/stock resets and
preserved motion fields, not a synthetic RAM range. Nonzero reentryD2 is consumed
(clear0) and branches to driver selection after all shared resets; it bypasses
panel/highscores/second licensed presentation and attract rotation.

If no reentry: prepare_attract_panel, blank_before_frontend_upload (which falls
through show_scrolling_highscores), show_licensed_splash again. The apparent
extra licensed/software sequence on cold initialization is source call ordering,
not a redundant sequence to delete. The first startup splash establishes signature
and defaults; later attract splashes can Start-unwind with that valid signature.

After these calls return, active-human mask0; byte0637 increments with wrap then
AND03. Table physical20DE bytes0,1,2,3 selects original ORDER slots, not named
track catalogue indices. Initial rotation0 chooses slot1,3 wraps0,FF wraps0.
Four nitro stock bytes become99 BCD. Final path INC inhibit, masks-only clear,
INIDISP80 forceblank, RTS. It does not clear records, wallets, unrelated truck
motion, or install any invented title/menu feature. finish_attract_presentation
asserts rotation, human mask, all stocks and real display blank/window mutation.

## Input poll, stack unwind and joining completion

`poll_start_or_splash_timeout` physical2C38 CPUAC38 (game.asm10334): SEP20,
OR all four JOY high bytes, AND10. Start tests ONLY signature FIRST byte, not a
full five-byte revalidation. Nonzero first byte JMP resume_frontend_from_start;
no timer decrement. Zero first byte falls through ordinary countdown even with
Start held. Without Start: REP20 DEC timer word, SEP20 RTS. SEP changes width
but preserves N, so caller BPL uses 16-bit DEC sign, not a low-byte sign.
Fixtures timer8000→7FFF Continue and8001→8000 TimedOut distinguish width.
Timer0→FFFF TimedOut;140 permits140 nonnegative polls then the141st underflows;
110 similarly111 polls. A generic “timeout equals zero” would be one frame early.
Other controller bits mask away; all four Start slots are required.

Warm first byte12 with other signature bytes corrupt still Start-reenters in poll;
only full entry comparison classifies warm/cold. The tests separate these rules.
No native saved-stack integer is needed: Poll::StartReentry is a nonlocal domain
transition; it must unwind the current sequence instead of executing its remaining
fade/record/rotation operations. The separate race poll at physical026E discards
return bytes/increments reentry and jumps boot frontend wrapper; race contracts
already assert that all-controller transition. It is not the same helper as AC38.

`resume_frontend_from_start` physical20E2 CPUA0E2 restores saved stack, calls
begin_player_selection THEN prepare_player_shop_frontend before checking joined
pages as a WORD. Even no joined players execute shop first. The native completion
interface is called only after these actual owners complete, not a replacement
for names/shop operations. Zero joined word restarts full initialize_frontend entry
(EntryPhase::RestartEntry), with no AI stock/order mutation by this completion.
It must NOT bypass next entry's audio/signature/display writes.

Nonzero word then uses byte page0 OR(page1<<2), with BYTE wrap and NO final low4
mask. Fixture81 and42 produces89. give_ai_initial_nitro starts Y16=3 and mask08,
visits slots3..0, writes15 BCD only to inactive slots and preserves human stock.
Then order slot0; increment inhibit/masks-only clear/forceblank as shared final
path. Tests [0,2] distinguish nonzero second page (active8) from empty first page;
they require retained human stock and AI15. Frontend page sampling asymmetry remains
controls/names owners, not a new player-count menu.

## Licensed/software sequence and asset consumers

`show_licensed_splash` physical2130 (game.asm7969): forceblank80, enabled/TM/TMW1,
load_frontend_graphics, draw licensed rectangle destination WORD4047 source
05:E4E0 (physical2E4E0) descriptor00:A1B3, fadein, REP20 timer140 and enter loop.
Each timed loop waits NMI BEFORE poll. Fadeout does not poll Start itself;
automatic inputs are tested only source poll sites. After licensed timer expiry,
fadeout, draw software destination40C3 source05:E864 descriptorA1C7, fadein,
set timer110. After second expiry, copy signature all five bytes descending,
fadeout, RTS. Source initial signature0 therefore makes early Start ineffective
through both first waits; only later warm splashes can abort at a poll.

Map descriptors are10 little-endian words: licensed width18,rows25,palettes0..7;
software width23,rows17,palettes0..7. Payload450 words900bytes and391words782bytes.
Palette remap bit15 behavior and row strides use shared presentation rectangle
rules, not independent duplicated decoding. Sprite/text/font owners remain separate.
FrontendAssets keeps original words ROM-backed; no original art embedded in tests.

Frontend graphics helper2130→21DB increments video-inhibit BYTE and disables HDMA,
waits one NMI, sets VRAM word4000 clear (401 words, zero, bytes8000..8801), copies
06:D400 physical35400 length2580 bytes to VRAM0, then05:DAC0 physical2DAC0 length940
bytes to VRAM word12C0 (byte2580). These are two distinct LoROM banks, not physical
adjacency. Palette color0 is zeroed; colors1..15 preserved;112 colors from05:E400
physical2E400 lengthE0 installed at color16..127; colors128..255 preserved.
No OAM or other VRAM below map is implicitly cleared. Real synthetic VideoMemory
contracts distinguish untouched sentinel bytes, palette and OAM preservation,
inhibitFF→00 wrapping, and transactional invalid resource rejection. Atomic reject
for native malformed assets is an implementation safety contract, not original
out-of-ROM behavior. All original spans load through named definitions.

Splash contains explicit phase, timer and cloneable Fade; advance_splashes consumes
one NMI boundary, invokes shared real fades and modifies VideoMemory/Display at
map installation boundaries. Tests initialize phase boundaries independently:
license0→FFFF switches fadeout without changing maps/signature; completed first
fadeout installs software map and fresh fadein; fadein completions set correct
wait timers; second0→FFFF writes all signature bytes before final fade; completed
last fade returns Done preserving signature. Warm Start mutates only transition
intent while retaining timer/maps/signature. These pending contracts are not a
second sequence interpreter or implementation oracle.

## Attract DMA and panel loop

begin_attract_graphics_upload physical31B5 PHP/REP10 prefix falls into31B8;
natural caller was SEP30 so M8 remains until explicit REP20. clear map4000,
disable HDMA; DMA graphics08:8000 len4D00→VRAM WORD0; palette08:D340 len100→CGRAM0;
map08:CD00 len600→VRAM WORD4040 (byte8080); clear maps4800 and4C00 separately,
each401 words. Palette only128colors, upper128 untouched. Source ranges physical
40000..44D00,45340..45440,44D00..45300. The40-byte gap45300..45340 is NOT copied
or assigned a fake role. Zero fills4800/4C00 overlap one word: final contiguous
byte9000..A001 zero;A002 sentinel must survive. First4000 map clear yields
8000..8080 prefix and8680..8802 suffix zero around loaded600-byte map.

prepare_attract_panel internal A248 forceblanks, sets retained layer05, clears
same20 stock/upgrades, uploads real attract assets, sets TM/TMW05, enables
window-Start-abort1 and calls initialize_frontend_window_motion (transitions).
That sets centerE0/80, masks03/00/00 and enters scrolling-window setup:100 waittimer,
geometry, two NMI HDMA priming waits. The begin_panel contract asserts native
Truck clearing, wallet preservation, actual VRAM, Display masks and Window state.
Transitions owns all window arithmetic/double-buffer staging; no duplicate model.

After window returns, each overlay boundary waits NMI, brightness8, normalize
selected wallet via economy, brightness15, then AC38 poll. Source normalization
walks selected six digits5..0; example [0,0,0,0,0,12] becomes[0,0,0,0,1,2], other
wallets untouched. Even Start still performs prior normalization/brightness writes;
only then reentry without timer decrement. Tests assert actual selected wallet,
brightness, state, timeout and Start. transitions::advance_attract_overlay already
has internal ordering contracts and must share attract::poll_start behavior;
composition may not substitute a phase-only response for actual normalization.
Normal timeout proceeds to standard driver panels/highscores; Start leaves directly.

## High-score scroll and adjacent owner boundaries

blank_before_frontend_upload physical2433 forceblanks then calls
prepare_standard_driver_panels (physical3426→3431). This installs driver tiles and
palette and draws common map at4121/4131 using separate descriptors. Names owns
those graphics/panels; results owns ten highscore rows. Source falls through
show_scrolling_highscores physical2441 (game.asm8550): heading rectangle
source09:CDA0→WORD4020 descriptor00:B3C2; draw_highscore_rows; BG3 scroll0118;
fadein; RAM scroll0118 and timer500 AFTER fade completes. begin_highscores is the
activation boundary after actual heading/rows/fade, not permission to skip them.

Each frame waits NMI, loads scroll WORD, compares01D6, increments retained scroll
if below, then writes the PREincrement A through BG3 latch, polls AC38. Thus0118
RAM→0119 while displayed0118;01D5→01D6 displaying01D5; at01D6 remains/display01D6.
Timeout is501 polls, although scroll caps after190 increments. Start is tested
AFTER scroll write/increment and preserves timer. On timed finish enable abort1,
call begin_contracting_window (frontend /16 contraction, not race /4), then poll
ONCE MORE before returning. Transitions contracts cover this closing window and
allowabort wrapper final poll. Tests retain real Display scroll, capped source
values, timer/sign/reentry, not a synthetic scrolling frame counter.

## Coverage, limits and stage4 integration handoff

All23 assigned regions below were reread with callers, callees and consumers.
No ledger disposition or reference files changed. No unresolved source fact needed
a new arbitrary behavior assumption in this area. Existing startup undefined-RAM
choices remain runtime uncertainty; frontend records/signature preserve source
within-session semantics rather than inventing battery persistence. No extra
emulator experiment, gameplay QA or platform research was performed.

Stage4 must compose Session, actual name/shop/result owners, shared Progress,
WorldObjects/Truck state and real Display/VideoMemory. Attract does not own a
second race winner/banner or copied results record table. SessionRecords/wallet
interfaces introduced here are shared owner proposals for upcoming results/economy
workers to refine/reuse; Game snapshots must include all frontend/splash/window/
fade/wallet/input state. Pure render must not advance those phases. Startup paths
are real ROM-backed splashes/attract panels, not an invented title screen.

32 tests compile/list successfully (388 suite total) on locked Rust1.90.0;
32 area failures all explicit pending-unimplemented panics, zero ignored.
Four infrastructure tests pass separately. Exact commands/results/log paths:
checks/contracts/attract-results.json. Passing compilation or infrastructure does
not establish source behavior implemented or game fidelity.

## Exhaustive assigned navigation (not implementation accounting)

- region-7: physical$002037..$0020DE, game.asm:7775; initialize_frontend_and_attract.
- region-806: physical$0020DE..$0020E2, game.asm:7859; attract_track_order_slots.
- region-8: physical$0020E2..$00210A, game.asm:7887; resume_frontend_from_start.
- region-734: physical$00210A..$002125, game.asm:7917; give_ai_initial_nitro.
- region-9: physical$002125..$00212A, game.asm:7943; inhibit_race_video_and_clear_window.
- region-807: physical$00212A..$002130, game.asm:7955; blank_display_after_attract_setup.
- region-10: physical$002130..$002160, game.asm:7969; show_licensed_splash.
- region-735: physical$002160..$002166, game.asm:7999; set_frontend_wait_140.
- region-11: physical$002166..$0021AE, game.asm:8012; wait_and_replace_startup_splashes.
- region-985: physical$0021AE..$0021B3, game.asm:8050; frontend_initialization_signature_bytes.
- region-65: physical$0021B3..$0021C7, game.asm:8073; licensed_splash_rectangle_descriptor.
- region-67: physical$0021C7..$0021DB, game.asm:8089; software_splash_rectangle_descriptor.
- region-12: physical$0021DB..$002275, game.asm:8108; load_frontend_graphics_and_prepare_attract_panel.
- region-14: physical$002433..$002441, game.asm:8532; blank_before_frontend_upload.
- region-809: physical$002441..$00249D, game.asm:8550; show_scrolling_highscores.
- region-15: physical$002C38..$002C5C, game.asm:10334; poll_start_or_splash_timeout.
- region-17: physical$0031B5..$0031B8, game.asm:11622; begin_attract_graphics_upload.
- region-811: physical$0031B8..$003257, game.asm:11636; upload_attract_display_dma.
- region-66: physical$02E4E0..$02E864, game.asm:18639; licensed_splash_tilemap.
- region-68: physical$02E864..$02EB72, game.asm:18648; software_splash_tilemap.
- region-853: physical$040000..$044D00, game.asm:19668; attract_display_graphics.
- region-854: physical$044D00..$045300, game.asm:19674; attract_display_tilemap.
- region-855: physical$045340..$045440, game.asm:19683; attract_display_palette.
