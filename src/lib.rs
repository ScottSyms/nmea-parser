/*
Copyright 2021 Timo Saarinen

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

//! # NMEA Parser: NMEA parser for Rust
//!
//! This crate aims to cover all AIS sentences and the most important GNSS sentences used with
//! NMEA 0183 standard. The parser supports AIS class A and B types. It also identifies GPS,
//! GLONASS, Galileo, BeiDou, NavIC and QZSS satellite systems.
//!
//! ## Tag Block Support
//! 
//! This parser supports NMEA 4.10 tag blocks, which provide additional metadata for NMEA sentences.
//! Tag blocks are enclosed in backslashes and contain comma-separated fields:
//! 
//! ```text
//! \g:1-2-73874,n:157036,s:r003669945,c:1241544035*4A\!AIVDM,1,1,,B,15N4cJ`005Jrek0H@9n`DW5608EP,0*13
//! ```
//! 
//! Supported tag block fields:
//! - `c` - UNIX timestamp (seconds or milliseconds)
//! - `d` - Destination identifier (max 15 chars)
//! - `g` - Sentence grouping (format: sentence-total-group_id)
//! - `n` - Line count
//! - `r` - Relative time
//! - `s` - Source/station identifier
//! - `t`/`i` - Text string (max 15 chars)
//!
//! Use `parse_sentence_with_tags()` to access tag block information, or continue using
//! `parse_sentence()` for backward compatibility (tag blocks are ignored).
//!
//! Usage in a `#[no_std]` environment is also possible though an allocator is required

#![forbid(unsafe_code)]
#![allow(dead_code)]
#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate log;

extern crate num_traits;

#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use bitvec::prelude::*;
pub use chrono;
use chrono::prelude::*;
use chrono::{DateTime, TimeZone};
use hashbrown::HashMap;
use core::cmp::max;
use core::str::FromStr;

#[cfg(not(test))]
use num_traits::float::FloatCore;

pub mod ais;
mod error;
pub mod gnss;
pub mod json_output;
pub mod tag_block;
mod util;
mod json_date_time_utc;
mod json_fixed_offset;

pub use error::ParseError;
pub use tag_block::TagBlock;
use util::*;

// -------------------------------------------------------------------------------------------------

/// Contains both a parsed NMEA message and any associated tag block
#[derive(Clone, Debug, PartialEq)]
pub struct NmeaMessage {
    /// The parsed NMEA message
    pub message: ParsedMessage,
    /// Associated tag block if present
    pub tag_block: Option<TagBlock>,
}

impl NmeaMessage {
    /// Create a new NMEA message with optional tag block
    pub fn new(message: ParsedMessage, tag_block: Option<TagBlock>) -> Self {
        NmeaMessage { message, tag_block }
    }
    
    /// Create a new NMEA message without tag block
    pub fn without_tag_block(message: ParsedMessage) -> Self {
        NmeaMessage { 
            message, 
            tag_block: None 
        }
    }
}

/// Result from function `NmeaParser::parse_sentence()`. If the given sentence represents only a
/// partial message `ParsedMessage::Incomplete` is returned.
#[derive(Clone, Debug, PartialEq)]
pub enum ParsedMessage {
    /// The given sentence is only part of multi-sentence message and we need more data to
    /// create the actual result. State is stored in `NmeaParser` object.
    Incomplete,

    /// AIS VDM/VDO t1, t2, t3, t18 and t27
    VesselDynamicData(ais::VesselDynamicData),

    /// AIS VDM/VDO t5 and t24
    VesselStaticData(ais::VesselStaticData),

    /// AIS VDM/VDO type 4
    BaseStationReport(ais::BaseStationReport),

    /// AIS VDM/VDO type 6
    BinaryAddressedMessage(ais::BinaryAddressedMessage),
    //
    //    /// AIS VDM/VDO type 7
    //    BinaryAcknowledge(ais::BinaryAcknowledge),
    //
    //    /// AIS VDM/VDO type 8
    //    BinaryBroadcastMessage(ais::BinaryBroadcastMessage),

    // AIS VDM/VDO type 9
    StandardSarAircraftPositionReport(ais::StandardSarAircraftPositionReport),

    // AIS VDM/VDO type 10
    UtcDateInquiry(ais::UtcDateInquiry),

    // AIS VDM/VDO type 11
    UtcDateResponse(ais::BaseStationReport),

    // AIS VDM/VDO type 12
    AddressedSafetyRelatedMessage(ais::AddressedSafetyRelatedMessage),

    // AIS VDM/VDO type 13
    SafetyRelatedAcknowledgement(ais::SafetyRelatedAcknowledgement),

    // AIS VDM/VDO type 14
    SafetyRelatedBroadcastMessage(ais::SafetyRelatedBroadcastMessage),

    // AIS VDM/VRO type 15
    Interrogation(ais::Interrogation),

    // AIS VDM/VRO type 16
    AssignmentModeCommand(ais::AssignmentModeCommand),

    // AIS VDM/VRO type 17
    DgnssBroadcastBinaryMessage(ais::DgnssBroadcastBinaryMessage),

    // AIS VDM/VRO type 20
    DataLinkManagementMessage(ais::DataLinkManagementMessage),

    // AIS VDM/VDO type 21
    AidToNavigationReport(ais::AidToNavigationReport),

    // AIS VDM/VDO type 22
    ChannelManagement(ais::ChannelManagement),

    // AIS VDM/VDO type 23
    GroupAssignmentCommand(ais::GroupAssignmentCommand),

    // AIS VDM/VDO type 25
    SingleSlotBinaryMessage(ais::SingleSlotBinaryMessage),

    // AIS VDM/VDO type 26
    MultipleSlotBinaryMessage(ais::MultipleSlotBinaryMessage),

    /// GGA
    Gga(gnss::GgaData),

    /// RMC
    Rmc(gnss::RmcData),

    /// GNS
    Gns(gnss::GnsData),

    /// GSA
    Gsa(gnss::GsaData),

    /// GSV
    Gsv(Vec<gnss::GsvData>),

    /// VTG
    Vtg(gnss::VtgData),

    /// GLL
    Gll(gnss::GllData),

    /// ALM
    Alm(gnss::AlmData),

    /// DTM
    Dtm(gnss::DtmData),

    /// MSS
    Mss(gnss::MssData),

    /// STN
    Stn(gnss::StnData),

    /// VBW
    Vbw(gnss::VbwData),

    /// ZDA
    Zda(gnss::ZdaData),

    /// DPT
    Dpt(gnss::DptData),

    /// DBS
    Dbs(gnss::DbsData),

    /// MTW
    Mtw(gnss::MtwData),

    /// VHW
    Vhw(gnss::VhwData),

    /// HDT
    Hdt(gnss::HdtData),

    /// MWV
    Mwv(gnss::MwvData),
}

// -------------------------------------------------------------------------------------------------

/// Read-only access to geographical position in the implementing type.
pub trait LatLon {
    /// Return the latitude of the position contained by the object. If the position is not
    /// available return `None`.
    fn latitude(&self) -> Option<f64>;

    /// Return the longitude of the position contained by the object. If the position is not
    /// available return `None`.
    fn longitude(&self) -> Option<f64>;
}

// -------------------------------------------------------------------------------------------------

/// NMEA sentence parser which keeps multi-sentence state between `parse_sentence` calls.
/// The parser tries to be as permissible as possible about the field formats because some NMEA
/// encoders don't follow the standards strictly.
#[derive(Clone)]
pub struct NmeaParser {
    saved_fragments: HashMap<String, String>,
    saved_vsds: HashMap<u32, ais::VesselStaticData>,
}

impl Default for NmeaParser {
    fn default() -> Self {
        Self::new()
    }
}

impl NmeaParser {
    /// Construct an empty parser which is ready to receive sentences.
    pub fn new() -> NmeaParser {
        NmeaParser {
            saved_fragments: HashMap::new(),
            saved_vsds: HashMap::new(),
        }
    }

    /// Clear internal state of the parser. Multi-sentence state is lost when this function
    /// is called.
    pub fn reset(&mut self) {
        self.saved_fragments.clear();
        self.saved_vsds.clear();
    }

    /// Push string-to-string mapping to store.
    fn push_string(&mut self, key: String, value: String) {
        self.saved_fragments.insert(key, value);
    }

    /// Pull string-to-string mapping by key from store.
    fn pull_string(&mut self, key: String) -> Option<String> {
        self.saved_fragments.remove(&key)
    }

    /// Tests whether the given string-to-string mapping exists in the store.
    fn contains_key(&mut self, key: String) -> bool {
        self.saved_fragments.contains_key(&key)
    }

    /// Return number of string-to-string mappings stored.
    fn strings_count(&self) -> usize {
        self.saved_fragments.len()
    }

    /// Push MMSI-to-VesselStaticData mapping to store.
    fn push_vsd(&mut self, mmsi: u32, vsd: ais::VesselStaticData) {
        self.saved_vsds.insert(mmsi, vsd);
    }

    /// Pull MMSI-to-VesselStaticData mapping from store.
    fn pull_vsd(&mut self, mmsi: u32) -> Option<ais::VesselStaticData> {
        self.saved_vsds.remove(&mmsi)
    }

    /// Return number of MMSI-to-VesselStaticData mappings in store.
    fn vsds_count(&self) -> usize {
        self.saved_vsds.len()
    }

    /// Parse one NMEA sentence and return the result, including any tag block information.
    /// Multi-sentence payloads in AIS and other message types are supported. If given sentence 
    /// is part of multi-sentence message, `ParsedMessage::Incomplete` is returned. The actual 
    /// result is returned when all the parts have been sent to the parser.
    pub fn parse_sentence_with_tags(&mut self, sentence: &str) -> Result<NmeaMessage, ParseError> {
        // Check for tag block at the beginning
        let (tag_block, nmea_sentence) = if sentence.starts_with('\\') {
            // Find the end of the tag block
            if let Some(end_pos) = sentence[1..].find('\\') {
                let tag_block_str = &sentence[0..=end_pos + 1];
                let tag_block = TagBlock::parse(tag_block_str)?;
                let remaining = sentence[end_pos + 2..].trim_start();
                (Some(tag_block), remaining)
            } else {
                return Err(ParseError::InvalidSentence(
                    "Tag block not properly closed".to_string()
                ));
            }
        } else {
            (None, sentence)
        };
        
        // Parse the NMEA sentence part
        let parsed_message = self.parse_sentence_internal(nmea_sentence)?;
        
        Ok(NmeaMessage::new(parsed_message, tag_block))
    }

    /// Parse NMEA sentence into `ParsedMessage` enum. If the given sentence is part of
    /// a multipart message the related state is saved into the parser and
    /// `ParsedMessage::Incomplete` is returned. The actual result is returned when all the parts
    /// have been sent to the parser.
    pub fn parse_sentence(&mut self, sentence: &str) -> Result<ParsedMessage, ParseError> {
        let result = self.parse_sentence_with_tags(sentence)?;
        Ok(result.message)
    }

    /// Internal function to parse the actual NMEA sentence (without tag blocks)
    fn parse_sentence_internal(&mut self, sentence: &str) -> Result<ParsedMessage, ParseError> {
        // Shed characters prefixing the message if they exist
        let sentence = {
            if let Some(start_idx) = sentence.find(['$', '!']) {
                &sentence[start_idx..]
            } else {
                return Err(ParseError::InvalidSentence(format!(
                    "Invalid NMEA sentence: {}",
                    sentence
                )));
            }
        };

        // Calculate NMEA checksum and compare it to the given one. Also, remove the checksum part
        // from the sentence to simplify next processing steps.
        let mut checksum = 0;
        let (sentence, checksum_hex_given) = {
            if let Some(pos) = sentence.rfind('*') {
                if pos + 3 <= sentence.len() {
                    (
                        sentence[0..pos].to_string(),
                        sentence[(pos + 1)..(pos + 3)].to_string(),
                    )
                } else {
                    debug!("Invalid checksum found for sentence: {}", sentence);
                    (sentence[0..pos].to_string(), "".to_string())
                }
            } else {
                debug!("No checksum found for sentence: {}", sentence);
                (sentence.to_string(), "".to_string())
            }
        };
        for c in sentence.as_str().chars().skip(1) {
            checksum ^= c as u8;
        }
        let checksum_hex_calculated = format!("{:02X?}", checksum);
        if checksum_hex_calculated != checksum_hex_given && !checksum_hex_given.is_empty() {
            return Err(ParseError::CorruptedSentence(format!(
                "Corrupted NMEA sentence: {:02X?} != {:02X?}",
                checksum_hex_calculated, checksum_hex_given
            )));
        }

        // Pick sentence type
        let sentence_type = {
            if let Some(i) = sentence.find(',') {
                &sentence[0..i]
            } else {
                return Err(ParseError::InvalidSentence(format!(
                    "Invalid NMEA sentence: {}",
                    sentence
                )));
            }
        };

        // Validate sentence type characters
        if !sentence_type
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '$' || c == '!')
        {
            return Err(ParseError::InvalidSentence(format!(
                "Invalid characters in sentence type: {}",
                sentence_type
            )));
        }

        let (nav_system, station, sentence_type) = if sentence_type.starts_with('$') {
            // Identify GNSS system by talker ID.
            let nav_system = gnss::NavigationSystem::from_str(
                sentence_type
                    .get(1..)
                    .ok_or(ParseError::CorruptedSentence("Empty String".to_string()))?,
            )?;
            let sentence_type = if !sentence_type.starts_with('P') && sentence_type.len() == 6 {
                format!(
                    "${}",
                    sentence_type
                        .get(3..6)
                        .ok_or(ParseError::InvalidSentence(format!(
                            "{sentence_type} is too short."
                        )))?
                )
            } else {
                String::from(sentence_type)
            };
            (nav_system, ais::Station::Other, sentence_type)
        } else if sentence_type.starts_with('!') {
            // Identify AIS station
            let station = ais::Station::from_str(
                sentence_type
                    .get(1..)
                    .ok_or(ParseError::CorruptedSentence("Empty String".to_string()))?,
            )?;
            let sentence_type = if sentence_type.len() == 6 {
                format!(
                    "!{}",
                    sentence_type
                        .get(3..6)
                        .ok_or(ParseError::InvalidSentence(format!(
                            "{sentence_type} is too short."
                        )))?
                )
            } else {
                String::from(sentence_type)
            };
            (gnss::NavigationSystem::Other, station, sentence_type)
        } else {
            (
                gnss::NavigationSystem::Other,
                ais::Station::Other,
                String::from(sentence_type),
            )
        };

        // Handle sentence types
        match sentence_type.as_str() {
            // $xxGGA - Global Positioning System Fix Data
            "$GGA" => gnss::gga::handle(sentence.as_str(), nav_system),
            // $xxRMC - Recommended minimum specific GPS/Transit data
            "$RMC" => gnss::rmc::handle(sentence.as_str(), nav_system),
            // $xxGNS - GNSS fix data
            "$GNS" => gnss::gns::handle(sentence.as_str(), nav_system),
            // $xxGSA - GPS DOP and active satellites
            "$GSA" => gnss::gsa::handle(sentence.as_str(), nav_system),
            // $xxGSV - GPS Satellites in view
            "$GSV" => gnss::gsv::handle(sentence.as_str(), nav_system, self),
            // $xxVTG - Track made good and ground speed
            "$VTG" => gnss::vtg::handle(sentence.as_str(), nav_system),
            // $xxGLL - Geographic position, latitude / longitude
            "$GLL" => gnss::gll::handle(sentence.as_str(), nav_system),
            // $xxALM - Almanac Data
            "$ALM" => gnss::alm::handle(sentence.as_str(), nav_system),
            // $xxDTM - Datum reference
            "$DTM" => gnss::dtm::handle(sentence.as_str(), nav_system),
            // $xxMSS - MSK receiver signal
            "$MSS" => gnss::mss::handle(sentence.as_str(), nav_system),
            // $xxSTN - Multiple Data ID
            "$STN" => gnss::stn::handle(sentence.as_str(), nav_system),
            // $xxVBW - MSK Receiver Signal
            "$VBW" => gnss::vbw::handle(sentence.as_str(), nav_system),
            // $xxZDA - Date and time
            "$ZDA" => gnss::zda::handle(sentence.as_str(), nav_system),

            // Received AIS data from other or own vessel
            "!VDM" | "!VDO" => {
                let own_vessel = sentence_type.as_str() == "!VDO";
                let mut fragment_count = 0;
                let mut fragment_number = 0;
                let mut message_id = None;
                let mut radio_channel_code = None;
                let mut payload_string: String = "".into();
                for (num, s) in sentence.split(',').enumerate() {
                    match num {
                        1 => {
                            match s.parse::<u8>() {
                                Ok(i) => {
                                    fragment_count = i;
                                }
                                Err(_) => {
                                    return Err(ParseError::InvalidSentence(format!(
                                        "Failed to parse fragment count: {}",
                                        s
                                    )));
                                }
                            };
                        }
                        2 => {
                            match s.parse::<u8>() {
                                Ok(i) => {
                                    fragment_number = i;
                                }
                                Err(_) => {
                                    return Err(ParseError::InvalidSentence(format!(
                                        "Failed to parse fragment count: {}",
                                        s
                                    )));
                                }
                            };
                        }
                        3 => {
                            message_id = s.parse::<u64>().ok();
                        }
                        4 => {
                            // Radio channel code
                            radio_channel_code = Some(s);
                        }
                        5 => {
                            payload_string = s.to_string();
                        }
                        6 => {
                            // fill bits
                        }
                        _ => {}
                    }
                }

                // Try parse the payload
                let mut bv: Option<BitVec> = None;
                match fragment_count {
                    1 => bv = parse_payload(&payload_string).ok(),
                    2 => {
                        if let Some(msg_id) = message_id {
                            let key1 = make_fragment_key(
                                &sentence_type.to_string(),
                                msg_id,
                                fragment_count,
                                1,
                                radio_channel_code.unwrap_or(""),
                            );
                            let key2 = make_fragment_key(
                                &sentence_type.to_string(),
                                msg_id,
                                fragment_count,
                                2,
                                radio_channel_code.unwrap_or(""),
                            );
                            match fragment_number {
                                1 => {
                                    if let Some(p) = self.pull_string(key2) {
                                        let mut payload_string_combined = payload_string;
                                        payload_string_combined.push_str(p.as_str());
                                        bv = parse_payload(&payload_string_combined).ok();
                                    } else {
                                        self.push_string(key1, payload_string);
                                    }
                                }
                                2 => {
                                    if let Some(p) = self.pull_string(key1) {
                                        let mut payload_string_combined = p;
                                        payload_string_combined.push_str(payload_string.as_str());
                                        bv = parse_payload(&payload_string_combined).ok();
                                    } else {
                                        self.push_string(key2, payload_string);
                                    }
                                }
                                _ => {
                                    warn!(
                                        "Unexpected NMEA fragment number: {}/{}",
                                        fragment_number, fragment_count
                                    );
                                }
                            }
                        } else {
                            warn!(
                                "NMEA message_id missing from {} than supported 2",
                                sentence_type
                            );
                        }
                    }
                    _ => {
                        warn!(
                            "NMEA sentence fragment count greater ({}) than supported 2",
                            fragment_count
                        );
                    }
                }

                if let Some(bv) = bv {
                    let message_type = pick_u64(&bv, 0, 6);
                    match message_type {
                        // Position report with SOTDMA/ITDMA
                        1..=3 => ais::vdm_t1t2t3::handle(&bv, station, own_vessel),
                        // Base station report
                        4 => ais::vdm_t4::handle(&bv, station, own_vessel),
                        // Ship static voyage related data
                        5 => ais::vdm_t5::handle(&bv, station, own_vessel),
                        // Addressed binary message
                        6 => ais::vdm_t6::handle(&bv, station, own_vessel),
                        // Binary acknowledge
                        7 => {
                            // TODO: implementation
                            Err(ParseError::UnsupportedSentenceType(format!(
                                "Unsupported {} message type: {}",
                                sentence_type, message_type
                            )))
                        }
                        // Binary broadcast message
                        8 => {
                            // TODO: implementation
                            Err(ParseError::UnsupportedSentenceType(format!(
                                "Unsupported {} message type: {}",
                                sentence_type, message_type
                            )))
                        }
                        // Standard SAR aircraft position report
                        9 => ais::vdm_t9::handle(&bv, station, own_vessel),
                        // UTC and Date inquiry
                        10 => ais::vdm_t10::handle(&bv, station, own_vessel),
                        // UTC and date response
                        11 => ais::vdm_t11::handle(&bv, station, own_vessel),
                        // Addressed safety related message
                        12 => ais::vdm_t12::handle(&bv, station, own_vessel),
                        // Safety related acknowledge
                        13 => ais::vdm_t13::handle(&bv, station, own_vessel),
                        // Safety related broadcast message
                        14 => ais::vdm_t14::handle(&bv, station, own_vessel),
                        // Interrogation
                        15 => ais::vdm_t15::handle(&bv, station, own_vessel),
                        // Assigned mode command
                        16 => ais::vdm_t16::handle(&bv, station, own_vessel),
                        // GNSS binary broadcast message
                        17 => ais::vdm_t17::handle(&bv, station, own_vessel),
                        // Standard class B CS position report
                        18 => ais::vdm_t18::handle(&bv, station, own_vessel),
                        // Extended class B equipment position report
                        19 => ais::vdm_t19::handle(&bv, station, own_vessel),
                        // Data link management
                        20 => ais::vdm_t20::handle(&bv, station, own_vessel),
                        // Aids-to-navigation report
                        21 => ais::vdm_t21::handle(&bv, station, own_vessel),
                        // Channel management
                        22 => ais::vdm_t22::handle(&bv, station, own_vessel),
                        // Group assignment command
                        23 => ais::vdm_t23::handle(&bv, station, own_vessel),
                        // Class B CS static data report
                        24 => ais::vdm_t24::handle(&bv, station, self, own_vessel),
                        // Single slot binary message
                        25 => ais::vdm_t25::handle(&bv, station, own_vessel),
                        // Multiple slot binary message
                        26 => ais::vdm_t26::handle(&bv, station, own_vessel),
                        // Long range AIS broadcast message
                        27 => ais::vdm_t27::handle(&bv, station, own_vessel),
                        _ => Err(ParseError::UnsupportedSentenceType(format!(
                            "Unsupported {} message type: {}",
                            sentence_type, message_type
                        ))),
                    }
                } else {
                    Ok(ParsedMessage::Incomplete)
                }
            }
            "$DPT" => gnss::dpt::handle(sentence.as_str()),
            "$DBS" => gnss::dbs::handle(sentence.as_str()),
            "$MTW" => gnss::mtw::handle(sentence.as_str()),
            "$VHW" => gnss::vhw::handle(sentence.as_str()),
            "$HDT" => gnss::hdt::handle(sentence.as_str()),
            "$MWV" => gnss::mwv::handle(sentence.as_str()),
            _ => Err(ParseError::UnsupportedSentenceType(format!(
                "Unsupported sentence type: {}",
                sentence_type
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_invalid_sentence() {
        let mut p = NmeaParser::new();
        assert_eq!(
            p.parse_sentence("$޴GAGSV,,"),
            Err(ParseError::InvalidSentence(
                "Invalid characters in sentence type: $\u{7b4}GAGSV".to_string()
            ))
        );
        assert_eq!(
            p.parse_sentence("$WIMWV,295.4,T,"),
            Err(ParseError::CorruptedSentence(
                "pick string for \"wind_speed_knots\" was None".to_string()
            ))
        );
        assert_eq!(
            p.parse_sentence("!AIVDM,not,a,valid,nmea,string,0*00"),
            Err(ParseError::CorruptedSentence(
                "Corrupted NMEA sentence: \"17\" != \"00\"".to_string()
            ))
        );
        assert_eq!(
            p.parse_sentence("!"),
            Err(ParseError::InvalidSentence(
                "Invalid NMEA sentence: !".to_string()
            ))
        );
    }
    #[test]
    fn test_parse_prefix_chars() {
        // Try a sentence with prefix characters
        let mut p = NmeaParser::new();
        assert!(p
            .parse_sentence(",1277,-106*35\r\n!AIVDM,1,1,,A,152IS=iP?w<tSF0l4Q@>4?wp0H:;,0*2")
            .ok()
            .is_some());
    }

    #[test]
    fn test_parse_corrupted() {
        // Try a sentence with mismatching checksum
        let mut p = NmeaParser::new();
        assert!(p
            .parse_sentence("!AIVDM,1,1,,A,38Id705000rRVJhE7cl9n;160000,0*41")
            .ok()
            .is_none());
    }

    #[test]
    fn test_parse_missing_checksum() {
        // Try a sentence without checksum
        let mut p = NmeaParser::new();
        assert!(p
            .parse_sentence("!AIVDM,1,1,,A,38Id705000rRVJhE7cl9n;160000,0")
            .ok()
            .is_some());
    }

    #[test]
    fn test_parse_invalid_utc() {
        // Try a sentence with invalite utc
        let mut p = NmeaParser::new();
        assert_eq!(
            p.parse_sentence("!AIVDM,1,1,,B,4028iqT47wP00wGiNbH8H0700`2H,0*13"),
            Err(ParseError::InvalidSentence(String::from(
                "Failed to parse Utc Date from y:4161 m:15 d:31 h:0 m:0 s:0"
            )))
        );
    }

    #[test]
    fn test_parse_proprietary() {
        /* FIXME: The test fails
                // Try a proprietary sentence
                let mut p = NmeaParser::new();
                assert_eq!(
                    p.parse_sentence("$PGRME,15.0,M,45.0,M,25.0,M*1C"),
                    Err(ParseError::UnsupportedSentenceType(String::from(
                        "Unsupported sentence type: $PGRME"
                    )))
                );
                // Try a proprietary sentence with four characters
                assert_eq!(
                    p.parse_sentence("$PGRM,00,1,,,*15"),
                    Err(ParseError::UnsupportedSentenceType(String::from(
                        "Unsupported sentence type: $PGRM"
                    )))
                );
        */
    }

    #[test]
    fn test_parse_invalid_talker() {
        // Try parse malformed sentences
        let mut p = NmeaParser::new();
        assert_eq!(
            p.parse_sentence("$QQ,*2C"),
            Err(ParseError::UnsupportedSentenceType(String::from(
                "Unsupported sentence type: $QQ"
            )))
        );
        assert_eq!(
            p.parse_sentence("$A,a0,*10"),
            Err(ParseError::InvalidSentence(String::from(
                "Invalid talker identifier"
            )))
        );
        assert_eq!(
            p.parse_sentence("$,0a,*51"),
            Err(ParseError::InvalidSentence(String::from(
                "Invalid talker identifier"
            )))
        );
    }

    #[test]
    fn test_nmea_parser() {
        let mut p = NmeaParser::new();

        // String test
        p.push_string("a".into(), "b".into());
        assert_eq!(p.strings_count(), 1);
        p.push_string("c".into(), "d".into());
        assert_eq!(p.strings_count(), 2);
        p.pull_string("a".into());
        assert_eq!(p.strings_count(), 1);
        p.pull_string("c".into());
        assert_eq!(p.strings_count(), 0);

        // VesselStaticData test
        p.push_vsd(1, Default::default());
        assert_eq!(p.vsds_count(), 1);
        p.push_vsd(2, Default::default());
        assert_eq!(p.vsds_count(), 2);
        p.pull_vsd(1);
        assert_eq!(p.vsds_count(), 1);
        p.pull_vsd(2);
        assert_eq!(p.vsds_count(), 0);
    }

    #[test]
    fn test_country() {
        assert_eq!(vsd(230992580).country().unwrap(), "FI");
        assert_eq!(vsd(276009860).country().unwrap(), "EE");
        assert_eq!(vsd(265803690).country().unwrap(), "SE");
        assert_eq!(vsd(273353180).country().unwrap(), "RU");
        assert_eq!(vsd(211805060).country().unwrap(), "DE");
        assert_eq!(vsd(257037270).country().unwrap(), "NO");
        assert_eq!(vsd(227232370).country().unwrap(), "FR");
        assert_eq!(vsd(248221000).country().unwrap(), "MT");
        assert_eq!(vsd(374190000).country().unwrap(), "PA");
        assert_eq!(vsd(412511368).country().unwrap(), "CN");
        assert_eq!(vsd(512003200).country().unwrap(), "NZ");
        assert_eq!(vsd(995126020).country(), None);
        assert_eq!(vsd(2300049).country(), None);
        assert_eq!(vsd(0).country(), None);
    }

    /// Create a `VesselStaticData` with the given MMSI
    fn vsd(mmsi: u32) -> ais::VesselStaticData {
        let mut vsd = ais::VesselStaticData::default();
        vsd.mmsi = mmsi;
        vsd
    }

    #[test]
    fn test_parse_sentence_with_tag_block() {
        let mut p = NmeaParser::new();
        let sentence = r"\g:1-2-73874,n:157036,s:r003669945,c:1241544035*4A\!AIVDM,1,1,,B,15N4cJ`005Jrek0H@9n`DW5608EP,0*13";
        
        let result = p.parse_sentence_with_tags(sentence);
        assert!(result.is_ok());
        
        let nmea_message = result.unwrap();
        assert!(nmea_message.tag_block.is_some());
        
        let tag_block = nmea_message.tag_block.unwrap();
        assert_eq!(tag_block.timestamp, Some(1241544035));
        assert_eq!(tag_block.line_count, Some(157036));
        assert_eq!(tag_block.source, Some("r003669945".to_string()));
        
        assert!(tag_block.grouping.is_some());
        let grouping = tag_block.grouping.unwrap();
        assert_eq!(grouping.sentence_number, 1);
        assert_eq!(grouping.total_sentences, 2);
        assert_eq!(grouping.group_id, 73874);
        
        // The parsed NMEA message should still be valid
        match nmea_message.message {
            ParsedMessage::VesselDynamicData(_) => {},
            _ => panic!("Expected VesselDynamicData message"),
        }
    }
    
    #[test]
    fn test_parse_sentence_without_tag_block() {
        let mut p = NmeaParser::new();
        let sentence = "!AIVDM,1,1,,B,15N4cJ`005Jrek0H@9n`DW5608EP,0*13";
        
        let result = p.parse_sentence_with_tags(sentence);
        assert!(result.is_ok());
        
        let nmea_message = result.unwrap();
        assert!(nmea_message.tag_block.is_none());
        
        // The parsed NMEA message should still be valid
        match nmea_message.message {
            ParsedMessage::VesselDynamicData(_) => {},
            _ => panic!("Expected VesselDynamicData message"),
        }
    }
    
    #[test]
    fn test_parse_sentence_backward_compatibility() {
        let mut p = NmeaParser::new();
        let sentence_with_tag = r"\c:1241544035*53\!AIVDM,1,1,,B,15N4cJ`005Jrek0H@9n`DW5608EP,0*13";
        let sentence_without_tag = "!AIVDM,1,1,,B,15N4cJ`005Jrek0H@9n`DW5608EP,0*13";
        
        // Both should return the same parsed message when using parse_sentence
        let result1 = p.parse_sentence(sentence_with_tag);
        let result2 = p.parse_sentence(sentence_without_tag);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        // Both results should be identical (tag block is ignored in the old API)
        assert_eq!(result1.unwrap(), result2.unwrap());
    }
    
    #[test]
    fn test_parse_gnss_sentence_with_tag_block() {
        let mut p = NmeaParser::new();
        let sentence = r"\s:station1,t:test*3C\$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";
        
        let result = p.parse_sentence_with_tags(sentence);
        assert!(result.is_ok());
        
        let nmea_message = result.unwrap();
        assert!(nmea_message.tag_block.is_some());
        
        let tag_block = nmea_message.tag_block.unwrap();
        assert_eq!(tag_block.source, Some("station1".to_string()));
        assert_eq!(tag_block.text, Some("test".to_string()));
        
        // The parsed NMEA message should be RMC data
        match nmea_message.message {
            ParsedMessage::Rmc(_) => {},
            _ => panic!("Expected RMC message"),
        }
    }
    
    #[test]
    fn test_parse_invalid_tag_block() {
        let mut p = NmeaParser::new();
        
        // Tag block not properly closed
        let sentence1 = r"\c:1241544035*53!AIVDM,1,1,,B,15N4cJ`005Jrek0H@9n`DW5608EP,0*13";
        let result1 = p.parse_sentence_with_tags(sentence1);
        assert!(result1.is_err());
        
        // Invalid checksum in tag block
        let sentence2 = r"\c:1241544035*FF\!AIVDM,1,1,,B,15N4cJ`005Jrek0H@9n`DW5608EP,0*13";
        let result2 = p.parse_sentence_with_tags(sentence2);
        assert!(result2.is_err());
    }
    
    #[test]
    fn test_tag_block_with_multipart_message() {
        let mut p = NmeaParser::new();
        
        // First part of multipart message with tag block
        let sentence1 = r"\g:1-2-12345*1B\!AIVDM,2,1,3,B,55P5TL01VIaAL@7WKO@mBplU@<PDhh000000001S;AJ::4A80?4i@E53,0*3E";
        let result1 = p.parse_sentence_with_tags(sentence1);
        assert!(result1.is_ok());
        
        let nmea_message1 = result1.unwrap();
        assert!(nmea_message1.tag_block.is_some());
        assert_eq!(nmea_message1.message, ParsedMessage::Incomplete);
        
        // Second part of multipart message with different tag block
        let sentence2 = r"\g:2-2-12345*1A\!AIVDM,2,2,3,B,1@0000000000000,2*55";
        let result2 = p.parse_sentence_with_tags(sentence2);
        assert!(result2.is_ok());
        
        let nmea_message2 = result2.unwrap();
        assert!(nmea_message2.tag_block.is_some());
        
        // The second part should complete the message
        match nmea_message2.message {
            ParsedMessage::VesselStaticData(_) => {},
            _ => panic!("Expected VesselStaticData message after completing multipart"),
        }
        
        // Check that grouping information is correctly parsed
        let tag_block1 = nmea_message1.tag_block.unwrap();
        let grouping1 = tag_block1.grouping.unwrap();
        assert_eq!(grouping1.sentence_number, 1);
        assert_eq!(grouping1.total_sentences, 2);
        assert_eq!(grouping1.group_id, 12345);
        
        let tag_block2 = nmea_message2.tag_block.unwrap();
        let grouping2 = tag_block2.grouping.unwrap();
        assert_eq!(grouping2.sentence_number, 2);
        assert_eq!(grouping2.total_sentences, 2);
        assert_eq!(grouping2.group_id, 12345);
    }
}

/// Parse a single NMEA sentence with tag block support.
/// This is a convenience function that creates a parser instance and parses the sentence.
/// For parsing multiple sentences efficiently, use NmeaParser directly.
pub fn parse_sentence_with_tags(sentence: &str) -> Result<NmeaMessage, ParseError> {
    let mut parser = NmeaParser::new();
    parser.parse_sentence_with_tags(sentence)
}
