//! SHA-256（FIPS 180-4）。添付のファイルの中身の印（FR-021・ADR 0016）。外部クレートは足さない（ARC-013）。

/// 最初の 64 個の素数の立方根の小数部（FIPS 180-4 の 4.2.2）。
const ROUND_CONSTANTS: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// 最初の 8 個の素数の平方根の小数部（初期値）。
const INITIAL: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// 1 ブロックのバイト数。
const BLOCK: usize = 64;

/// バイト列の SHA-256。
#[must_use]
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut state = INITIAL;
    let (blocks, rest) = bytes.as_chunks::<BLOCK>();
    for block in blocks {
        compress(&mut state, block);
    }
    let (tail, used) = padding(rest, bytes.len());
    for block in tail.iter().take(used) {
        compress(&mut state, block);
    }
    let mut out = [0_u8; 32];
    for (slot, word) in out.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        *slot = word.to_be_bytes();
    }
    out
}

/// 詰め物（FIPS 180-4 の 5.1.1）: 余りに 0x80 を続け、最後の 8 バイトに全体のビット数。使うブロックの数を添える。
fn padding(rest: &[u8], total: usize) -> ([[u8; BLOCK]; 2], usize) {
    let mut tail = [[0_u8; BLOCK]; 2];
    let used = if rest.len() < BLOCK - 8 { 1 } else { 2 };
    let bits = u64::try_from(total).unwrap_or(u64::MAX).wrapping_mul(8);
    let bytes = tail.as_flattened_mut();
    for (slot, byte) in bytes
        .iter_mut()
        .zip(rest.iter().chain(core::iter::once(&0x80)))
    {
        *slot = *byte;
    }
    let length_at = used * BLOCK - 8;
    for (slot, byte) in bytes.iter_mut().skip(length_at).zip(bits.to_be_bytes()) {
        *slot = byte;
    }
    (tail, used)
}

/// 64 バイトのブロックを 1 つ畳み込む。
fn compress(state: &mut [u32; 8], block: &[u8; BLOCK]) {
    let schedule = schedule_of(block);
    let [
        mut ra,
        mut rb,
        mut rc,
        mut rd,
        mut re,
        mut rf,
        mut rg,
        mut rh,
    ] = *state;
    for (word, constant) in schedule.iter().zip(ROUND_CONSTANTS) {
        let sum1 = re.rotate_right(6) ^ re.rotate_right(11) ^ re.rotate_right(25);
        let choice = (re & rf) ^ (!re & rg);
        let temp1 = rh
            .wrapping_add(sum1)
            .wrapping_add(choice)
            .wrapping_add(constant)
            .wrapping_add(*word);
        let sum0 = ra.rotate_right(2) ^ ra.rotate_right(13) ^ ra.rotate_right(22);
        let majority = (ra & rb) ^ (ra & rc) ^ (rb & rc);
        let temp2 = sum0.wrapping_add(majority);
        rh = rg;
        rg = rf;
        rf = re;
        re = rd.wrapping_add(temp1);
        rd = rc;
        rc = rb;
        rb = ra;
        ra = temp1.wrapping_add(temp2);
    }
    for (slot, value) in state.iter_mut().zip([ra, rb, rc, rd, re, rf, rg, rh]) {
        *slot = slot.wrapping_add(value);
    }
}

/// メッセージスケジュール（FIPS 180-4 の 6.2.2 の 1）。
fn schedule_of(block: &[u8; BLOCK]) -> [u32; 64] {
    let mut schedule = [0_u32; 64];
    for (word, chunk) in schedule.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_be_bytes(*chunk);
    }
    for index in 16..64 {
        let (Some(&w16), Some(&w15), Some(&w7), Some(&w2)) = (
            schedule.get(index - 16),
            schedule.get(index - 15),
            schedule.get(index - 7),
            schedule.get(index - 2),
        ) else {
            continue;
        };
        let sigma0 = w15.rotate_right(7) ^ w15.rotate_right(18) ^ (w15 >> 3_u32);
        let sigma1 = w2.rotate_right(17) ^ w2.rotate_right(19) ^ (w2 >> 10_u32);
        if let Some(slot) = schedule.get_mut(index) {
            *slot = w16
                .wrapping_add(sigma0)
                .wrapping_add(w7)
                .wrapping_add(sigma1);
        }
    }
    schedule
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;

    #[test]
    fn known_vectors_match_fips_180_4() {
        assert_eq!(
            hex::encode(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex::encode(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex::encode(&sha256(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        // 55・56・64 バイトの境（詰め物が 1 ブロックか 2 ブロックか）。
        let million: alloc::vec::Vec<u8> = alloc::vec![b'a'; 1_000_000];
        assert_eq!(
            hex::encode(&sha256(&million)),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
        assert_eq!(hex::encode(&sha256(&[b'a'; 55])).len(), 64);
        assert_eq!(hex::encode(&sha256(&[b'a'; 56])).len(), 64);
        assert_eq!(hex::encode(&sha256(&[b'a'; 64])).len(), 64);
        assert_ne!(sha256(&[b'a'; 55]), sha256(&[b'a'; 56]));
    }
}
