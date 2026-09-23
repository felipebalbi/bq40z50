# BQ40Z50 driver — TRM conformance audit

**Audit base commit:** `67132680` ("Add cargo-vet supply-chain security (#58)")
**Branch:** `review/trm-conformance-audit`
**Scope:** conformance of `device_r{1,3,4,5}.yaml`, `src/interface.rs`, `src/common.rs`,
`src/consts.rs` and `src/versions/r{1,3,4,5}.rs` against the four Technical Reference
Manuals. The generated `src/versions/gen_r*.rs` files were not opened; they are treated
as a faithful rendering of the manifests.

## How to use this document

**The finding tests are a RED baseline. They are expected to FAIL.** Every test added by
this audit asserts the behaviour a TRM requires, so every failure is one real defect, and
each test going green is one bug fixed. Nothing is `#[ignore]`d — an ignored test is an
invisible test.

No driver code was changed by this audit. `src/interface.rs`, `src/common.rs`,
`src/consts.rs`, `src/versions/*.rs` and the YAML manifests are untouched; the only edit
outside `src/tests.rs` is this file.

Run them:

```
cargo test --features r5                  # also: r1, r3, r4, "r5,embassy-timeout"
cargo test --features r5 finding_         # just the audit tests
cargo test --features r5 finding_c1_operation_status_emshut_belongs_at_bit6   # one by name
```

Expected on `67132680` (pre-existing tests in the *passed* column, findings in the
*failed* column, nothing ignored anywhere):

| feature set | passed | failed | ignored |
|---|---|---|---|
| `r1` | 16 | 12 | 0 |
| `r3` | 17 | 19 | 0 |
| `r4` | 19 | 19 | 0 |
| `r5` | 19 | 19 | 0 |
| `r5,embassy-timeout` | 19 | 19 | 0 |

`r1` runs seven fewer finding tests: the six C1 tests and the M2 test are `cfg`'d off
because the driver is genuinely **correct** for R1 on those points (see C1 and M2).

### Keeping the tests compilable while red

A test that names an accessor or signature which does not exist yet is a compile error,
and a compile error takes the whole suite down. Each test therefore states which pattern
it uses:

| Pattern | Situation | Consequence for the test |
|---|---|---|
| **A** | The accessor survives the fix; only its bit or wire encoding moves. | Needs no edit. Goes green automatically. |
| **B** | The accessor is renamed or deleted by the fix, so the new name cannot be written down yet. The test asserts that the *currently wrong* accessor does **not** report the bit. | Stops compiling after the fix. Every Pattern B test carries a `WHEN FIXED:` line saying exactly what to replace it with. |
| **C** | The defect is a **missing** field, so there is no method to name. | Not testable. Routed to this document only. |
| **D** | The fix changes a function signature. The call site uses today's signature; the assertion is already TRM-correct. | Carries a `WHEN FIXED:` line naming the new signature. |
| **E** | The driver panics where it should return an error. | Fails by panicking inside the driver, before the test's own assertion is reached. The panic *is* the finding. |

## Specifications

| Literature no. | Title | Applies to |
|---|---|---|
| SLUUA43A | bq40z50 Technical Reference | `r1` |
| SLUUBU5A | bq40z50-R3 Technical Reference | `r3` |
| SLUUCH2  | BQ40Z50-R4 Technical Reference Manual | `r4` |
| SLUUCN4B | BQ40Z50-R5 Technical Reference Manual | `r5` |

The datasheet (SLUSBS8B) contains no register table; it defers to the TRMs, so every
citation below is to a TRM.

---

## Summary

| ID | Sev | Title | Test (RED) / untestable | Affects |
|---|---|---|---|---|
| C1a-1 | Critical | OperationStatus bit 29 is `DISCONN`, not `EMSHUT` | `finding_c1_operation_status_bit29_is_disconn_not_emshut` (B) | r3, r4, r5 |
| C1a-2 | Critical | OperationStatus bit 26 is `STORAGEM`/`VLB`, not `SLPAD` | `finding_c1_operation_status_bit26_is_not_slpad` (B) | r3, r4, r5 |
| C1a-3 | Critical | The real `EMSHUT` is at OperationStatus bit 6 and is unreachable | `finding_c1_operation_status_emshut_belongs_at_bit6` (A) | r3, r4, r5 |
| C1a-4 | Critical | PFStatus bit 25 is `FORCE` (manual PF), not `OPNCELL` | `finding_c1_pf_status_bit25_is_force_not_opncell` (B) | r3, r4, r5 |
| C1a-5 | Critical | PFAlert bit 25 is Reserved, exposed as `OPNC` | `finding_c1_pf_alert_bit25_is_reserved` (B) | r3, r4, r5 |
| C1a-6 | Critical | SafetyStatus bit 19 is Reserved, exposed as `PTOS` (SafetyAlert is fine) | `finding_c1_safety_status_bit19_is_reserved_but_safety_alert_bit19_is_ptos` (B) | r3, r4, r5 |
| C1b | Critical | ~25 TRM-defined protection bits have no accessor at all | untestable (Pattern C) | r3, r4, r5 |
| C2 | Critical | PEC accumulator reused across retries — bad CRC accepted | `finding_c2_pec_must_be_recomputed_on_every_retry` (A) | all |
| C3 | Critical | DF PEC retry silently returns the *next* block | `finding_c3_df_pec_retry_must_resend_the_starting_address` (A) | all |
| C4 | Critical | `MAC_STOP_OUTPUT_CCADC_CAL` sends the *enable* subcommand | `finding_c4_stop_ccadc_cal_must_send_subcommand_f080` (B) | all |
| C5 | Critical | `AsyncBufferInterface::write` transmits 30 bytes of stack padding | `finding_c5_buffer_write_must_emit_only_the_payload` (A) | all |
| C6 | Critical | MAC command echo never validated | `finding_c6_mismatched_mac_command_echo_must_be_an_error` (A) | all |
| M1 | Major | `write_authentication_key` emits a self-inconsistent 22-byte frame | `finding_m1_auth_key_frame_must_be_16_key_bytes` (D) | all |
| M2 | Major | One `SECURITY_KEYS_DATA_LEN_BYTES` shared by four revisions that disagree | `finding_m2_security_keys_block_must_match_the_revision` (D) | r3, r4, r5 |
| M3 | Major | `LIFETIME_DATA_BLOCK_1` discharge fields declared unsigned | untestable | all |
| M4 | Major | `set_battery_mode` caches the unit before the write, no rollback | `finding_m4_failed_battery_mode_write_must_not_change_the_cached_unit` (A) | all |
| M5 | Major | `set_remaining_capacity_alarm` / `set_at_rate` discard the unit tag | `finding_m5_capacity_alarm_must_reject_a_mismatched_unit` (A) | all |
| M6 | Major | Read-only registers generated as read-write | untestable | r3, r4, r5 |
| M7 | Major | Oversized slices panic instead of returning `DataTooLarge` | `finding_m7_oversized_buffer_write_must_return_data_too_large` (E) | all |
| M8a | Major | DF address never bounds-checked against 0x4000–0x5FFF | `finding_m8_df_address_outside_the_window_must_be_rejected` (A) | all |
| M8b | Major | DF per-chunk address can overflow u16 | `finding_m8_df_address_overflow_must_be_rejected_not_panic` (E) | all |
| M9 | Major | `MAC_CHEM_ID` reads only one byte of a 16-bit ID | `finding_m9_chem_id_must_be_16_bits` (A) | all |
| M10 | Major | CI never runs the test suite; two further CI defects | untestable | all |
| Md1–Md8 | Medium | see below | untestable | varies |
| Mn1–Mn5 | Minor | see below | untestable | all |
| — | — | Missing commands (GPIORead, StorageMode, …) | untestable | varies |

---

# Critical

## C1 — Protection and safety registers were never re-derived per revision

`SAFETY_ALERT`, `SAFETY_STATUS`, `PF_ALERT`, `PF_STATUS` and `OPERATION_STATUS` are
**byte-identical across all four manifests, at the same line numbers** (217, 292, 373,
448, 532). Verified by hashing `device_r{1,3,4,5}.yaml` lines 217–611: all four produce
the same digest. They were copied from the R1 manifest and never revisited. TI remapped
several of these bits at R3. The R1 manifest is correct; `r3`, `r4` and `r5` inherit a
register map that belongs to different silicon.

The practical consequence is that the driver reports **the wrong fault**, and cannot
report several real ones at all.

**File:** `device_r5.yaml:217`, `:292`, `:373`, `:448`, `:532` (and the identical blocks
in `device_r3.yaml` / `device_r4.yaml`).
**Affects:** `r3`, `r4`, `r5`. Not `r1`.

### C1a — Mislabelled bits

| Register | Bit | Manifest says | TRM says | Citation |
|---|---|---|---|---|
| OperationStatus | 29 | `EMSHUT` (`device_r5.yaml:624`) | `DISCONN` | SLUUCN4B §16.1.41 *ManufacturerAccess() 0x0054 OperationStatus*: "DISCONN (Bit 29): System disconnect" |
| OperationStatus | 26 | `SLPAD` (`device_r5.yaml:615`) | `STORAGEM` (R5) / `VLB` (R4) | SLUUCN4B §16.1.41: "STORAGEM (Bit 26): Storage Mode is triggered via command"; SLUUCH2 §16.1.40 *ManufacturerAccess() 0x0054 OperationStatus*: "VLB (Bit 26): Very low battery warning" |
| OperationStatus | 6 | *(no accessor)* | `EMSHUT` | SLUUCN4B §16.1.41: "EMSHUT (Bit 6): Emergency FET Shutdown" |
| PFStatus | 25 | `OPNCELL` (`device_r5.yaml:515`) | `FORCE` | SLUUCN4B §16.1.40 *ManufacturerAccess() 0x0053 PFStatus*: "FORCE (Bit 25): Manual PF" |
| PFAlert | 25 | `OPNC` (`device_r5.yaml:434`) | Reserved | SLUUCN4B §16.1.39 *ManufacturerAccess() 0x0052 PFAlert*: "RSVD (Bits 26–23): Reserved. Do not use." |
| SafetyStatus | 19 | `PTOS` (`device_r5.yaml:350`) | Reserved | SLUUCN4B §16.1.38 *ManufacturerAccess() 0x0051 SafetyStatus*: "RSVD (Bit 19): Reserved. Do not use." |

All the wrong names are correct for R1 — SLUUA43A §12.1.40 *OperationStatus* ("EMSHUT
(Bit 29): Emergency Shutdown", "SLPAD (Bit 26): ADC Measurement in SLEEP mode"), §12.1.39
*PFStatus* ("OPNCELL (Bit 25): Open Cell Tab Connection Failure"), §12.1.38 *PFAlert*
("OPNC (Bit 25): Open Cell Tab Connection Failure"), §12.1.36 *SafetyAlert* ("PTOS (Bit
19): Precharge Timeout Suspend") — which is how the wrong map survived.

The PFStatus bit 25 case is the most dangerous: a permanent fail the host itself
requested (`FORCE`, manual PF) is reported as an open cell-tab hardware failure.

Note the double error on `EMSHUT`: the manifest puts the name at bit 29 where R5 has
`DISCONN`, and the real `EMSHUT` at bit 6 has no accessor at all.

> **Correction to the original finding list:** the `PTOS` complaint applies to
> **SafetyStatus only**. SLUUCN4B §16.1.37 *ManufacturerAccess() 0x0050 SafetyAlert*
> does define "PTOS (Bit 19): Precharge timeout suspend" and "CTOS (Bit 21): Charge
> timeout suspend", so the `SAFETY_ALERT` block is **right** about those two bits on R5
> and must not be changed. `finding_c1_safety_status_bit19_is_reserved_but_safety_alert_bit19_is_ptos`
> contains a **positive control** asserting `SafetyAlert::ptos()` is still true, so a
> SafetyStatus fix cannot be over-applied to SafetyAlert unnoticed.

**Tests (RED), all `#[cfg(not(feature = "r1"))]`:**

| Bit | Test | Pattern | When fixed |
|---|---|---|---|
| OperationStatus 29 | `finding_c1_operation_status_bit29_is_disconn_not_emshut` | B | delete, replace with `disconn() == true` for bit 29 |
| OperationStatus 26 | `finding_c1_operation_status_bit26_is_not_slpad` | B | delete, replace with `storagem() == true` (r5) / `vlb() == true` (r4) |
| OperationStatus 6 | `finding_c1_operation_status_emshut_belongs_at_bit6` | **A** | no edit needed |
| PFStatus 25 | `finding_c1_pf_status_bit25_is_force_not_opncell` | B | delete, replace with `force() == true` for bit 25 |
| PFAlert 25 | `finding_c1_pf_alert_bit25_is_reserved` | B | delete; bit 25 should have no accessor |
| SafetyStatus 19 | `finding_c1_safety_status_bit19_is_reserved_but_safety_alert_bit19_is_ptos` | B | keep the SafetyAlert control, delete the SafetyStatus assertion |

### C1b — Missing bits

Bits defined by SLUUCN4B that have no accessor in the generated driver. All verified
against the manifest field tables and against the TRM bit lists.

| Register | Missing bits | Citation (SLUUCN4B) |
|---|---|---|
| SafetyStatus | `OCDL` (29), `COVL` (28), `HWDF` (17), `DCOT` (15) | §16.1.38: "OCDL (Bit 29): Overcurrent in discharge"; "COVL (Bit 28): Cell overvoltage latch"; "HWDF (Bit 17): SBS Host watchdog timeout"; "DCOT (Bit 15): Delta cell overtemperature" |
| SafetyAlert | `OCDL` (29), `COVL` (28), `DCOT` (15) | §16.1.37, same bit names |
| PFStatus | `TMPC` (27), `OCDL` (18), `ASCDL` (15), `ASCCL` (14), `AOLDL` (13), `COVL` (5) | §16.1.40: "TMPC (Bit 27): TMP468 Communication Failure"; "OCDL (Bit 18): Overcurrent in discharge"; "ASCDL (Bit 15): Short circuit in discharge"; "ASCCL (Bit 14): Short circuit in charge"; "AOLDL (Bit 13): Overload in discharge"; "COVL (Bit 5): Cell overvoltage latch" |
| PFAlert | `TMPC` (27), `DFW` (26), `IFC` (24), `PTC` (23), `OCDL` (18), `ASCDL` (15), `ASCCL` (14), `AOLDL` (13), `COVL` (5) | §16.1.39, same bit names |
| OperationStatus | `IOSHUT` (31), `PSSHUT` (30), `EMSHUT` (6), `ACTHR` (4) | §16.1.41: "IOSHUT (Bit 31): IO-based shutdown"; "PSSHUT (Bit 30): Power saving shutdown"; "EMSHUT (Bit 6): Emergency FET Shutdown"; "ACTHR (Bit 4): Accumulated charge threshold" |
| GaugingStatus | `VLB` (21), `OCVPRED` (14) | §16.1.43 *ManufacturerAccess() 0x0056 GaugingStatus*: "VLB (Bit 21): Very low battery warning"; "OCVPRED (Bit 14): Open-circuit-voltage predicted" |

**untestable (Pattern C):** a missing bit produces a *missing method*. A test that calls
`status.ioshut()` is a compile error today, which would take the whole suite down and
give the reader nothing; and there is no way to assert from inside the crate that a method
does not exist. **These become testable in the same commit that adds the fields** — the
table above lists the exact register/bit pairs so that work is fully specified.

The one exception is `EMSHUT` at bit 6, which *is* testable today because the accessor
name already exists (at the wrong bit) — see `finding_c1_operation_status_emshut_belongs_at_bit6`.

## C2 — PEC accumulator is reused across retries

`smbus_pec::Pec` is a `core::hash::Hasher`; `finish()` does not reset the internal
state. The accumulator is constructed **outside** the retry loop in `read_with_retries`,
so the second attempt's CRC continues from the first attempt's residue rather than
restarting from the address/command preamble. A frame carrying the wrong PEC is
therefore accepted on retry — which defeats the entire purpose of PEC.

Concretely, for a 2-byte read of register `0x16` returning `40 00`, the only correct PEC
is CRC-8 over `[0x16, 0x16, 0x17, 0x40, 0x00]` = `0x85`, on *every* attempt. The test
feeds `0xAC` four times; the driver rejects it on attempt 1 and accepts it on attempt 2.

**File:** `src/interface.rs:149` (accumulator hoisted above the `loop` at `:166`) and the
`embassy-timeout` copy at `src/interface.rs:524` / `:541`.
**Spec:** SMBus PEC is CRC-8, poly `0x07`, init `0x00`. SLUUCN4B and SLUSBS8B delegate the
definition to the SMBus specification at smbus.org and never restate it, so there is no
TRM sentence to quote here; the invariant is simply that the CRC is per-transaction.
**Affects:** all revisions.
**Status: already addressed by open PR #61.** This test should go green the moment #61
merges, and then stands as its regression guard.
**Test (RED):** `finding_c2_pec_must_be_recomputed_on_every_retry` (Pattern A).

## C3 — Data-flash PEC retry silently returns the next block

When a DF block read fails its PEC check, the retry path `continue`s the **inner chunk
loop** without re-sending the starting address. Because the gauge auto-increments its DF
read pointer, the retry reads the *next* 32-byte block, and the driver returns that block
to the caller as if it were the one requested. A CRC failure is thus converted into
silent data corruption.

**File:** `src/interface.rs:441-447`, and the `embassy-timeout` copy at
`src/interface.rs:859-866`.
**Spec:** SLUUCN4B §16.1.101 *ManufacturerAccess() 0x4000–0x5FFF DataFlashAccess*: "The
gauge supports an auto-increment on the address during a DF read. […] If another SMBus
read block is sent with command 0x44, the gauge returns another 32 bytes of DF data,
starting with address 0x4020."
**Affects:** all revisions (`pec_read: true` only).
**Test (RED):** `finding_c3_df_pec_retry_must_resend_the_starting_address` (Pattern A).
The mock encodes the conforming sequence — set address, bad block, **set address again**,
good block. Today the driver skips the re-send and issues a bare read, so the failure
surfaces as `i2c::write_read unexpected mode / left: Write / right: WriteRead`.

## C4 — `MAC_STOP_OUTPUT_CCADC_CAL` sends the *enable* subcommand

`MAC_STOP_OUTPUT_CCADC_CAL` and `MAC_OUTPUT_CCADC_CAL` are both declared at address
`0x4481F0`, i.e. MAC subcommand `0xF081`. `0xF081` starts raw ADC output. The command
that stops it is `0xF080`. `allow_address_overlap: true` suppressed the duplicate-address
check that would have caught this. The identical mistake applies to
`MAC_STOP_OUTPUT_SHORTED_CCADC_CAL`, declared at `0x4482F0` (= `0xF082`, the
shorted-input *enable*).

A caller trying to leave calibration mode re-enters it.

**File:** `device_r5.yaml:2376-2384` and `:2440-2448`; `device_r1.yaml:1582`,
`device_r3.yaml:1880`, `device_r4.yaml:2290`.
**Spec:** SLUUCN4B §14.2 *Calibration*, ManufacturerAccess() table: "0xF080  Disables raw
ADC data output on ManufacturerData()" / "0xF081  Outputs raw ADC data of voltage,
current, and temperature on ManufacturerData()". The same table appears in SLUUA43A
§12.1.61 *ManufacturerAccess() 0xF080 Exit Calibration Output Mode* and §12.1.62
*ManufacturerAccess() 0xF081 Output CCADC Cal*, SLUUBU5A §15.1.84 and SLUUCH2 §16.1.99
*ManufacturerAccess() 0xF080 and 0xF081 Output CCADCCal Control*.
**Affects:** all revisions.
**Recommended fix — deletion, not re-addressing.** `0xF080` is *already* reachable under
another name: `MAC_EXIT_CALIBRATION_OUTPUT_MODE` at `0x4480F0`. The two `MAC_STOP_*`
entries are therefore redundant as well as wrong, and are better deleted than moved.
**Test (RED):** `finding_c4_stop_ccadc_cal_must_send_subcommand_f080` (Pattern B).
**When fixed:** if the two entries are **deleted**, delete this test — its second half
(`mac_exit_calibration_output_mode()`) already covers the surviving command. If they are
instead **re-addressed** to `0x4480F0`, the test goes green unchanged.

## C5 — `AsyncBufferInterface::write` transmits the whole backing array

`write` copies the caller's buffer into a 34-byte stack array and then passes `&data` —
the whole array — to `write_with_retries`, instead of `&data[..=buf.len()]`. A 3-byte
buffer write puts 30 bytes of zero padding on the wire after the payload. The return
value still reports the caller's length, so the padding is invisible from the API.

Compare the sibling `write_register` at `src/interface.rs:902`, which correctly slices
`&buf[..=data.len()]`.

**File:** `src/interface.rs:977`.
**Spec:** the SMBus block protocol is length-prefixed and no SLUUCN4B register accepts
trailing padding; there is no sentence to quote because no TRM contemplates this. The
defect is self-evident from the diff against `write_register`.
**Affects:** all revisions.
**Test (RED):** `finding_c5_buffer_write_must_emit_only_the_payload` (Pattern A).

## C6 — MAC responses are never validated against the request

`mac_read_with_retries` skips exactly three bytes — the length byte and the two-byte
command echo — and copies the remainder out. It never compares the echoed command to the
command it sent, and never checks the length byte against the expected payload size. A
stale or misaddressed response is indistinguishable from a correct one. `BQ40Z50Error`
has no variant to report such a mismatch, so fixing this needs a new variant too.

**File:** `src/interface.rs:284-287`, and the `embassy-timeout` copy at
`src/interface.rs:676-679`.
**Spec:** SLUUCN4B §16.1 *0x00 ManufacturerAccess() and 0x44 ManufacturerBlockAccess()*:
"SMBus block read. Command = 0x44. Data read = 06 00 00 01 […] The first 2 bytes, '06
00', is the MAC command." The echo exists so the host can confirm which command it is
looking at.
**Affects:** all revisions.
**Test (RED):** `finding_c6_mismatched_mac_command_echo_must_be_an_error` (Pattern A).
The assertion is deliberately only `is_err()` so that it stays valid whatever the new
error variant is called. **When fixed:** tighten it to match the new variant.

---

# Major

## M1 — `write_authentication_key` takes 18 bytes and emits a self-inconsistent frame

The function takes `&[u8; AUTH_KEY_LEN_BYTES]` = `&[u8; 18]` and writes a length byte of
18, but then emits two command bytes plus eighteen key bytes — twenty bytes after the
length byte. The frame contradicts itself *and* the key is the wrong size. The sibling
`read_authentication_key` gets it right, using `AUTH_KEY_DATA_LEN_BYTES` (16).

Correct frame: `44 12 37 00` followed by **16** key bytes (`0x12` = 18 = 2 command bytes
+ 16 key bytes), 20 bytes total. Today 22 bytes go on the wire.

**File:** `src/versions/r5.rs:137-146` and the identical bodies in `r1.rs` / `r3.rs` /
`r4.rs`; `src/consts.rs:16-18`.
**Spec:** SLUUCN4B §16.1.35 *ManufacturerAccess() 0x0037 Authentication Key*: "Send the
AuthenticationKey() + the new 128-bit authentication key to ManufacturerBlockAccess()".
128 bits = 16 bytes. SLUUA43A §12.1.34, SLUUBU5A §15.1.34 and SLUUCH2 §16.1.34 carry the
same wording.
**Affects:** all revisions.
**Test (RED):** `finding_m1_auth_key_frame_must_be_16_key_bytes` (Pattern D).
**When fixed:** the signature becomes `&[u8; 16]`; change the argument at the call site to
`&[0xAA; 16]`. The expected frame is already correct and needs no change.

## M2 — One `SECURITY_KEYS_DATA_LEN_BYTES` shared by four revisions that disagree

`SECURITY_KEYS_DATA_LEN_BYTES` is hard-coded to 8 for every revision. The Security Keys
block grew at every revision:

| Revision | Keys | Key bytes | Citation |
|---|---|---|---|
| R1 | UNSEAL, FULL ACCESS | **8** | SLUUA43A §12.1.33 *ManufacturerAccess() 0x0035 Security Keys*: "Data = MAC command + New UNSEAL key + New FULL ACCESS KEY = 35 00 34 12 78 56 FF FF FF FF" |
| R3 | + Manual PF, Lifetimes Reset | **16** | SLUUBU5A §15.1.33: "= 35 00 34 12 78 56 FF FF FF FF 57 28 98 2A 14 2B 8A 2C" |
| R4 | + DF Read Only, Override | **24** | SLUUCH2 §16.1.33: "= 35 00 34 12 78 56 FF FF FF FF 32 76 12 17 57 28 98 2A 14 2B 8A 2C 18 2C 9B 2E" |
| R5 | + MfgInfoC Write | **28** | SLUUCN4B §16.1.34: "= 35 00 34 12 78 56 FF FF FF FF 32 76 12 17 57 28 98 2A 14 2B 8A 2C 18 2D 9B 2E 45 3C 89 5D" |

> **Correction to the original finding list.** The finding was stated as unqualified
> ("TRM block is 14 words = 28 bytes"). Re-derived against all four TRMs it is
> **revision-dependent**: 28 bytes is correct for R5 only, and **the driver's 8 bytes is
> correct for R1**. The defect is therefore *one shared constant for four revisions that
> disagree* — on R3 half the keys are unreachable, on R4 two thirds, on R5 just over two
> thirds. The test is `#[cfg(not(feature = "r1"))]` so r1 is not marked buggy.
>
> **`read_security_keys` has the identical defect** (`src/versions/r5.rs:64`, reading 8
> bytes into a `&mut [u8; 8]`). It is not separately tested because the same constant fix
> covers both; the fix must update the reader too.

**File:** `src/consts.rs:13`, consumed by `src/versions/r5.rs:88-103` (write) and `:64-80`
(read), and the r3/r4 equivalents.
**Affects:** `r3`, `r4`, `r5`. Not `r1`.
**Test (RED):** `finding_m2_security_keys_block_must_match_the_revision` (Pattern D). The
expected key length is selected by `cfg`: 16 on r3, 24 on r4, 28 on r5.
**When fixed:** `SECURITY_KEYS_DATA_LEN_BYTES` becomes revision-specific; change the
argument to `&[0xAA; KEY_BYTES]`. The expectation needs no change.

## M3 — `LIFETIME_DATA_BLOCK_1` discharge fields are declared unsigned

`MAX_DISCHARGE_A`, `MAX_AVG_DISCHARGE_A` and `MAX_AVG_DISCHARGE_PWR` are declared
`base: uint`. The TRM types all three as `I2` with range −32768..0 — they are *always
negative or zero*. As unsigned, a recorded −1000 mA peak reads back as 64536.

**File:** `device_r5.yaml:4130`, `:4135`, `:4140` (equivalent lines in r1/r3/r4).
**Spec:** SLUUCN4B §17.17 *Data Flash Summary*, Lifetimes/Current rows:
`0x43D4 Max Discharge Current  I2  −32768  0  0  mA`,
`0x43D6 Max Avg Dsg Current  I2  −32768  0  0  mA`,
`0x43D8 Max Avg Dsg Power  I2  −32768  0  0  cW`.
(Contrast `0x43D2 Max Charge Current  I2  0  32767`, which genuinely is non-negative.)
**Affects:** all revisions.
**untestable:** the accessor's return type is generated directly from `base: uint`, so a
bus-level test can only feed `0xFC18` in and assert `64536` out — which asserts the
manifest against itself, not against SLUUCN4B. There is no signed value the driver could
return today, and writing a test that names an `i16` accessor would not compile. The fix
is a one-word manifest edit (`uint` → `int`), verified by reading §17.17.

## M4 — `set_battery_mode` caches the capacity unit before the write, with no rollback

`self.set_capacity_mode_state(flags)` runs *before* `write_async`, and there is no
rollback on failure. After a write that was NAK'd four times, the cache says centiwatts
while the part is still in milliamps, and every subsequent `remaining_capacity()`,
`full_charge_capacity()`, `design_capacity()` and `at_rate()` is tagged with the wrong
unit. Separately, both constructors assume `CapacityModeState::Milliamps` without ever
reading `BatteryMode()` from the part.

**File:** `src/common.rs:156-162`; constructors at `src/versions/r5.rs:22-34`.
**Spec:** SLUUCN4B §16.4 *0x03 BatteryMode()* — the reporting unit is whatever is latched
in `CAPACITY_MODE`. A failed write latches nothing.
**Affects:** all revisions.
**Test (RED):** `finding_m4_failed_battery_mode_write_must_not_change_the_cached_unit`
(Pattern A).

## M5 — `set_remaining_capacity_alarm` and `set_at_rate` discard the unit tag

Both collapse the two `CapacityModeValue` / `CapacityModeSignedValue` variants into a
single match arm and write the raw number:

```rust
MilliAmpUnsigned(value) | CentiWattUnsigned(value) => value
```

A caller passing centiwatt-hours while the gauge is in milliamp-hour mode programs an
mAh alarm with a cWh magnitude and gets no error. The enum exists precisely to carry
that distinction. The driver cannot convert cWh to mAh (that needs the pack voltage), so
the only sound behaviour is to reject the mismatch — which is what the test asserts.

**File:** `src/common.rs:128-131` and `:179-182`.
**Spec:** SLUUCN4B §16.2 *0x01 RemainingCapacityAlarm()* and §16.5 *0x04 AtRate()* — the
register unit is selected by `BatteryMode()[CAPACITY_MODE]` (§16.4), not by the caller.
**Affects:** all revisions.
**Test (RED):** `finding_m5_capacity_alarm_must_reject_a_mismatched_unit` (Pattern A).
The mock is pre-loaded with the frame the driver wrongly emits so the failure surfaces as
this test's own assertion rather than as a mock panic; `done()` is deliberately not called
because a conforming driver leaves that expectation unconsumed. `set_at_rate` has the
identical defect and is covered by the same fix.

## M6 — Read-only registers are generated as read-write

`LIFETIME_DATA_BLOCK_6`, `_7`, `_8`, `_11`, `_12` and `GAUGE_STATUS_2` omit the
register-level `access:` key. device-driver 1.0.9 defaults to read-write, so `write_async`
is generated for registers TI documents as read-only. Compare `LIFETIME_DATA_BLOCK_2` at
`device_r5.yaml:4145`, which correctly carries `access: RO`.

The result is incoherent: every *field* in these blocks carries `access: RO` (there are
no setters), but the *register* is writable, so the only thing a caller can do is zero
the entire block.

**File:** `device_r5.yaml:4368`, `:4414`, `:4460`, `:4532`, `:4578`, `:4870`.
**Spec:** SLUUCN4B §16.1 Table 16-1 *ManufacturerAccess() Command List* types these `R`
("0x0065  LifetimeDataBlock6  R  Block"), and the SBS sections §16.65 *0x65
LifetimeDataBlock6()* / §16.76 *0x74 GaugeStatus2()* describe them as returning data
only; lifetime data is cleared by the Lifetimes Reset MAC key (§16.1.34, "First word of
the Lifetimes Reset key"), not by writing the block back.
**Affects:** `r3`, `r4`, `r5` (`LIFETIME_DATA_BLOCK_6` does not exist in `device_r1.yaml`).
**untestable:** the defect is that a method **exists**. Asserting the *absence* of
`lifetime_data_block_6().write_async(..)` is a compile-time property and is not
expressible from inside the crate — a test that calls it passes today (which is exactly
the green-baseline trap this audit is avoiding) and stops compiling after the fix.
Verify this one by reading Table 16-1 and diffing against `LIFETIME_DATA_BLOCK_2`.

## M7 — Oversized slices panic instead of returning `DataTooLarge`

Public trait entry points guard length with `debug_assert!`, which is compiled out in
release, and then index past a fixed backing array. On a `no_std` target a
caller-supplied length becomes a panic (debug) or an out-of-bounds slice (release) rather
than an error. The `BQ40Z50Error::DataTooLarge` variant already exists and is used
correctly by `write_mfg_info_c` (`src/versions/r5.rs:288`).

**File:** `src/interface.rs:892` (`write_register`), `:912` (`read_register`), `:948`
(`dispatch_command`), `:970` (`AsyncBufferInterface::write`).
**Spec:** no TRM sentence applies; this is an API-robustness defect, judged against the
crate's own existing error variant.
**Affects:** all revisions.
**Test (RED):** `finding_m7_oversized_buffer_write_must_return_data_too_large`
(Pattern E). It asserts `Err(BQ40Z50Error::DataTooLarge)` and fails today by panicking at
`src/interface.rs:970` with "Buffer size too big" — *before* the assertion is reached.
That panic is the finding: a caller-supplied length must not be able to abort the
firmware. (Caveat: because the panic pre-empts the assertion, the explanatory message
lives in the test's doc comment and in this entry, not in the failure output.)

## M8 — Data-flash address is never bounds-checked and can overflow

The DF starting address is never validated against the documented `0x4000`–`0x5FFF`
window, and the per-chunk address is computed as a plain
`(starting_address + start_idx as u16)`. An address near the top of the u16 space with a
multi-chunk payload overflows: a panic in debug, a silent wrap to a low address in
release — which on a fuel gauge means writing over whatever DF item lives there. The
driver's own doc comment states the range ("Starting address should be between 0x4000 and
0x5FFF", `src/versions/r5.rs:462`) but nothing enforces it.

**File:** `src/interface.rs:63-66`.
**Spec:** SLUUCN4B §16.1.101 *ManufacturerAccess() 0x4000–0x5FFF DataFlashAccess* — the
section title states the valid range.
**Affects:** all revisions.
**Tests (RED):**

* `finding_m8_df_address_outside_the_window_must_be_rejected` (Pattern A) — a single-chunk
  write at `0x6000`. Clean failure on the test's own assertion, no panic. This is the
  bounds-check half of the finding.
* `finding_m8_df_address_overflow_must_be_rejected_not_panic` (Pattern E) — a two-chunk
  write at `0xFFF0`. Fails today by panicking with "attempt to add with overflow" at
  `src/interface.rs:66`. Same caveat as M7: the panic pre-empts the assertion message.
  Note that chunk 1 reaches the bus *before* the overflow, which also demonstrates Md2.

## M9 — `MAC_CHEM_ID` reads only one byte

`size_bits_out: 8`, so the driver sizes its read at 1 length + 2 echo + 1 data byte and
returns only the low byte of a 16-bit chemistry ID.

**File:** `device_r5.yaml:85`; same line in `device_r1.yaml`, `device_r3.yaml`,
`device_r4.yaml`.
**Spec:** SLUUCN4B §16.1 *0x00 ManufacturerAccess() and 0x44 ManufacturerBlockAccess()*,
worked example: "SMBus block read. Command = 0x44. Data read = 06 00 00 01 […] The second
2 bytes, '00 01', is the chem ID returning in little endian. That is 0x0100, chem ID
100." SLUUA43A §12.1 carries the identical example ("That is 0x0100, chem ID 100").
**Affects:** all revisions.
**Test (RED):** `finding_m9_chem_id_must_be_16_bits` (Pattern A). The mock is the TRM's
own example response, length-prefixed. The driver clocks 4 bytes where the example is 5,
so it fails with `i2c::write_read mismatched response length / left: 4 / right: 5`. The
value assertion is wrapped in `u16::from(...)`, which compiles against both today's `u8`
accessor and the `u16` one the fix produces — so the test needs no edit.

## M10 — CI never runs the test suite

`.github/workflows/check.yml` runs `fmt`, `clippy`, `doc`, `deny`, `msrv` and
`cargo hack --feature-powerset clippy`. None of them is `cargo test`, and no clippy
invocation passes `--all-targets`, so **`src/tests.rs` is not even type-checked in CI**.
The 19 tests on `main` have never been run by the pipeline.

Two secondary defects in the same file:

* **The comma is missing in the second clippy step.** It is written
  `--features embassy-timeout ${{ matrix.chip-rev }}`, which expands to
  `--features embassy-timeout r1` — `r1` is parsed as a **positional argument**, not a
  feature. That job has therefore **never actually exercised `embassy-timeout` together
  with a revision feature**. It needs to be `--features embassy-timeout,${{ matrix.chip-rev }}`.
* **`cargo clippy --all-targets` does not compile on `67132680`.** Verified with the audit
  tests stashed: **7 errors**, denied by `[lints.clippy] pedantic = "deny"` in
  `Cargo.toml`. They include `src/interface.rs:121` (*unneeded late initialization*),
  `src/interface.rs:982` (*unused `async` for async trait impl function with no `.await`
  statements*), and two pre-existing `too many lines` in `src/tests.rs`
  (`test_df_transactions`, `test_df_transactions_pec`). **These must be fixed before
  `--all-targets` can be switched on.** This audit's tests add no new clippy diagnostics:
  the count is still 7 on this branch, on both `r1` and `r5`.

**File:** `.github/workflows/check.yml` (clippy job, lines ~50–75).
**Affects:** all revisions.
**untestable in-crate:** this is a property of the CI configuration, not of any code path
the test harness can reach — a test cannot observe whether GitHub Actions invoked it. Fix
by adding a `test` job running `cargo test --features <rev>` for each of r1/r3/r4/r5 plus
`r5,embassy-timeout`, by fixing the missing comma, and by adding `--all-targets` to the
clippy invocations once the 7 errors above are cleared.

> Note for whoever wires up the `test` job: **on this branch `cargo test` is expected to
> fail.** That is the point. The job should be added in the same commit that fixes the
> findings, or added as non-blocking until then.

---

# Medium

## Md1 — `MAC_GAUGE_STATUS_3` padded to 256 bits on an undocumented claim

`device_r5.yaml:2172` declares 256 bits, while its SBS twin at `device_r5.yaml:4982`
stayed at 192. The padding was justified by an empirical claim about the part returning
more bytes than documented (the same reasoning appears in `src/common.rs:231-234` for
`max_error`), but that claim is not recorded next to the manifest entry, and the same
padding was *not* applied to `DA_STATUS_2`, `CB_STATUS` or `AFE_REG`.

**Spec:** SLUUCN4B §16.1.69 *ManufacturerAccess() 0x0075 GaugeStatus3*: "Action: Output 24
bytes of IT data values on ManufacturerBlockAccess() or ManufacturerData()". 24 bytes =
192 bits.
**untestable:** the discrepancy is between two manifest entries and an unrecorded bench
observation. A test can only pin whichever number the manifest currently holds, and there
is no way to tell from a mock which number the silicon agrees with. What this needs is a
comment in the YAML recording the measurement and its date, and a decision applied
consistently to the other four blocks.

## Md2 — DF writes commit chunks with no rollback

`mac_write_to_df_with_retries` writes 32-byte chunks in a loop and returns on the first
failure, leaving the earlier chunks committed to flash. A caller who retries the whole
write re-writes those chunks; a caller who gives up leaves the DF half-updated.
**File:** `src/interface.rs:56-89`.
**untestable as its own case, but demonstrated:**
`finding_m8_df_address_overflow_must_be_rejected_not_panic` shows chunk 1 reaching the bus
before chunk 2 aborts. A dedicated test would need a definition of "correct" (all-or-
nothing buffering? a partial-write error carrying the committed count?) that this audit
is not in a position to choose.

## Md3 — Non-PEC DF reader short-reads the final partial chunk, PEC reader does not

`mac_read_from_df_with_retries` sizes the final chunk at `n + 3` bytes
(`src/interface.rs:336-339`), NAKing early; `mac_read_from_df_with_retries_pec` always
clocks the full 36 (`src/interface.rs:418`). The asymmetry is deliberate and commented
("For PEC, we need to read in 32 byte chunks"), but it means the two paths put different
traffic on the bus for identical calls, and the non-PEC path relies on the gauge
tolerating an early NAK mid-block — which SLUUCN4B §16.1.101 does not promise.
**untestable against the spec:** both behaviours are consistent with the manifest and with
the existing fixtures (`test_df_transactions` vs `test_df_transactions_pec` already pin
them). Deciding which is correct needs a scope on a real part, not a mock.

## Md4 — ~490 lines of transport logic duplicated between the two `cfg` copies

`src/interface.rs:95-463` (`not(feature = "embassy-timeout")`) and `:465-880`
(`feature = "embassy-timeout"`) are near-identical. C2, C3 and C6 each exist twice
because of it, and any fix must be applied twice. The only difference is the `with_timeout`
wrapper around each `i2c` call.
**untestable:** structural. Note that the `embassy-timeout` copy is only compiled — and
therefore only tested — when that feature is on, so the duplicate is also a coverage
hazard. (The `r5,embassy-timeout` run confirms all 19 findings reproduce on that copy too.)

## Md5 — Several existing tests assert nothing

* `test_battery_status` / `test_battery_status_pec` assert `status.error_code() ==
  ErrorCode::Ok`, which is the zero discriminant — the assertion passes on an all-zero
  buffer and would pass if the read silently returned nothing.
* `test_read_mfg_info_c` / `test_read_mfg_info_c_pec` never inspect `buf` after the call.
* the 128-byte DF fixtures in `test_df_transactions` and `test_df_transactions_pec` return
  the *same* 32-byte block four times, so a driver that ignored the chunk index entirely
  would still pass.
**untestable:** this is a critique of existing tests, not of driver behaviour, and those
19 tests are deliberately left unmodified by this audit so they stay a stable green
reference. Fix by asserting a non-zero, distinguishable payload in each.

## Md6 — `ChargingVoltageOverride` fields are unsigned

`src/common.rs:46-50` declares all five fields `u16`.
**Spec:** SLUUCN4B §16.1 Table 16-1 *ManufacturerAccess() Command List*: "0x00B0
ChargingVoltageOverride  R/W  Block  Yes  -  Yes  Signed Int  mV". (The adjacent row reads
"0x00B2  ChargingCurrentOverride  R/W  Block  Yes  -  Yes  Signed Int  mA".)
**Affects:** `r3`, `r4`, `r5`.
**untestable:** `read_charging_voltage_override` hand-decodes with `u16::from_le_bytes`
(`src/versions/r5.rs:452-456`), so a test would assert the driver's own choice of type back
at itself, and a test naming an `i16` field would not compile. Fix is a type change in
`src/common.rs` plus `i16::from_le_bytes`.

## Md7 — `ManufacturingStatus` bit 15 is named `CAL_TEST`

`device_r5.yaml:854` (and `:3872`) call bit 15 `CAL_TEST`. R3 onwards name it `CAL_EN`.
**Spec:** SLUUCN4B §16.1.44 *ManufacturerAccess() 0x0057 ManufacturingStatus*: "CAL_EN
(Bit 15): CALIBRATION mode". The surrounding prose in §14.2 *Calibration* likewise refers
to `ManufacturingStatus()[CAL_EN]`.
**untestable:** pure rename — the bit position is right, only the identifier is wrong, so
there is no observable difference on the bus and no correct accessor name to call today.

## Md8 — `MfgInfo` / `MfgInfoB` modelled as 4 × u64 LE, which byte-reverses each group

Modelling a 32-byte opaque blob as four little-endian 64-bit integers reverses byte order
within each 8-byte group on decode. Manufacturer info is a byte string, not four integers.
**untestable:** the transformation is applied by generated code from the manifest, so a
test asserts the manifest. Fix by modelling as a byte buffer, as `r1` already does for
`ManufacturerInfo()` 0x70.

---

# Minor

| ID | Finding | File | Spec |
|---|---|---|---|
| Mn1 | The same bit is called `OPNC` in PFAlert and `OPNCELL` in PFStatus | `device_r5.yaml:434`, `:515` | SLUUA43A §12.1.38 / §12.1.39 use both spellings too, so the manifest faithfully reproduces TI's own inconsistency — worth normalising anyway |
| Mn2 | `NO_LOAD_REM_CAP` is missing the `MAC_` prefix used by every other MAC command | `device_r5.yaml:948` | SLUUCN4B §16.1 Table 16-1 *ManufacturerAccess() Command List*: "0x005A  NoLoadRemCap  R  Block" — it is a MAC command |
| Mn3 | `MAC_ALL_DF_SIGNATURE` has its field named `STATIC_CHEM_DF_SIG`, copy-pasted from the neighbouring `MAC_STATIC_CHEM_DF_SIG` | `device_r5.yaml:110` (block at `:92`) | — |
| Mn4 | Trailing space in the key `TURBO_RHF_EFFECTIVE ` | `device_r5.yaml:4513` | — |
| Mn5 | `GAUGE_STATUS_2.DOD0_0` is missing `access: RO` while its siblings have it | `device_r5.yaml:4924` | see M6 |

All five: **untestable** — they are naming and whitespace defects in the manifest with no
observable effect on bus traffic, so any test would compare the manifest to itself and
any test naming the corrected identifier would not compile. Mn3 and Mn4 are the ones most
likely to bite later (a wrong field name, and a key that will not match a `grep`).

---

# Missing features

Commands defined by the TRMs that the driver cannot reach at all.

## SBS commands

| Command | Address | Spec | Note |
|---|---|---|---|
| `GPIORead()` | 0x48 | SLUUCN4B §16.39 *0x48 GPIORead()*: "When the read-only subcommand GPIORead() is sent by the host, the level of the GPIO pins is reflected in the data read back." | absent from all four manifests |
| `GPIOWrite()` | 0x49 | SLUUCN4B §16.40 *0x49 GPIOWrite()*: "When GPIO mode is selected and the write-only subcommand GPIOWrite() is sent by the host…" | This is the **only write-only SBS command**, which is why `access: WO` appears nowhere in any manifest. Adding it will be the first exercise of that access mode. |
| `ManufacturerInfo()` | 0x70 | SLUUCN4B §16.72 *0x70 ManufacturerInfo()* | modelled as a buffer in `device_r1.yaml` only; r3/r4/r5 reach it solely through the hand-written `read_mfg_info` / `write_mfg_info` helpers |

## MAC commands

None of the following appear in `device_r5.yaml` (verified by searching for the encoded
`0x44xxxx` address form):

| Command | MAC subcommand | Spec |
|---|---|---|
| `StorageMode` | 0x000A | SLUUCN4B §16.1.10 *ManufacturerAccess() 0x000A STORAGE Mode* |
| `ChargingStatusExt` | 0x005E | SLUUCN4B §16.1.47 *ManufacturerAccess() 0x005E ChargingStatusExt* |
| `RSOCWrite` | 0x0079 | SLUUCN4B §16.1.73 *ManufacturerAccess() 0x0079 RSOCWrite* |
| `ChargingCurrentOverride` | 0x00B2 | SLUUCN4B §16.1.95 *ManufacturerAccess() 0x00B2 ChargingCurrentOverride* |
| `WriteTemp` | 0x3008 | SLUUCN4B §16.1.100 *ManufacturerAccess() 0x3008 WriteTemp*: "the temperature must be written in 0.1 K"; note it is gated behind a two-word override key sequence |
| `Accumulation*` family | 0x0098–0x009F | SLUUCN4B §16.1 Table 16-1 *ManufacturerAccess() Command List* |
| `IATA*` family | 0x00F0–0x00F2 | SLUUCN4B §16.1 Table 16-1 *ManufacturerAccess() Command List* |
| TMP468 family | 0x0081–0x008B | SLUUCH2 / SLUUCN4B; `r4` and `r5` only |

**untestable (Pattern C):** a command that is not in the manifest generates no method, so
there is nothing to call and a test naming it would not compile. These are feature gaps,
tracked here as a to-do list; each becomes testable in the commit that adds it.

### Correctly gated — not a gap

`CHRG_VOLTAGE_OVERRIDE_CMD` is `cfg`'d out for `r1` (`src/consts.rs:22-25`). This is
correct: the R1 TRM contains **zero** occurrences of `0x00B0`. The R1 silicon does not
have the command.

---

# Known-unverifiable

These could not be resolved from the TRMs and need either TI clarification or bench
measurement.

1. **Signedness of DAStatus1 cell current/power, and the r1 `TURBO*` registers.** The TRMs
   give units but no Type column for these, unlike §17.17 *Data Flash Summary* which does.
   Cannot be decided from the documents.
2. **Whether r1/r3/r4 share the GaugeStatus3 32-byte PEC quirk.** The 256-bit padding in
   the manifests rests on a bench observation on one revision (see Md1). Unknown for the
   others.
3. **PEC preamble conformance.** No TRM defines PEC; all four delegate to the SMBus
   specification at smbus.org. Whether the driver's preamble (`[addr<<1, cmd, addr<<1|1]`)
   matches what the silicon computes for every command class is asserted only by the
   existing fixtures, which were themselves derived from the driver.
4. **TRM self-contradictions.** Recorded here so nobody re-derives them:
   * SLUUCN4B §16.1.66 *ManufacturerAccess() 0x0072 DAStatus2* says "24 bytes" in the
     Action line and then lists 26.
   * `ManufacturingStatus` is typed `H4` in Table 16-1 *ManufacturerAccess() Command List*
     but only 16 bits are mapped in §16.1.44.
   * SLUUCN4B §16.1.37 *SafetyAlert* prose says "OCC2 (Bit 4)" while the bitmap row places
     `OCC2` at bit 3 (and the prose then also gives bit 4 to `OCD1`). **The manifest
     follows the bitmap (bit 3), which is correct** — the prose line is a typo, since bit 4
     is unambiguously `OCD1` in both the bitmap and the R5 SafetyStatus list at §16.1.38.
     No change needed and no test asserts otherwise.
