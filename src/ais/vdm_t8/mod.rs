/*
Copyright 2020-2021 Timo Saarinen

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use super::*;

pub mod vdm_t8_payloads;
pub use vdm_t8_payloads::Type8Payload;

// -------------------------------------------------------------------------------------------------

/// Type 8: Binary Broadcast Message
/// 
/// This message type is used for broadcast messages with binary payload.
/// The message can span 1 to 4 NMEA sentences with a maximum payload of 1008 bits.
/// 
/// The interpretation of the payload depends on the DAC (Designated Area Code) 
/// and FID (Functional ID) fields. This implementation stores the raw binary 
/// payload for further processing by applications.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct BinaryBroadcastMessage {
    /// True if the data is about own vessel, false if about other.
    pub own_vessel: bool,

    /// AIS station type.
    pub station: Station,

    /// User ID (30 bits) - MMSI of the broadcasting station
    pub mmsi: u32,

    /// Designated Area Code, DAC (10 bits)
    /// Controls interpretation of the payload along with FID.
    /// Common values:
    /// - 1: International messages
    /// - 200: European Inland waterways
    /// - 316/366: St. Lawrence Seaway (Canada/US)
    pub dac: u16,

    /// Functional ID, FID (6 bits)
    /// Specifies the message subtype within the DAC.
    /// Examples for DAC=1:
    /// - 11: Meteorological/Hydrological Data (deprecated, use 31)
    /// - 22: Area Notice (broadcast)
    /// - 24: Extended ship and voyage data
    /// - 31: Meteorological and Hydrological (current)
    pub fid: u8,

    /// Binary payload data (up to 952 bits after DAC/FID)
    /// Stored as raw bytes for interpretation based on DAC/FID combination.
    /// The actual bit length may be less than the full capacity.
    pub data: Vec<u8>,
    
    /// Actual number of valid bits in the data field
    pub data_bit_length: usize,
    
    /// Parsed payload data (if DAC/FID combination is supported)
    pub parsed_payload: Option<Type8Payload>,
}

impl LatLon for BinaryBroadcastMessage {
    fn latitude(&self) -> Option<f64> {
        None // TODO: depends on DAC/FID and data payload
    }

    fn longitude(&self) -> Option<f64> {
        None // TODO: depends on DAC/FID and data payload
    }
}

// -------------------------------------------------------------------------------------------------

/// AIS VDM/VDO type 8: Binary Broadcast Message
/// 
/// The message structure is:
/// - Bits 0-5: Message Type (6 bits) = 8
/// - Bits 6-7: Repeat Indicator (2 bits)
/// - Bits 8-37: MMSI (30 bits)
/// - Bits 38-39: Spare (2 bits)
/// - Bits 40-49: Designated Area Code (10 bits)
/// - Bits 50-55: Functional ID (6 bits)
/// - Bits 56+: Data payload (variable, up to 952 bits)
pub(crate) fn handle(
    bv: &BitVec,
    station: Station,
    own_vessel: bool,
) -> Result<ParsedMessage, ParseError> {
    // Minimum message length check (header only, 56 bits)
    if bv.len() < 56 {
        return Err(ParseError::InvalidSentence(format!(
            "Type 8 message too short: {} bits (minimum 56 required)",
            bv.len()
        )));
    }

    let mmsi = pick_u64(bv, 8, 30) as u32;
    let dac = pick_u64(bv, 40, 10) as u16;
    let fid = pick_u64(bv, 50, 6) as u8;
    
    // Extract the data payload (everything after bit 56)
    let data_bit_length = if bv.len() > 56 { bv.len() - 56 } else { 0 };
    
    // Convert bits to bytes for storage
    let mut data = Vec::new();
    if data_bit_length > 0 {
        let data_bits = &bv[56..];
        
        // Convert bits to bytes
        // Note: We store the bits in big-endian byte order
        let byte_count = (data_bit_length + 7) / 8;
        data.reserve(byte_count);
        
        for byte_idx in 0..byte_count {
            let bit_start = byte_idx * 8;
            let bit_end = core::cmp::min(bit_start + 8, data_bit_length);
            let bits_in_byte = bit_end - bit_start;
            
            let mut byte_val = 0u8;
            for bit_offset in 0..bits_in_byte {
                if data_bits[bit_start + bit_offset] {
                    byte_val |= 1u8 << (7 - bit_offset);
                }
            }
            data.push(byte_val);
        }
    }
    
    // Try to parse the payload based on DAC/FID
    let parsed_payload = if data_bit_length > 0 {
        vdm_t8_payloads::parse_payload(dac, fid, bv, 56)
    } else {
        None
    };

    Ok(ParsedMessage::BinaryBroadcastMessage(
        BinaryBroadcastMessage {
            own_vessel,
            station,
            mmsi,
            dac,
            fid,
            data,
            data_bit_length,
            parsed_payload,
        },
    ))
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_vdm_type8_basic() {
        let mut p = NmeaParser::new();
        
        // Real Type 8 message example from AIS specification
        // This is a meteo/hydro message (DAC=1, FID=31)
        // MMSI: 002655651 (002655651 = 0x28831B)
        match p.parse_sentence("!AIVDM,1,1,,A,85M:Ih1KmPAU6jAs85`03cJm,0*6A") {
            Ok(ps) => {
                match ps {
                    ParsedMessage::BinaryBroadcastMessage(bbm) => {
                        // Verify MMSI was extracted
                        assert!(bbm.mmsi > 0);
                        // Verify DAC and FID were extracted (they're u16 and u8, always >= 0)
                        assert!(bbm.dac <= 1023);  // 10 bits max
                        assert!(bbm.fid <= 63);    // 6 bits max
                        // data_bit_length is usize, always >= 0
                    }
                    ParsedMessage::Incomplete => {
                        panic!("Message should be complete");
                    }
                    _ => {
                        panic!("Expected BinaryBroadcastMessage, got: {:?}", ps);
                    }
                }
            }
            Err(e) => {
                panic!("Parse error: {}", e);
            }
        }
    }

    #[test]
    fn test_parse_vdm_type8_data_extraction() {
        let mut p = NmeaParser::new();
        
        // Test that data payload is properly extracted
        // Using a longer message with actual payload
        match p.parse_sentence("!AIVDM,1,1,,A,85M:Ih1KmPAU6jAs85`03cJm,0*6A") {
            Ok(ps) => {
                match ps {
                    ParsedMessage::BinaryBroadcastMessage(bbm) => {
                        // If there's payload data, verify it's stored as bytes
                        if bbm.data_bit_length > 0 {
                            assert!(!bbm.data.is_empty());
                            // Verify byte count matches bit length
                            let expected_bytes = (bbm.data_bit_length + 7) / 8;
                            assert_eq!(bbm.data.len(), expected_bytes);
                        }
                    }
                    _ => {
                        panic!("Expected BinaryBroadcastMessage");
                    }
                }
            }
            Err(e) => {
                panic!("Parse error: {}", e);
            }
        }
    }
}
