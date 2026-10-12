//! ctt-splice-digest — splice the CTT_MODULE_DIGEST static into the guest wasm
//! artifact (W-ABI2 §2 splice pipeline).
//!
//! The guest declares a 32-byte zero-initialized static (`CTT_MODULE_DIGEST`)
//! that lands as a zero run in the wasm data section. This tool:
//!
//! 1. walks the wasm binary (LEB128 section ids/sizes) to the data section,
//! 2. locates the unique maximal 32-byte all-zero run in an active segment
//!    payload (refusing typed when absent or ambiguous),
//! 3. computes `digest = BLAKE3(artifact with slot zeroed)` and writes the 32
//!    digest bytes into the slot in place (length-preserving),
//! 4. `--verify` re-reads the artifact, zeroes the slot, recomputes, and
//!    asserts splice == recompute, length unchanged, digest non-zero.
//!
//! Fail closed: every failure path is a typed error and exit 1.

use std::fmt;
use std::fs;
use std::io::Write as _;
use std::process::ExitCode;

const SLOT_LEN: usize = 32;
const WASM_MAGIC: [u8; 4] = *b"\0asm";
const SECTION_DATA: u8 = 11;

/// Typed failure surface. Every variant is fail-closed: no silent success.
#[derive(Debug)]
enum SpliceError {
    /// CLI usage error (missing artifact path argument).
    Usage,
    /// Artifact file could not be read (missing path, permissions, ...).
    ArtifactRead { path: String, source: std::io::Error },
    /// Artifact file could not be written back in place.
    ArtifactWrite { path: String, source: std::io::Error },
    /// Not a wasm module (bad magic or version).
    NotWasm { reason: String },
    /// Section framing is corrupt (truncated LEB128, section overruns file).
    Unparseable { reason: String },
    /// Data section present but no 32-byte zero run found.
    MissingSlot,
    /// More than one candidate zero run (or a run longer than 32 bytes): the
    /// CTT_MODULE_DIGEST slot cannot be identified uniquely.
    AmbiguousSlot { candidates: usize },
    /// A data segment's offset expression is not a supported const form, so
    /// the segment payload cannot be bounded safely.
    UnsupportedSegment { reason: String },
    /// Artifact changed length across splice (must be length-preserving).
    LengthChanged { before: u64, after: u64 },
    /// The spliced slot is all-zero (digest must be non-zero).
    ZeroDigest,
}

impl fmt::Display for SpliceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => write!(f, "usage: ctt-splice-digest [--verify] <artifact.wasm>"),
            Self::ArtifactRead { path, source } => {
                write!(f, "cannot read artifact {path}: {source}")
            }
            Self::ArtifactWrite { path, source } => {
                write!(f, "cannot write artifact {path}: {source}")
            }
            Self::NotWasm { reason } => write!(f, "not a wasm module: {reason}"),
            Self::Unparseable { reason } => write!(f, "unparseable wasm section: {reason}"),
            Self::MissingSlot => write!(
                f,
                "no {SLOT_LEN}-byte zero slot for CTT_MODULE_DIGEST found in data section"
            ),
            Self::AmbiguousSlot { candidates } => write!(
                f,
                "ambiguous CTT_MODULE_DIGEST slot: {candidates} candidate zero runs in data section"
            ),
            Self::UnsupportedSegment { reason } => {
                write!(f, "unsupported data segment form: {reason}")
            }
            Self::LengthChanged { before, after } => {
                write!(f, "artifact length changed: {before} -> {after}")
            }
            Self::ZeroDigest => write!(f, "spliced digest is all-zero"),
        }
    }
}

/// Reads one unsigned LEB128 value from `bytes` starting at `pos`.
/// Returns `(value, new_pos)` or `None` on truncation/overflow.
fn read_leb128_u32(bytes: &[u8], pos: usize) -> Option<(u32, usize)> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    let mut i = pos;
    loop {
        let b = *bytes.get(i)?;
        i += 1;
        // 5 bytes max for u32 (last byte may only carry 4 bits).
        if shift >= 32 || (shift == 28 && b & 0x70 != 0) {
            return None;
        }
        result |= u64::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            return Some((u32::try_from(result).ok()?, i));
        }
        shift += 7;
    }
}

/// Locates every data section (id 11) body range in the wasm binary.
fn find_data_sections(bytes: &[u8]) -> Result<Vec<(usize, usize)>, SpliceError> {
    if bytes.len() < 8 || bytes[..4] != WASM_MAGIC {
        return Err(SpliceError::NotWasm { reason: "missing \\0asm magic".into() });
    }
    if &bytes[4..8] != &[0x01, 0x00, 0x00, 0x00] {
        return Err(SpliceError::NotWasm { reason: "unsupported wasm version".into() });
    }
    let mut sections = Vec::new();
    let mut pos = 8usize;
    while pos < bytes.len() {
        let (id, after_id) = read_leb128_u32(bytes, pos)
            .ok_or(SpliceError::Unparseable { reason: "truncated section id".into() })?;
        let (size, after_size) = read_leb128_u32(bytes, after_id)
            .ok_or(SpliceError::Unparseable { reason: "truncated section size".into() })?;
        let start = after_size;
        let end = start
            .checked_add(size as usize)
            .filter(|&e| e <= bytes.len())
            .ok_or(SpliceError::Unparseable { reason: format!("section id {id} overruns file") })?;
        if id as u8 == SECTION_DATA {
            sections.push((start, end));
        }
        pos = end;
    }
    Ok(sections)
}

/// Parses active data segments of one data section body, returning
/// `(file_offset_of_payload, payload)` pairs.
fn active_segment_payloads(
    bytes: &[u8],
    body: (usize, usize),
) -> Result<Vec<(usize, &[u8])>, SpliceError> {
    let (mut pos, end) = body;
    let (count, after) = read_leb128_u32(bytes, pos)
        .ok_or(SpliceError::Unparseable { reason: "truncated segment count".into() })?;
    pos = after;
    let mut payloads = Vec::new();
    for seg in 0..count {
        let (flags, after) = read_leb128_u32(bytes, pos).ok_or(SpliceError::Unparseable {
            reason: format!("segment {seg}: truncated flags"),
        })?;
        pos = after;
        // Active with flat memory index (0) or explicit memidx (2). Passive
        // (1) and other forms carry no static slot; skip their bodies safely.
        if flags == 0 {
            // offset expr follows directly
        } else if flags == 2 {
            let (_memidx, after) = read_leb128_u32(bytes, pos).ok_or(SpliceError::Unparseable {
                reason: format!("segment {seg}: truncated memidx"),
            })?;
            pos = after;
        } else {
            // Passive/declarative: payload length-prefixed, not static data.
            let (len, after) = read_leb128_u32(bytes, pos).ok_or(SpliceError::Unparseable {
                reason: format!("segment {seg}: truncated passive length"),
            })?;
            pos = after.checked_add(len as usize).filter(|&p| p <= end).ok_or(
                SpliceError::Unparseable {
                    reason: format!("segment {seg}: passive payload overruns section"),
                },
            )?;
            continue;
        }
        // Minimal const-expr parse: i32.const/i64.const <sleb> end, or
        // global.get <idx>. Anything else is refused (fail closed).
        let opcode = *bytes.get(pos).ok_or(SpliceError::UnsupportedSegment {
            reason: format!("segment {seg}: truncated offset expr"),
        })?;
        pos += 1;
        match opcode {
            0x41 | 0x42 => {
                // signed LEB128 payload — skip it
                let mut i = pos;
                loop {
                    let b = *bytes.get(i).ok_or(SpliceError::UnsupportedSegment {
                        reason: format!("segment {seg}: truncated const operand"),
                    })?;
                    i += 1;
                    if b & 0x80 == 0 {
                        break;
                    }
                }
                pos = i;
            }
            0x23 => {
                let (_, after) =
                    read_leb128_u32(bytes, pos).ok_or(SpliceError::UnsupportedSegment {
                        reason: format!("segment {seg}: truncated global index"),
                    })?;
                pos = after;
            }
            other => {
                return Err(SpliceError::UnsupportedSegment {
                    reason: format!("segment {seg}: offset expr opcode 0x{other:02x}"),
                })
            }
        }
        if *bytes.get(pos).ok_or(SpliceError::UnsupportedSegment {
            reason: format!("segment {seg}: missing expr end"),
        })? != 0x0b
        {
            return Err(SpliceError::UnsupportedSegment {
                reason: format!("segment {seg}: offset expr not terminated by end (0x0b)"),
            });
        }
        pos += 1;
        let (len, after) = read_leb128_u32(bytes, pos).ok_or(SpliceError::Unparseable {
            reason: format!("segment {seg}: truncated data length"),
        })?;
        let payload_start = after;
        let payload_end = payload_start.checked_add(len as usize).filter(|&p| p <= end).ok_or(
            SpliceError::Unparseable { reason: format!("segment {seg}: payload overruns section") },
        )?;
        payloads.push((payload_start, &bytes[payload_start..payload_end]));
        pos = payload_end;
    }
    Ok(payloads)
}

/// Candidate slot: file offset of a maximal all-zero run of exactly
/// `SLOT_LEN` bytes within an active segment payload. Runs of other lengths
/// (including longer than 32) are not candidates — a longer run means the
/// slot boundary is not identifiable.
fn find_slot(bytes: &[u8]) -> Result<usize, SpliceError> {
    let mut candidates: Vec<usize> = Vec::new();
    for body in find_data_sections(bytes)? {
        for (payload_off, payload) in active_segment_payloads(bytes, body)? {
            let mut i = 0usize;
            while i < payload.len() {
                if payload[i] != 0 {
                    i += 1;
                    continue;
                }
                let run_start = i;
                while i < payload.len() && payload[i] == 0 {
                    i += 1;
                }
                if i - run_start == SLOT_LEN {
                    candidates.push(payload_off + run_start);
                }
            }
        }
    }
    match candidates.len() {
        1 => Ok(candidates[0]),
        0 => Err(SpliceError::MissingSlot),
        n => Err(SpliceError::AmbiguousSlot { candidates: n }),
    }
}

fn zeroed_copy(bytes: &[u8], slot: usize) -> Vec<u8> {
    let mut copy = bytes.to_vec();
    copy[slot..slot + SLOT_LEN].fill(0);
    copy
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Splice mode: digest = BLAKE3(artifact with slot zeroed); write digest into
/// the slot in place. Length-preserving by construction (32 bytes over 32).
fn splice(path: &str, bytes: &[u8]) -> Result<String, SpliceError> {
    let slot = find_slot(bytes)?;
    let digest = blake3::hash(&zeroed_copy(bytes, slot));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(false)
        .open(path)
        .map_err(|source| SpliceError::ArtifactWrite { path: path.to_string(), source })?;
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::Start(slot as u64))
        .map_err(|source| SpliceError::ArtifactWrite { path: path.to_string(), source })?;
    file.write_all(digest.as_bytes())
        .map_err(|source| SpliceError::ArtifactWrite { path: path.to_string(), source })?;
    file.flush()
        .map_err(|source| SpliceError::ArtifactWrite { path: path.to_string(), source })?;
    Ok(hex(digest.as_bytes()))
}

/// Verify mode: re-read, locate the spliced slot, zero it, recompute, and
/// compare against the spliced bytes; assert length unchanged and digest
/// non-zero.
///
/// After a splice the 32-byte zero run is gone, so the slot cannot be found
/// by zero-run search. Instead the slot is located self-verifyingly: the
/// unique 32-byte window `w` in an active data-segment payload for which
/// `BLAKE3(artifact with w zeroed) == w` holds. A corrupted or unspliced
/// artifact therefore refuses typed (no matching window) instead of passing.
fn verify(path: &str, spliced_len_before: usize) -> Result<String, SpliceError> {
    let bytes = fs::read(path)
        .map_err(|source| SpliceError::ArtifactRead { path: path.to_string(), source })?;
    if find_slot(&bytes).is_ok() {
        // An exact-32 zero run still present: the artifact was never spliced.
        return Err(SpliceError::ZeroDigest);
    }
    let slot = {
        let mut candidates: Vec<usize> = Vec::new();
        for body in find_data_sections(&bytes)? {
            for (payload_off, payload) in active_segment_payloads(&bytes, body)? {
                if payload.len() < SLOT_LEN {
                    continue;
                }
                for w in 0..=payload.len() - SLOT_LEN {
                    let candidate = payload_off + w;
                    let recomputed = blake3::hash(&zeroed_copy(&bytes, candidate));
                    if recomputed.as_bytes() == &bytes[candidate..candidate + SLOT_LEN] {
                        candidates.push(candidate);
                    }
                }
            }
        }
        match candidates.len() {
            1 => Ok(candidates[0]),
            0 => Err(SpliceError::MissingSlot),
            n => Err(SpliceError::AmbiguousSlot { candidates: n }),
        }
    }?;
    let spliced: [u8; SLOT_LEN] = bytes[slot..slot + SLOT_LEN]
        .try_into()
        .map_err(|_| SpliceError::Unparseable { reason: "slot range out of bounds".into() })?;
    if spliced == [0u8; SLOT_LEN] {
        return Err(SpliceError::ZeroDigest);
    }
    let recomputed = blake3::hash(&zeroed_copy(&bytes, slot));
    if spliced.as_slice() != recomputed.as_bytes() {
        return Err(SpliceError::MissingSlot);
    }
    if bytes.len() != spliced_len_before {
        return Err(SpliceError::LengthChanged {
            before: spliced_len_before as u64,
            after: bytes.len() as u64,
        });
    }
    Ok(hex(recomputed.as_bytes()))
}

fn run() -> Result<(), SpliceError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (verify_mode, path) = match args.as_slice() {
        [p] => (false, p),
        [v, p] if v == "--verify" => (true, p),
        _ => return Err(SpliceError::Usage),
    };
    let bytes = fs::read(path)
        .map_err(|source| SpliceError::ArtifactRead { path: path.clone(), source })?;
    let len_before = bytes.len();
    let mode;
    let digest_hex;
    if verify_mode {
        digest_hex = verify(path, len_before)?;
        mode = "verify";
    } else {
        digest_hex = splice(path, &bytes)?;
        mode = "splice";
    }
    println!(
        "{{\"path\":\"{}\",\"len\":{},\"digest_hex\":\"{}\",\"mode\":\"{}\"}}",
        path, len_before, digest_hex, mode
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-built minimal wasm module: magic + one data section with a single
    /// active segment (memory 0, offset i32.const 0) whose payload is
    /// [0xAA, 0xBB, 32 zeros, 0xCC].
    fn tiny_wasm_with_zero_slot() -> Vec<u8> {
        let mut payload: Vec<u8> = vec![0xAA, 0xBB];
        payload.extend_from_slice(&[0u8; SLOT_LEN]);
        payload.push(0xCC);

        let mut seg = vec![0x00]; // flags: active, flat memidx 0
        seg.extend_from_slice(&[0x41, 0x00, 0x0b]); // i32.const 0, end
        seg.push((payload.len() as u8) & 0x7f); // vector length (1 segment, small payload)
        seg.extend_from_slice(&payload);

        let mut data = Vec::new();
        data.push(0x01); // 1 segment
        data.extend_from_slice(&seg);

        let mut wasm = WASM_MAGIC.to_vec();
        wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        wasm.push(SECTION_DATA);
        wasm.push(data.len() as u8); // section size (small enough for 1-byte LEB128)
        wasm.extend_from_slice(&data);
        wasm
    }

    #[test]
    fn splice_then_verify_roundtrip_on_tiny_wasm() {
        let dir = std::env::temp_dir().join("ctt-splice-digest-test");
        fs::create_dir_all(&dir).unwrap(); // JUSTIFICATION: test code only
        let path = dir.join("tiny.wasm");
        let wasm = tiny_wasm_with_zero_slot();
        fs::write(&path, &wasm).unwrap(); // JUSTIFICATION: test code only

        let path_str = path.to_str().unwrap(); // JUSTIFICATION: test code only
        let digest = splice(path_str, &wasm).unwrap(); // JUSTIFICATION: test code only
        assert_eq!(digest.len(), 64);
        assert_ne!(digest, "0".repeat(64));

        // verify: splice == recompute must hold on the written artifact
        let written = fs::read(&path).unwrap(); // JUSTIFICATION: test code only
        assert_eq!(written.len(), wasm.len(), "splice must be length-preserving");
        let verified = verify(path_str, written.len()).unwrap(); // JUSTIFICATION: test code only
        assert_eq!(digest, verified);

        // verify refuses a zeroed slot
        let zeroed = zeroed_copy(&wasm, 8 + 1 + 1 + 1 + 1 + 1 + 1 + 2 + 2);
        let zpath = dir.join("zeroed.wasm");
        fs::write(&zpath, &zeroed).unwrap(); // JUSTIFICATION: test code only
        match verify(zpath.to_str().unwrap(), zeroed.len()) {
            // JUSTIFICATION: test code only
            Err(SpliceError::ZeroDigest) => {}
            other => panic!("expected ZeroDigest, got {other:?}"),
        }
    }

    #[test]
    fn ambiguous_two_zero_runs_refused() {
        let mut wasm = tiny_wasm_with_zero_slot();
        // Append a second data section holding another exact-32 zero run.
        let mut seg = vec![0x00, 0x41, 0x00, 0x0b, SLOT_LEN as u8];
        seg.extend_from_slice(&[0u8; SLOT_LEN]);
        let mut data = vec![0x01];
        data.extend_from_slice(&seg);
        wasm.push(SECTION_DATA);
        wasm.push(data.len() as u8);
        wasm.extend_from_slice(&data);
        assert!(matches!(find_slot(&wasm), Err(SpliceError::AmbiguousSlot { candidates: 2 })));
    }

    #[test]
    fn no_zero_run_refused() {
        let mut wasm = tiny_wasm_with_zero_slot();
        // Fill the slot run with non-zero bytes.
        let slot_in_file = 8 + 1 + 1 + 1 + 1 + 1 + 1 + 2 + 2;
        for b in &mut wasm[slot_in_file..slot_in_file + SLOT_LEN] {
            *b = 0x7f;
        }
        assert!(matches!(find_slot(&wasm), Err(SpliceError::MissingSlot)));
    }

    #[test]
    fn non_wasm_input_refused() {
        assert!(matches!(find_slot(b"not a wasm"), Err(SpliceError::NotWasm { .. })));
    }
}
