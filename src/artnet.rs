const ARTNET_ID: [u8; 8] = [b'A', b'r', b't', b'-', b'N', b'e', b't', 0x00];
const ARTNET_OP_ARTDMX: u16 = 0x5000;
const HEADER_SIZE: usize = 18;
const DMX_SIZE: usize = 512;

#[derive(Debug, Clone)]
pub struct ArtDmx {
    pub sequence: u8,
    pub physical: u8,
    pub universe: u16,
    pub data: [u8; DMX_SIZE],
    pub length: u16,
}

impl ArtDmx {
    pub fn decode(buf: &[u8]) -> Option<Self> {
        if buf.len() < HEADER_SIZE {
            return None;
        }
        if buf[..8] != ARTNET_ID {
            return None;
        }
        let opcode = u16::from_le_bytes([buf[8], buf[9]]);
        if opcode != ARTNET_OP_ARTDMX {
            return None;
        }
        let sequence = buf[12];
        let physical = buf[13];
        let universe = u16::from_le_bytes([buf[14], buf[15]]);
        let length = u16::from_le_bytes([buf[16], buf[17]]).min(DMX_SIZE as u16);

        let end = HEADER_SIZE + length as usize;
        if buf.len() < end {
            return None;
        }

        let mut data = [0u8; DMX_SIZE];
        data[..length as usize].copy_from_slice(&buf[HEADER_SIZE..end]);

        Some(Self { sequence, physical, universe, data, length })
    }
}
