//! JSON serialization structures for NMEA messages
//! This module provides JSON-serializable equivalents of the main NMEA message types

use crate::ParsedMessage;
use crate::tag_block::TagBlock;
use serde::{Deserialize, Serialize};
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;

/// Augmentation information for modified/enhanced data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Augmentation {
    pub timestamp: i64,
    pub description: String,
}

/// Serializable version of NmeaMessage for JSON output
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonNmeaMessage {
    pub raw_sentence: String,
    pub tag_block: Option<TagBlock>,
    pub message: JsonParsedMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub augmentations: Option<Vec<Augmentation>>,
}

/// Serializable version of ParsedMessage for JSON output
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum JsonParsedMessage {
    // AIS Messages (simplified for JSON)
    VesselDynamicData {
        mmsi: u32,
        latitude: Option<f64>,
        longitude: Option<f64>,
        speed_over_ground: Option<f64>,
        course_over_ground: Option<f64>,
        true_heading: Option<u16>,
        timestamp: Option<u8>,
        message_type: u8,
    },
    VesselStaticData {
        mmsi: u32,
        vessel_name: Option<String>,
        call_sign: Option<String>,
        vessel_type: Option<u8>,
        dimensions: Option<String>,
        message_type: u8,
    },
    BaseStationReport {
        mmsi: u32,
        latitude: Option<f64>,
        longitude: Option<f64>,
        timestamp: Option<i64>,
        message_type: u8,
    },
    BinaryBroadcastMessage {
        mmsi: u32,
        dac: u16,
        fid: u8,
        data_hex: String,
        data_bit_length: usize,
        message_type: u8,
        #[serde(skip_serializing_if = "Option::is_none")]
        parsed_payload: Option<serde_json::Value>,
    },
    // GNSS Messages (simplified for JSON)
    Gga {
        latitude: Option<f64>,
        longitude: Option<f64>,
        fix_quality: Option<u8>,
        num_satellites: Option<u8>,
        hdop: Option<f64>,
        altitude: Option<f64>,
        timestamp: Option<i64>,
    },
    Rmc {
        latitude: Option<f64>,
        longitude: Option<f64>,
        speed: Option<f64>,
        course: Option<f64>,
        date: Option<String>,
        timestamp: Option<i64>,
        status: Option<char>,
    },
    // Generic message for unsupported types
    Unknown {
        sentence_type: String,
        raw_data: String,
    },
}

impl JsonNmeaMessage {
    pub fn new(message: ParsedMessage, tag_block: Option<TagBlock>, raw_sentence: String) -> Self {
        JsonNmeaMessage {
            raw_sentence,
            tag_block,
            message: JsonParsedMessage::from(message),
            augmentations: None,
        }
    }
    
    pub fn with_augmentations(mut self, augmentations: Vec<Augmentation>) -> Self {
        self.augmentations = Some(augmentations);
        self
    }
}

impl From<ParsedMessage> for JsonParsedMessage {
    fn from(msg: ParsedMessage) -> Self {
        match msg {
            ParsedMessage::VesselDynamicData(vdd) => JsonParsedMessage::VesselDynamicData {
                mmsi: vdd.mmsi,
                latitude: vdd.latitude,
                longitude: vdd.longitude,
                speed_over_ground: vdd.sog_knots,
                course_over_ground: vdd.cog,
                true_heading: vdd.heading_true.map(|h| h as u16),
                timestamp: Some(vdd.timestamp_seconds),
                message_type: 1, // Type 1/2/3 dynamic data
            },
            ParsedMessage::VesselStaticData(vsd) => JsonParsedMessage::VesselStaticData {
                mmsi: vsd.mmsi,
                vessel_name: vsd.name.clone(),
                call_sign: vsd.call_sign.clone(),
                vessel_type: Some(vsd.ship_type as u8),
                dimensions: {
                    if let (Some(bow), Some(stern), Some(port), Some(starboard)) = 
                        (vsd.dimension_to_bow, vsd.dimension_to_stern, vsd.dimension_to_port, vsd.dimension_to_starboard) {
                        Some(format!("bow:{}m,stern:{}m,port:{}m,starboard:{}m", bow, stern, port, starboard))
                    } else {
                        None
                    }
                },
                message_type: 5, // Type 5 static data
            },
            ParsedMessage::BaseStationReport(bsr) => JsonParsedMessage::BaseStationReport {
                mmsi: bsr.mmsi,
                latitude: bsr.latitude,
                longitude: bsr.longitude,
                timestamp: bsr.timestamp.map(|ts| ts.timestamp()),
                message_type: 4, // Type 4 base station report
            },
            ParsedMessage::BinaryBroadcastMessage(bbm) => {
                // Convert binary data to hex string
                let data_hex = bbm.data.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join("");
                
                // Convert parsed payload to JSON value if present
                let parsed_payload = bbm.parsed_payload.as_ref()
                    .and_then(|payload| serde_json::to_value(payload).ok());
                
                JsonParsedMessage::BinaryBroadcastMessage {
                    mmsi: bbm.mmsi,
                    dac: bbm.dac,
                    fid: bbm.fid,
                    data_hex,
                    data_bit_length: bbm.data_bit_length,
                    message_type: 8, // Type 8 binary broadcast
                    parsed_payload,
                }
            },
            ParsedMessage::Gga(gga) => JsonParsedMessage::Gga {
                latitude: gga.latitude,
                longitude: gga.longitude,
                fix_quality: Some(gga.quality as u8),
                num_satellites: gga.satellite_count,
                hdop: gga.hdop,
                altitude: gga.altitude,
                timestamp: gga.timestamp.map(|ts| ts.timestamp()),
            },
            ParsedMessage::Rmc(rmc) => JsonParsedMessage::Rmc {
                latitude: rmc.latitude,
                longitude: rmc.longitude,
                speed: rmc.sog_knots,
                course: rmc.bearing,
                date: None, // RmcData doesn't have a separate date field in this version
                timestamp: rmc.timestamp.map(|ts| ts.timestamp()),
                status: rmc.status_active.map(|active| if active { 'A' } else { 'V' }),
            },
            // For all other message types, create a generic representation
            _ => {
                let sentence_type = match &msg {
                    ParsedMessage::BinaryAddressedMessage(_) => "BinaryAddressedMessage",
                    ParsedMessage::StandardSarAircraftPositionReport(_) => "StandardSarAircraftPositionReport",
                    ParsedMessage::UtcDateInquiry(_) => "UtcDateInquiry",
                    ParsedMessage::AddressedSafetyRelatedMessage(_) => "AddressedSafetyRelatedMessage",
                    ParsedMessage::SafetyRelatedAcknowledgement(_) => "SafetyRelatedAcknowledgement",
                    ParsedMessage::SafetyRelatedBroadcastMessage(_) => "SafetyRelatedBroadcastMessage",
                    ParsedMessage::Interrogation(_) => "Interrogation",
                    ParsedMessage::AssignmentModeCommand(_) => "AssignmentModeCommand",
                    ParsedMessage::DgnssBroadcastBinaryMessage(_) => "DgnssBroadcastBinaryMessage",
                    ParsedMessage::UtcDateResponse(_) => "UtcDateResponse",
                    ParsedMessage::DataLinkManagementMessage(_) => "DataLinkManagementMessage",
                    ParsedMessage::AidToNavigationReport(_) => "AidToNavigationReport",
                    ParsedMessage::ChannelManagement(_) => "ChannelManagement",
                    ParsedMessage::GroupAssignmentCommand(_) => "GroupAssignmentCommand",
                    ParsedMessage::SingleSlotBinaryMessage(_) => "SingleSlotBinaryMessage",
                    ParsedMessage::MultipleSlotBinaryMessage(_) => "MultipleSlotBinaryMessage",
                    ParsedMessage::Alm(_) => "Alm",
                    ParsedMessage::Dbs(_) => "Dbs",
                    ParsedMessage::Dpt(_) => "Dpt",
                    ParsedMessage::Dtm(_) => "Dtm",
                    ParsedMessage::Gll(_) => "Gll",
                    ParsedMessage::Gns(_) => "Gns",
                    ParsedMessage::Gsa(_) => "Gsa",
                    ParsedMessage::Gsv(_) => "Gsv",
                    ParsedMessage::Hdt(_) => "Hdt",
                    ParsedMessage::Mss(_) => "Mss",
                    ParsedMessage::Mtw(_) => "Mtw",
                    ParsedMessage::Mwv(_) => "Mwv",
                    ParsedMessage::Stn(_) => "Stn",
                    ParsedMessage::Vbw(_) => "Vbw",
                    ParsedMessage::Vhw(_) => "Vhw",
                    ParsedMessage::Vtg(_) => "Vtg",
                    ParsedMessage::Zda(_) => "Zda",
                    _ => "Unknown",
                };
                JsonParsedMessage::Unknown {
                    sentence_type: sentence_type.to_string(),
                    raw_data: format!("{:?}", msg),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_serialization() {
        // Test with a basic AIS message structure
        use crate::ais::VesselDynamicData;
        use crate::ais::{AisClass, NavigationStatus, Station};
        
        let vdd = VesselDynamicData {
            own_vessel: false,
            station: Station::Other,
            ais_type: AisClass::ClassA,
            mmsi: 12345,
            nav_status: NavigationStatus::UnderWayUsingEngine,
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
            sog_knots: Some(10.5),
            cog: Some(45.0),
            heading_true: Some(50.0),
            timestamp_seconds: 30,
            ..Default::default()
        };
        
        let parsed_msg = ParsedMessage::VesselDynamicData(vdd);
        let json_msg = JsonNmeaMessage::new(parsed_msg, None, "test sentence".to_string());
        let json_str = serde_json::to_string_pretty(&json_msg).unwrap();
        
        // Verify we can deserialize back
        let _: JsonNmeaMessage = serde_json::from_str(&json_str).unwrap();
        
        assert!(json_str.contains("VesselDynamicData"));
        assert!(json_str.contains("12345"));
    }
}