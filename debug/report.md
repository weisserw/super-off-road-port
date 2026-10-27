# Bounded initial QA

One pass completed the authorized startup and short initial-race smoke, plus the separate initial original-fidelity sanity. No whole-game traversal, desktop window or physical audio device was tested.

The natural native route reaches Names, Shop and Racing; 46 ticks of throttle/steering/nitro move and turn actors without faults. Fresh shop/grid/driven captures are reviewed in `artifacts/smoke-run-003/initial-native-smoke/review.json`.

Six meaningful full 256x224 RGB pairs match exactly: license, software, highscore scroll0132/0142, initial grid and countdown13/4. Original captures/snapshots come from documented natural inputs; setup validates source scene/countdown/pickup parity. Four unaffected frontend pairs remain from sanity-run-005 with explicit producer impact checks, while affected race comparisons were regenerated in sanity-run-007. Native physical ticks differ from original CPU/NMI startup/loading latency; named source state determines alignment, with no pixel search, masking or memory modification.

Real original/native initial audio captures each contain23961 stereo frames at48kHz with zero drops. All8 source voices match sequence/duration/actual DSP pitch/sample/gain at both endpoints30frames apart. OriginalNMI85/native83 alignment follows the source asynchronous upload/command handoff. Captured signal review finds the same269.53Hz dominant component and harmonics, power-spectrum cosine0.962959. Native linear interpolation differs from original Gaussian output in waveform/amplitude (mono RMS0.055332 versus0.051197); this limited difference is documented. No physical playback/listening claim is made. See sanity-run-007 review and audio-review-metrics.json.

Failures and superseded evidence remain in sanity-run-001..006 and smoke-run-001..002. Repairs traced source scanline/BG3 map geometry, separate BG/OAM and HDMA visible-row origins, deterministic cold RNG baseline, prepared pickup parity and the shared16-bit countdown/banner word $077D (delay is independent $077F). Native source-order/row/map contracts verify the repairs. No exact comparison was relaxed, disabled or learned from native output.

The global zero cold RNG/history baseline is an explicit deterministic reference assumption: source boot leaves history unseeded; physical hardware startup state remains unknown. The extra released setup tick aligns prepared pickup parity through natural input only. Prior nonzero policy and mismatch evidence are preserved.

Final administrative check/audit results are recorded in checks/implementation/stage5-final-* and the Stage5 handoff. Passing these records establishes this bounded scope only; implementation source accounting and QA remain separate.

The current native paired-ready checkpoint was exported and imported with normal identity validation at13/4, track4/pickup2. Semantic state and re-exported frame/audio hashes match exactly; see artifacts/ready-checkpoint-current/validation.json. This is replay setup validation of the same bounded recipe, not additional gameplay coverage.

Final required gates PASS: run.py check and run.py audit (all six checks,620 native tests,0failures/0ignored), implementation accounting1527regions/1408units/22uncertainties, and toolkit port_debug.py audit (onepass/twochecks). Exact records are checks/implementation/stage5-final-check.json, stage5-final-audit.json and stage5-final-qa-audit.json.
