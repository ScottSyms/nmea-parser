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

//! NMEA Tag Block support for NMEA 4.10 standard
//!
//! This module provides support for parsing NMEA tag blocks which contain additional
//! metadata for NMEA sentences. Tag blocks are enclosed in backslashes and contain
//! comma-separated fields with type indicators followed by colons and values.
//!
//! Example: `\g:1-2-73874,n:157036,s:r003669945,c:1241544035*4A\`

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::error::ParseError;

/// Represents sentence grouping information from NMEA 4.10 tag blocks
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SentenceGrouping {
    /// Sentence number in the group
    pub sentence_number: u32,
    /// Total number of sentences in the group
    pub total_sentences: u32,
    /// Group identifier
    pub group_id: u32,
}

/// Represents a parsed NMEA tag block
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TagBlock {
    /// UNIX timestamp in seconds or milliseconds (c field)
    pub timestamp: Option<u64>,
    
    /// Destination identifier (d field) - at most 15 characters
    pub destination: Option<String>,
    
    /// Sentence grouping information (g field) 
    pub grouping: Option<SentenceGrouping>,
    
    /// Line count (n field)
    pub line_count: Option<u32>,
    
    /// Relative time (r field)
    pub relative_time: Option<u32>,
    
    /// Source/station identifier (s field)
    pub source: Option<String>,
    
    /// Text string (t/i field) - at most 15 characters
    pub text: Option<String>,
}

impl TagBlock {
    /// Create a new empty tag block
    pub fn new() -> Self {
        TagBlock {
            timestamp: None,
            destination: None,
            grouping: None,
            line_count: None,
            relative_time: None,
            source: None,
            text: None,
        }
    }
    
    /// Parse a tag block from a string
    /// 
    /// # Arguments
    /// * `tag_block_str` - The tag block string including the opening and closing backslashes
    /// 
    /// # Returns
    /// * `Ok(TagBlock)` - Successfully parsed tag block
    /// * `Err(ParseError)` - Error parsing the tag block
    pub fn parse(tag_block_str: &str) -> Result<TagBlock, ParseError> {
        // Check that the string starts and ends with backslashes
        if !tag_block_str.starts_with('\\') || !tag_block_str.ends_with('\\') {
            return Err(ParseError::InvalidSentence(
                "Tag block must start and end with backslashes".to_string()
            ));
        }
        
        // Remove the outer backslashes
        let content = &tag_block_str[1..tag_block_str.len()-1];
        
        // Find the checksum position
        let (fields_str, checksum_str) = if let Some(asterisk_pos) = content.rfind('*') {
            if asterisk_pos + 2 < content.len() {
                (
                    &content[0..asterisk_pos],
                    &content[asterisk_pos + 1..]
                )
            } else {
                return Err(ParseError::InvalidSentence(
                    "Invalid checksum in tag block".to_string()
                ));
            }
        } else {
            return Err(ParseError::InvalidSentence(
                "Missing checksum in tag block".to_string()
            ));
        };
        
        // Validate checksum
        let calculated_checksum = Self::calculate_checksum(fields_str);
        let expected_checksum = u8::from_str_radix(checksum_str, 16)
            .map_err(|_| ParseError::InvalidSentence(
                "Invalid checksum format in tag block".to_string()
            ))?;
            
        if calculated_checksum != expected_checksum {
            return Err(ParseError::CorruptedSentence(format!(
                "Tag block checksum mismatch: calculated {:02X}, expected {:02X}",
                calculated_checksum, expected_checksum
            )));
        }
        
        // Parse fields
        let mut tag_block = TagBlock::new();
        
        for field in fields_str.split(',') {
            if field.is_empty() {
                continue;
            }
            
            if let Some(colon_pos) = field.find(':') {
                let field_type = &field[0..colon_pos];
                let field_value = &field[colon_pos + 1..];
                
                match field_type {
                    "c" => {
                        tag_block.timestamp = field_value.parse::<u64>().ok();
                    },
                    "d" => {
                        if field_value.len() <= 15 {
                            tag_block.destination = Some(field_value.to_string());
                        }
                    },
                    "g" => {
                        tag_block.grouping = Self::parse_grouping(field_value)?;
                    },
                    "n" => {
                        tag_block.line_count = field_value.parse::<u32>().ok();
                    },
                    "r" => {
                        tag_block.relative_time = field_value.parse::<u32>().ok();
                    },
                    "s" => {
                        tag_block.source = Some(field_value.to_string());
                    },
                    "t" | "i" => {
                        if field_value.len() <= 15 {
                            tag_block.text = Some(field_value.to_string());
                        }
                    },
                    _ => {
                        // Ignore unknown field types for forward compatibility
                    }
                }
            }
        }
        
        Ok(tag_block)
    }
    
    /// Parse sentence grouping from a string like "1-2-73874"
    fn parse_grouping(value: &str) -> Result<Option<SentenceGrouping>, ParseError> {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() != 3 {
            return Ok(None);
        }
        
        let sentence_number = parts[0].parse::<u32>()
            .map_err(|_| ParseError::InvalidSentence(
                "Invalid sentence number in grouping".to_string()
            ))?;
        let total_sentences = parts[1].parse::<u32>()
            .map_err(|_| ParseError::InvalidSentence(
                "Invalid total sentences in grouping".to_string()
            ))?;
        let group_id = parts[2].parse::<u32>()
            .map_err(|_| ParseError::InvalidSentence(
                "Invalid group ID in grouping".to_string()
            ))?;
            
        Ok(Some(SentenceGrouping {
            sentence_number,
            total_sentences,
            group_id,
        }))
    }
    
    /// Calculate NMEA checksum for tag block fields
    fn calculate_checksum(data: &str) -> u8 {
        let mut checksum = 0u8;
        for byte in data.bytes() {
            checksum ^= byte;
        }
        checksum
    }
}

impl Default for TagBlock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_complete_tag_block() {
        let tag_block_str = r"\g:1-2-73874,n:157036,s:r003669945,c:1241544035*4A\";
        let result = TagBlock::parse(tag_block_str);
        
        assert!(result.is_ok());
        let tag_block = result.unwrap();
        
        assert_eq!(tag_block.timestamp, Some(1241544035));
        assert_eq!(tag_block.line_count, Some(157036));
        assert_eq!(tag_block.source, Some("r003669945".to_string()));
        
        assert!(tag_block.grouping.is_some());
        let grouping = tag_block.grouping.unwrap();
        assert_eq!(grouping.sentence_number, 1);
        assert_eq!(grouping.total_sentences, 2);
        assert_eq!(grouping.group_id, 73874);
    }
    
    #[test]
    fn test_parse_simple_tag_block() {
        let tag_block_str = r"\c:1241544035*53\";
        let result = TagBlock::parse(tag_block_str);
        
        assert!(result.is_ok());
        let tag_block = result.unwrap();
        
        assert_eq!(tag_block.timestamp, Some(1241544035));
        assert_eq!(tag_block.line_count, None);
        assert_eq!(tag_block.source, None);
        assert_eq!(tag_block.grouping, None);
    }
    
    #[test]
    fn test_parse_tag_block_with_text() {
        let tag_block_str = r"\s:station1,t:hello*5A\";
        let result = TagBlock::parse(tag_block_str);
        
        assert!(result.is_ok());
        let tag_block = result.unwrap();
        
        assert_eq!(tag_block.source, Some("station1".to_string()));
        assert_eq!(tag_block.text, Some("hello".to_string()));
    }
    
    #[test]
    fn test_parse_invalid_checksum() {
        let tag_block_str = r"\c:1241544035*FF\";
        let result = TagBlock::parse(tag_block_str);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::CorruptedSentence(_) => {},
            _ => panic!("Expected CorruptedSentence error"),
        }
    }
    
    #[test]
    fn test_parse_missing_backslashes() {
        let tag_block_str = "c:1241544035*53";
        let result = TagBlock::parse(tag_block_str);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_missing_checksum() {
        let tag_block_str = r"\c:1241544035\";
        let result = TagBlock::parse(tag_block_str);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_calculate_checksum() {
        let data = "g:1-2-73874,n:157036,s:r003669945,c:1241544035";
        let checksum = TagBlock::calculate_checksum(data);
        assert_eq!(checksum, 0x4A);
    }
    
    #[test]
    fn test_parse_grouping() {
        let result = TagBlock::parse_grouping("1-2-73874");
        assert!(result.is_ok());
        
        let grouping = result.unwrap().unwrap();
        assert_eq!(grouping.sentence_number, 1);
        assert_eq!(grouping.total_sentences, 2);
        assert_eq!(grouping.group_id, 73874);
    }
    
    #[test]
    fn test_parse_invalid_grouping() {
        let result = TagBlock::parse_grouping("1-2");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}