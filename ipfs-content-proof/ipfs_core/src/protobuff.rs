    /// Protobuf wire types.
    #[derive(Debug, PartialEq)]
    pub enum WireType {
        Varint,
        Bits64,
        LengthDelimited,
        StartGroup,
        EndGroup,
        Bits32,
        WireTypeMax,
    }

    impl WireType {
        fn from(val: &u64) -> Option<WireType> {
            match *val {
                0 => Some(WireType::Varint),
                1 => Some(WireType::Bits64),
                2 => Some(WireType::LengthDelimited),
                3 => Some(WireType::StartGroup),
                4 => Some(WireType::EndGroup),
                5 => Some(WireType::Bits32),
                _ => None,
            }
        }
    }
    /// Maximum number of bytes for a varint.
    /// 64 bits, in groups of base-128 (7 bits).
    pub const MAX_VARINT_BYTES: u64 = 10;

    /// Decode varint from buffer starting at position p.
    /// Returns a tuple containing:
    /// - A boolean indicating if the decoding was successful
    /// - The new position in the buffer after decoding
    /// - The decoded varint value
    fn decode_varint(p: u64, buf: &[u8]) -> (bool, u64, u64) {
        let mut val: u64 = 0;
        let mut i: u64 = 0;

        while i < MAX_VARINT_BYTES {
            // Check that index is within bounds
            if i + p >= buf.len() as u64 {
                return (false, p, 0);
            }

            // Get byte at offset
            let b: u8 = buf[(p + i) as usize];

            // Highest bit is used to indicate if there are more bytes to come
            // Mask to get 7-bit value: 0111 1111
            let v: u8 = b & 0x7F;

            // Groups of 7 bits are ordered least significant first
            val |= (v as u64) << (i * 7);

            // Mask to get keep going bit: 1000 0000
            if b & 0x80 == 0 {
                // [STRICT]
                // Check for trailing zeroes if more than one byte is used
                // (the value 0 still uses one byte)
                if i > 0 && v == 0 {
                    return (false, p, 0);
                }

                break;
            }
            i += 1;
        }

        // Check that at most MAX_VARINT_BYTES are used
        if i >= MAX_VARINT_BYTES {
            return (false, p, 0);
        }

        // [STRICT]
        // If all 10 bytes are used, the last byte (most significant 7 bits)
        // must be at most 0000 0001, since 7*9 = 63
        if i == MAX_VARINT_BYTES - 1 {
            if buf[(p + i) as usize] > 1 {
                return (false, p, 0);
            }
        }

        (true, p + i + 1, val)
    }


    fn decode_key(p: u64, buf: &[u8]) -> (bool, u64, u64, WireType) {
        // The key is a varint with encoding
        // (field_number << 3) | wire_type
        let (success, pos, key) = decode_varint(p, buf);
        if !success {
            return (false, pos, 0, WireType::WireTypeMax);
        }

        let field_number = key >> 3;
        let wire_type_val = key & 0x07;
        // Check that wire type is bounded
        if wire_type_val >= WireType::WireTypeMax as u64 {
            return (false, pos, 0, WireType::WireTypeMax);
        }
        let wire_type = WireType::from(&wire_type_val).unwrap();

        // Start and end group types are deprecated, so forbid them
        if wire_type == WireType::StartGroup || wire_type == WireType::EndGroup {
            return (false, pos, 0, WireType::WireTypeMax);
        }

        (true, pos, field_number, wire_type)
    }

    fn decode_int32(p: u64, buf: &[u8]) -> (bool, u64, i32) {
        let (success, pos, val) = decode_varint(p, buf);
        if !success {
            return (false, pos, 0);
        }

        // [STRICT]
        // Highest 4 bytes must be 0 if positive
        if val >> 63 == 0 {
            if val & 0xFFFFFFFF00000000 != 0 {
                return (false, pos, 0);
            }
        }

        (true, pos, val as i32)
    }

    fn decode_int64(p: u64, buf: &[u8]) -> (bool, u64, i64) {
        let (success, pos, val) = decode_varint(p, buf);
        if !success {
            return (false, pos, 0);
        }

        (true, pos, val as i64)
    }

    fn decode_uint32(p: u64, buf: &[u8]) -> (bool, u64, u32) {
        let (success, pos, val) = decode_varint(p, buf);
        if !success {
            return (false, pos, 0);
        }

        // [STRICT]
        // Highest 4 bytes must be 0
        if val & 0xFFFFFFFF00000000 != 0 {
            return (false, pos, 0);
        }

        (true, pos, val as u32)
    }


    fn decode_uint64(p: u64, buf: &[u8]) -> (bool, u64, u64) {
        let (success, pos, val) = decode_varint(p, buf);
        if !success {
            return (false, pos, 0);
        }

        (true, pos, val)
    }


    fn decode_bool(p: u64, buf: &[u8]) -> (bool, u64, bool) {
        let (success, pos, val) = decode_varint(p, buf);
        if !success {
            return (false, pos, false);
        }

        // [STRICT]
        // Value must be 0 or 1
        if val > 1 {
            return (false, pos, false);
        }

        if val == 0 {
            return (true, pos, false);
        }

        (true, pos, true)
    }


    fn decode_enum(p: u64, buf: &[u8]) -> (bool, u64, i32) {
        decode_int32(p, buf)
    }


    fn decode_bits64(p: u64, buf: &[u8]) -> (bool, u64, u64) {
        let mut val: u64 = 0;

        // Check that index is within bounds
        if 8 + p > buf.len() as u64 {
            return (false, p, 0);
        }

        for i in 0..8 {
            let b: u8 = buf[(p + i) as usize];

            // Little endian
            val |= u64::from(b) << (i * 8);
        }

        (true, p + 8, val)
    }
    fn decode_fixed64(p: u64, buf: &[u8]) -> (bool, u64, u64) {
        let (success, pos, val) = decode_bits64(p, buf);
        if !success {
            return (false, pos, 0);
        }

        (true, pos, val)
    }

    fn decode_length_delimited(p: u64, buf: &[u8]) -> (bool, u64, u64) {
        // Length-delimited fields begin with a varint of the number of bytes that follow
        let (success, pos, size) = decode_varint(p, buf);
        if !success {
            return (false, pos, 0);
        }

        // Check for overflow
        if pos.checked_add(size).is_none() {
            return (false, pos, 0);
        }

        // Check that index is within bounds
        if size.checked_add(pos).map(|sum| sum > buf.len() as u64).unwrap_or(true) {
            return (false, pos, 0);
        }

        (true, pos, size)
    }


    fn encode_varint(n: u64) -> Vec<u8> {
        let mut tmp = n;
        let mut num_bytes = 1;
        while tmp > 0x7F {
            tmp = tmp >> 7;
            num_bytes += 1;
        }

        let mut buf = vec![0u8; num_bytes];

        tmp = n;
        for i in 0..num_bytes {
            buf[i] = 0x80 | (tmp & 0x7F) as u8;
            tmp = tmp >> 7;
        }
        buf[num_bytes - 1] &= 0x7F;

        buf
    }

    fn encode_int32(n: i32) -> Vec<u8> {
        encode_varint(n as u64)
    }
    fn encode_int64(n: i64) -> Vec<u8> {
        encode_varint(n as u64)
    }

    fn encode_uint32(n: u32) -> Vec<u8> {
        encode_varint(n as u64)
    }

    fn encode_uint64(n: u64) -> Vec<u8> {
        encode_varint(n)
    }
