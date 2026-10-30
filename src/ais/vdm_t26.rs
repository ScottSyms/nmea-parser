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

// -------------------------------------------------------------------------------------------------

/// Type 26: Multiple Slot Binary Message
#[derive(Default, Clone, Debug, PartialEq)]
pub struct MultipleSlotBinaryMessage {
    /// True if the data is about own vessel, false if about other.
    pub own_vessel: bool,

    /// AIS station type.
    pub station: Station,

    /// User ID (30 bits)
    pub mmsi: u32,

    /// When 'addressed' flag is on this field contains the parsed destination MMSI.
    pub dest_mmsi: Option<u32>,

    /// When 'addressed' flag is off and 'structured' flag on this field contains
    /// application ID which consists of 10-bit DAC and 6-bit FID as in message types 6 and 8.
    pub app_id: Option<u16>,

    /// Data field of length 0-1004 bits.
    pub data: BitVec,

    /// Radio status
    pub radio: u32,
}

// -------------------------------------------------------------------------------------------------

/// AIS VDM/VDO type 26: Multiple Slot Binary Message
#[allow(clippy::collapsible_if)]
pub(crate) fn handle(
    bv: &BitVec,
    station: Station,
    own_vessel: bool,
) -> Result<ParsedMessage, ParseError> {
    let addressed = pick_u64(bv, 38, 1) != 0;
    let structured = pick_u64(bv, 39, 1) != 0;
    // Radio status is the last 20 bits; the data field ends where it begins.
    let radio_index = bv.len().saturating_sub(20);

    Ok(ParsedMessage::MultipleSlotBinaryMessage(
        MultipleSlotBinaryMessage {
            own_vessel: { own_vessel },
            station: { station },
            mmsi: { pick_u64(bv, 8, 30) as u32 },
            dest_mmsi: {
                if addressed {
                    Some(pick_u64(bv, 40, 30) as u32)
                } else {
                    None
                }
            },
            app_id: {
                if addressed {
                    None
                } else if structured {
                    Some(pick_u64(bv, 70, 16) as u16)
                } else {
                    None
                }
            },
            data: {
                if addressed {
                    pick_bits(bv, 70, radio_index)
                } else if structured {
                    pick_bits(bv, 86, radio_index)
                } else {
                    pick_bits(bv, 40, radio_index)
                }
            },
            radio: { pick_u64(bv, radio_index, 20) as u32 },
        },
    ))
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_vdm_type26() {
        let mut p = NmeaParser::new();

        // Valid message
        match p.parse_sentence("!AIVDM,1,1,,A,JB3R0GO7p>vQL8tjw0b5hqpd0706kh9d3lR2vbl0400,2*40") {
            Ok(ps) => {
                match ps {
                    // The expected result
                    ParsedMessage::MultipleSlotBinaryMessage(msbm) => {
                        assert_eq!(msbm.mmsi, 137920605);
                        assert_eq!(msbm.dest_mmsi, Some(838351848));
                        assert_eq!(msbm.app_id, None);
                    }
                    ParsedMessage::Incomplete => {
                        assert!(false);
                    }
                    _ => {
                        assert!(false);
                    }
                }
            }
            Err(e) => {
                assert_eq!(e.to_string(), "OK");
            }
        }

        // Too short payload
        match p.parse_sentence("!AIVDM,1,1,,,Jl@bhbmCU`:lwOd0,0*48") {
            Ok(ps) => {
                match ps {
                    // The expected result
                    ParsedMessage::MultipleSlotBinaryMessage(msbm) => {
                        assert_eq!(msbm.mmsi, 285913259);
                        assert_eq!(msbm.dest_mmsi, None);
                        assert_eq!(msbm.app_id, Some(16254));
                    }
                    ParsedMessage::Incomplete => {
                        assert!(false);
                    }
                    _ => {
                        assert!(false);
                    }
                }
            }
            Err(e) => {
                assert_eq!(e.to_string(), "OK");
            }
        }
    }

    #[test]
    fn test_parse_vdm_type26_truncated() {
        let mut p = NmeaParser::new();

        // Shorter than the header plus radio status (36 bits; the last two MMSI bits are missing)
        match p.parse_sentence("!AIVDM,1,1,,A,J1mg=5,0*5F") {
            Ok(ParsedMessage::MultipleSlotBinaryMessage(msbm)) => {
                assert_eq!(msbm.mmsi, 123456788);
                assert!(msbm.data.is_empty());
            }
            Ok(_) => assert!(false),
            Err(e) => assert_eq!(e.to_string(), "OK"),
        }
    }
}
