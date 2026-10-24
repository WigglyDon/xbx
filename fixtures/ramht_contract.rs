//! Original RAMHT geometry fixtures; no firmware/game input or GPU execution.
//! Public reference: Nouveau v6.18 nvkm/core/ramht.c and engine/fifo/nv04.c.
//! The C ABI executes the exact retained PFIFO function, not a Rust translation.
#[link(name = "xbx_ramht_fixture", kind = "static")]
unsafe extern "C" {
    fn xbx_fixture_ramht_hash(size_field: u32, handle: u32, channel: u32) -> u32;
}

fn index(size: u32, handle: u32, channel: u32) -> u32 {
    unsafe { xbx_fixture_ramht_hash(size, handle, channel) }
}

#[test]
fn configured_table_contains_the_synthetic_high_handle() {
    // 4 KiB / 8 = 512 entries. The next handle bit must fold into bit zero.
    let result = index(0, 0x200, 0);
    assert!(
        result * 8 < 4096,
        "synthetic handle escaped the configured table: {result}"
    );
    assert_eq!(result, 1);
}

#[test]
fn independent_fixed_vectors_cover_all_table_sizes() {
    for (size, high_bit, all_bits) in [(0, 16, 480), (1, 2, 1020), (2, 512, 1023), (3, 128, 255)] {
        assert_eq!(index(size, 0, 0), 0);
        assert_eq!(index(size, 1, 0), 1);
        assert_eq!(index(size, 0x8000_0000, 0), high_bit);
        assert_eq!(index(size, u32::MAX, 0), all_bits);
    }
}

#[test]
fn handle_bits_and_low_sixteen_channels_stay_inside_their_table() {
    // This covers channels 0..15, including the reference's ordinary channel0
    // contract. It makes no new claim about additional NV2A channel semantics.
    for size in 0..4 {
        let entries = 512 << size;
        let bits = 9 + size;
        for channel in 0..16 {
            for bit in 0..32 {
                // Independent bit-plane parity rather than chunk-folding code.
                let expected = (1 << (bit % bits)) ^ (channel << (bits - 4));
                let result = index(size, 1 << bit, channel);
                assert!(result < entries);
                assert_eq!(result, expected);
            }
        }
    }
}
