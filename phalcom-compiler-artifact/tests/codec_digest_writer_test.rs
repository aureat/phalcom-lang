use phalcom_compiler_artifact::{ArtifactDigest, BinaryWriter};

#[test]
fn digest_writer_hashes_the_exact_buffer_writer_byte_stream_without_retaining_bytes() {
    const DOMAIN: &str = "telos.test.binary-writer.digest-equivalence";

    let mut buffered = BinaryWriter::new();
    buffered.write_u8(7);
    buffered.write_u16(0x1234);
    buffered.write_u32(0x89ab_cdef);
    buffered.write_u64(0x0123_4567_89ab_cdef);
    buffered.write_bool(true);
    buffered.write_str("portable");
    buffered.write_bytes_with_len(&[1, 3, 5, 7, 9]);

    let expected_len = buffered.len();
    let expected = ArtifactDigest::digest_domain(DOMAIN, buffered.as_slice());

    let mut digest = BinaryWriter::digest(DOMAIN);
    digest.write_u8(7);
    digest.write_u16(0x1234);
    digest.write_u32(0x89ab_cdef);
    digest.write_u64(0x0123_4567_89ab_cdef);
    digest.write_bool(true);
    digest.write_str("portable");
    digest.write_bytes_with_len(&[1, 3, 5, 7, 9]);

    assert_eq!(digest.len(), expected_len);
    assert_eq!(digest.into_digest(), expected);
}
