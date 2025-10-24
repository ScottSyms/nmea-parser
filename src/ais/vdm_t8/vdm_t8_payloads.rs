/*
Copyright 2025

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

//! AIS Type 8 Binary Broadcast Message Payload Parsers
//! 
//! This module handles decoding of specific DAC/FID combinations for Type 8 messages.
//! Each DAC/FID combination has a different binary layout and interpretation.

use super::*;
use serde::{Deserialize, Serialize};

// -------------------------------------------------------------------------------------------------

/// Parsed payload data for Type 8 messages
/// The variant depends on the DAC (Designated Area Code) and FID (Functional ID)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "format")]
pub enum Type8Payload {
    /// DAC=1, FID=11: Meteorological and Hydrological Data (deprecated, use FID=31)
    MeteoHydro11(MeteoHydroData11),
    
    /// DAC=1, FID=31: Meteorological and Hydrological Data (current standard)
    MeteoHydro31(MeteoHydroData31),
    
    /// Unknown or unsupported DAC/FID combination
    Unsupported {
        dac: u16,
        fid: u8,
    },
}

// -------------------------------------------------------------------------------------------------

/// DAC=1, FID=11: Meteorological/Hydrological Data (deprecated)
/// Fixed length: 352 bits (44 bytes)
/// This format has been deprecated by IMO in favor of FID=31
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeteoHydroData11 {
    /// Latitude in degrees (-90.0 to +90.0), None if N/A
    pub latitude: Option<f64>,
    
    /// Longitude in degrees (-180.0 to +180.0), None if N/A
    pub longitude: Option<f64>,
    
    /// Day of month (1-31), None if N/A
    pub day: Option<u8>,
    
    /// Hour (0-23), None if N/A
    pub hour: Option<u8>,
    
    /// Minute (0-59), None if N/A
    pub minute: Option<u8>,
    
    /// Average wind speed in knots (0-120), None if N/A
    pub wind_speed_avg: Option<u8>,
    
    /// Wind gust speed in knots (0-120), None if N/A
    pub wind_gust: Option<u8>,
    
    /// Wind direction in degrees (0-359), None if N/A
    pub wind_direction: Option<u16>,
    
    /// Wind gust direction in degrees (0-359), None if N/A
    pub wind_gust_direction: Option<u16>,
    
    /// Air temperature in degrees Celsius (-60.0 to +60.0), None if N/A
    pub air_temperature: Option<f32>,
    
    /// Relative humidity in percent (0-100), None if N/A
    pub humidity: Option<u8>,
    
    /// Dew point in degrees Celsius (-20.0 to +50.0), None if N/A
    pub dew_point: Option<f32>,
    
    /// Air pressure in hPa (800-1200), None if N/A
    pub air_pressure: Option<u16>,
    
    /// Pressure tendency (0=steady, 1=decreasing, 2=increasing), None if N/A
    pub pressure_tendency: Option<u8>,
    
    /// Horizontal visibility in nautical miles (0-25.0), None if N/A
    pub visibility: Option<f32>,
    
    /// Water level in meters (-10.0 to +30.0), None if N/A
    pub water_level: Option<f32>,
    
    /// Water level trend (0=steady, 1=decreasing, 2=increasing), None if N/A
    pub water_level_trend: Option<u8>,
    
    /// Surface current speed in knots (0-25.0), None if N/A
    pub surface_current_speed: Option<f32>,
    
    /// Surface current direction in degrees (0-359), None if N/A
    pub surface_current_direction: Option<u16>,
    
    /// Current speed #2 in knots (0-25.0), None if N/A
    pub current_speed_2: Option<f32>,
    
    /// Current direction #2 in degrees (0-359), None if N/A
    pub current_direction_2: Option<u16>,
    
    /// Measurement depth #2 in meters (0-30), None if N/A
    pub current_depth_2: Option<f32>,
    
    /// Current speed #3 in knots (0-25.0), None if N/A
    pub current_speed_3: Option<f32>,
    
    /// Current direction #3 in degrees (0-359), None if N/A
    pub current_direction_3: Option<u16>,
    
    /// Measurement depth #3 in meters (0-30), None if N/A
    pub current_depth_3: Option<f32>,
    
    /// Significant wave height in meters (0-25), None if N/A
    pub wave_height: Option<f32>,
    
    /// Wave period in seconds (0-60), None if N/A
    pub wave_period: Option<u8>,
    
    /// Wave direction in degrees (0-359), None if N/A
    pub wave_direction: Option<u16>,
    
    /// Swell height in meters (0-25), None if N/A
    pub swell_height: Option<f32>,
    
    /// Swell period in seconds (0-60), None if N/A
    pub swell_period: Option<u8>,
    
    /// Swell direction in degrees (0-359), None if N/A
    pub swell_direction: Option<u16>,
    
    /// Sea state (Beaufort scale 0-12), None if N/A
    pub sea_state: Option<u8>,
    
    /// Water temperature in degrees Celsius (-10.0 to +50.0), None if N/A
    pub water_temperature: Option<f32>,
    
    /// Precipitation type (0-7), None if N/A
    pub precipitation_type: Option<u8>,
    
    /// Salinity in percent (0-50.0), None if N/A
    pub salinity: Option<f32>,
    
    /// Ice presence (0=No, 1=Yes), None if N/A
    pub ice: Option<u8>,
}

// -------------------------------------------------------------------------------------------------

/// DAC=1, FID=31: Meteorological and Hydrological Data (current standard)
/// Fixed length: 360 bits (45 bytes)
/// Supersedes FID=11 with better precision and clearer N/A values
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeteoHydroData31 {
    /// Longitude in degrees (-180.0 to +180.0), None if N/A
    pub longitude: Option<f64>,
    
    /// Latitude in degrees (-90.0 to +90.0), None if N/A
    pub latitude: Option<f64>,
    
    /// Position accuracy (true = high <=10m, false = low >10m)
    pub position_accuracy: bool,
    
    /// Day of month (1-31), None if N/A
    pub day: Option<u8>,
    
    /// Hour (0-23), None if N/A
    pub hour: Option<u8>,
    
    /// Minute (0-59), None if N/A
    pub minute: Option<u8>,
    
    /// Average wind speed in knots (0-126), None if N/A
    pub wind_speed_avg: Option<u8>,
    
    /// Wind gust speed in knots (0-126), None if N/A
    pub wind_gust: Option<u8>,
    
    /// Wind direction in degrees (0-359), None if N/A
    pub wind_direction: Option<u16>,
    
    /// Wind gust direction in degrees (0-359), None if N/A
    pub wind_gust_direction: Option<u16>,
    
    /// Air temperature in degrees Celsius (-60.0 to +60.0), None if N/A
    pub air_temperature: Option<f32>,
    
    /// Relative humidity in percent (0-100), None if N/A
    pub humidity: Option<u8>,
    
    /// Dew point in degrees Celsius (-20.0 to +50.0), None if N/A
    pub dew_point: Option<f32>,
    
    /// Air pressure in hPa (0-1201+), None if N/A
    pub air_pressure: Option<u16>,
    
    /// Pressure tendency (0=steady, 1=decreasing, 2=increasing), None if N/A
    pub pressure_tendency: Option<u8>,
    
    /// Visibility greater than reported value
    pub visibility_greater_than: bool,
    
    /// Horizontal visibility in nautical miles (0-12.7), None if N/A
    pub visibility: Option<f32>,
    
    /// Water level in meters (-10.0 to +30.0), None if N/A
    pub water_level: Option<f32>,
    
    /// Water level trend (0=steady, 1=decreasing, 2=increasing), None if N/A
    pub water_level_trend: Option<u8>,
    
    /// Surface current speed in knots (0-25.1), None if N/A
    pub surface_current_speed: Option<f32>,
    
    /// Surface current direction in degrees (0-359), None if N/A
    pub surface_current_direction: Option<u16>,
    
    /// Current speed #2 in knots (0-25.1), None if N/A
    pub current_speed_2: Option<f32>,
    
    /// Current direction #2 in degrees (0-359), None if N/A
    pub current_direction_2: Option<u16>,
    
    /// Measurement depth #2 in meters (0-3.0), None if N/A
    pub current_depth_2: Option<f32>,
    
    /// Current speed #3 in knots (0-25.1), None if N/A
    pub current_speed_3: Option<f32>,
    
    /// Current direction #3 in degrees (0-359), None if N/A
    pub current_direction_3: Option<u16>,
    
    /// Measurement depth #3 in meters (0-3.0), None if N/A
    pub current_depth_3: Option<f32>,
    
    /// Significant wave height in meters (0-25.1), None if N/A
    pub wave_height: Option<f32>,
    
    /// Wave period in seconds (0-60), None if N/A
    pub wave_period: Option<u8>,
    
    /// Wave direction in degrees (0-359), None if N/A
    pub wave_direction: Option<u16>,
    
    /// Swell height in meters (0-25.1), None if N/A
    pub swell_height: Option<f32>,
    
    /// Swell period in seconds (0-60), None if N/A
    pub swell_period: Option<u8>,
    
    /// Swell direction in degrees (0-359), None if N/A
    pub swell_direction: Option<u16>,
    
    /// Sea state (Beaufort scale 0-12), None if N/A
    pub sea_state: Option<u8>,
    
    /// Water temperature in degrees Celsius (-10.0 to +50.0), None if N/A
    pub water_temperature: Option<f32>,
    
    /// Precipitation type (0-7), None if N/A
    pub precipitation_type: Option<u8>,
    
    /// Salinity in percent (0-51.0), None if N/A
    pub salinity: Option<f32>,
    
    /// Ice presence (0=No, 1=Yes), None if N/A
    pub ice: Option<u8>,
}

// -------------------------------------------------------------------------------------------------

/// Parse Type 8 payload based on DAC and FID
pub fn parse_payload(dac: u16, fid: u8, bv: &BitVec, bit_offset: usize) -> Option<Type8Payload> {
    match (dac, fid) {
        (1, 11) => parse_meteo_hydro_11(bv, bit_offset).map(Type8Payload::MeteoHydro11),
        (1, 31) => parse_meteo_hydro_31(bv, bit_offset).map(Type8Payload::MeteoHydro31),
        _ => Some(Type8Payload::Unsupported { dac, fid }),
    }
}

// -------------------------------------------------------------------------------------------------

/// Parse DAC=1, FID=11 payload (352 bits starting at bit_offset)
fn parse_meteo_hydro_11(bv: &BitVec, offset: usize) -> Option<MeteoHydroData11> {
    // Need 352 bits for complete message
    if bv.len() < offset + 352 {
        return None;
    }
    
    // Latitude: bits 56-79 (24 bits), signed, minutes * 0.001
    let lat_raw = pick_i64(bv, offset, 24);
    let latitude = if lat_raw == 0x7FFFFF {
        None
    } else {
        Some((lat_raw as f64) / 60000.0) // Convert from minutes*1000 to degrees
    };
    
    // Longitude: bits 80-104 (25 bits), signed, minutes * 0.001
    let lon_raw = pick_i64(bv, offset + 24, 25);
    let longitude = if lon_raw == 0x1FFFFFF {
        None
    } else {
        Some((lon_raw as f64) / 60000.0) // Convert from minutes*1000 to degrees
    };
    
    // Day: bits 105-109 (5 bits)
    let day = pick_u64(bv, offset + 49, 5) as u8;
    let day = if day == 31 { None } else { Some(day) };
    
    // Hour: bits 110-114 (5 bits)
    let hour = pick_u64(bv, offset + 54, 5) as u8;
    let hour = if hour == 31 { None } else { Some(hour) };
    
    // Minute: bits 115-120 (6 bits)
    let minute = pick_u64(bv, offset + 59, 6) as u8;
    let minute = if minute == 63 { None } else { Some(minute) };
    
    // Wind speed: bits 121-127 (7 bits)
    let wspeed = pick_u64(bv, offset + 65, 7) as u8;
    let wind_speed_avg = if wspeed == 127 { None } else { Some(wspeed) };
    
    // Wind gust: bits 128-134 (7 bits)
    let wgust = pick_u64(bv, offset + 72, 7) as u8;
    let wind_gust = if wgust == 127 { None } else { Some(wgust) };
    
    // Wind direction: bits 135-143 (9 bits)
    let wdir = pick_u64(bv, offset + 79, 9) as u16;
    let wind_direction = if wdir >= 511 { None } else { Some(wdir) };
    
    // Wind gust direction: bits 144-152 (9 bits)
    let wgustdir = pick_u64(bv, offset + 88, 9) as u16;
    let wind_gust_direction = if wgustdir >= 511 { None } else { Some(wgustdir) };
    
    // Air temperature: bits 153-163 (11 bits), 0.1 deg C, -60 to +60
    let temp_raw = pick_u64(bv, offset + 97, 11) as i16;
    let air_temperature = if temp_raw == 2047 {
        None
    } else {
        Some((temp_raw as f32) * 0.1 - 60.0)
    };
    
    // Humidity: bits 164-170 (7 bits)
    let humidity = pick_u64(bv, offset + 108, 7) as u8;
    let humidity = if humidity == 127 { None } else { Some(humidity) };
    
    // Dew point: bits 171-180 (10 bits), 0.1 deg C, -20 to +50
    let dew_raw = pick_u64(bv, offset + 115, 10) as u16;
    let dew_point = if dew_raw == 1023 {
        None
    } else {
        Some((dew_raw as f32) * 0.1 - 20.0)
    };
    
    // Air pressure: bits 181-189 (9 bits), 1 hPa, 800-1200
    let pressure_raw = pick_u64(bv, offset + 125, 9) as u16;
    let air_pressure = if pressure_raw == 511 {
        None
    } else {
        Some(pressure_raw + 800)
    };
    
    // Pressure tendency: bits 190-191 (2 bits)
    let ptend = pick_u64(bv, offset + 134, 2) as u8;
    let pressure_tendency = if ptend == 3 { None } else { Some(ptend) };
    
    // Visibility: bits 192-199 (8 bits), 0.1 nm, 0-25.0
    let vis_raw = pick_u64(bv, offset + 136, 8) as u8;
    let visibility = if vis_raw == 255 {
        None
    } else {
        Some((vis_raw as f32) * 0.1)
    };
    
    // Water level: bits 200-208 (9 bits), signed, 0.1m, -10 to +30
    let wlevel_raw = pick_i64(bv, offset + 144, 9) as i16;
    let water_level = if wlevel_raw == 511 {
        None
    } else {
        Some((wlevel_raw as f32) * 0.1 - 10.0)
    };
    
    // Water level trend: bits 209-210 (2 bits)
    let wtrend = pick_u64(bv, offset + 153, 2) as u8;
    let water_level_trend = if wtrend == 3 { None } else { Some(wtrend) };
    
    // Surface current speed: bits 211-218 (8 bits), 0.1 knots
    let cspeed = pick_u64(bv, offset + 155, 8) as u8;
    let surface_current_speed = if cspeed == 255 {
        None
    } else {
        Some((cspeed as f32) * 0.1)
    };
    
    // Surface current direction: bits 219-227 (9 bits)
    let cdir = pick_u64(bv, offset + 163, 9) as u16;
    let surface_current_direction = if cdir >= 511 { None } else { Some(cdir) };
    
    // Current speed #2: bits 228-235 (8 bits), 0.1 knots
    let cspeed2 = pick_u64(bv, offset + 172, 8) as u8;
    let current_speed_2 = if cspeed2 == 255 {
        None
    } else {
        Some((cspeed2 as f32) * 0.1)
    };
    
    // Current direction #2: bits 236-244 (9 bits)
    let cdir2 = pick_u64(bv, offset + 180, 9) as u16;
    let current_direction_2 = if cdir2 >= 511 { None } else { Some(cdir2) };
    
    // Current depth #2: bits 245-249 (5 bits), 0.1m
    let cdepth2 = pick_u64(bv, offset + 189, 5) as u8;
    let current_depth_2 = if cdepth2 == 31 {
        None
    } else {
        Some((cdepth2 as f32) * 0.1)
    };
    
    // Current speed #3: bits 250-257 (8 bits), 0.1 knots
    let cspeed3 = pick_u64(bv, offset + 194, 8) as u8;
    let current_speed_3 = if cspeed3 == 255 {
        None
    } else {
        Some((cspeed3 as f32) * 0.1)
    };
    
    // Current direction #3: bits 258-266 (9 bits)
    let cdir3 = pick_u64(bv, offset + 202, 9) as u16;
    let current_direction_3 = if cdir3 >= 511 { None } else { Some(cdir3) };
    
    // Current depth #3: bits 267-271 (5 bits), 0.1m
    let cdepth3 = pick_u64(bv, offset + 211, 5) as u8;
    let current_depth_3 = if cdepth3 == 31 {
        None
    } else {
        Some((cdepth3 as f32) * 0.1)
    };
    
    // Wave height: bits 272-279 (8 bits), 0.1m
    let wheight = pick_u64(bv, offset + 216, 8) as u8;
    let wave_height = if wheight == 255 {
        None
    } else {
        Some((wheight as f32) * 0.1)
    };
    
    // Wave period: bits 280-285 (6 bits)
    let wperiod = pick_u64(bv, offset + 224, 6) as u8;
    let wave_period = if wperiod == 63 { None } else { Some(wperiod) };
    
    // Wave direction: bits 286-294 (9 bits)
    let wdir_wave = pick_u64(bv, offset + 230, 9) as u16;
    let wave_direction = if wdir_wave >= 511 { None } else { Some(wdir_wave) };
    
    // Swell height: bits 295-302 (8 bits), 0.1m
    let sheight = pick_u64(bv, offset + 239, 8) as u8;
    let swell_height = if sheight == 255 {
        None
    } else {
        Some((sheight as f32) * 0.1)
    };
    
    // Swell period: bits 303-308 (6 bits)
    let speriod = pick_u64(bv, offset + 247, 6) as u8;
    let swell_period = if speriod == 63 { None } else { Some(speriod) };
    
    // Swell direction: bits 309-317 (9 bits)
    let sdir = pick_u64(bv, offset + 253, 9) as u16;
    let swell_direction = if sdir >= 511 { None } else { Some(sdir) };
    
    // Sea state: bits 318-321 (4 bits), Beaufort scale
    let seastate = pick_u64(bv, offset + 262, 4) as u8;
    let sea_state = if seastate >= 13 { None } else { Some(seastate) };
    
    // Water temperature: bits 322-331 (10 bits), 0.1 deg C, -10 to +50
    let wtemp_raw = pick_u64(bv, offset + 266, 10) as u16;
    let water_temperature = if wtemp_raw == 1023 {
        None
    } else {
        Some((wtemp_raw as f32) * 0.1 - 10.0)
    };
    
    // Precipitation type: bits 332-334 (3 bits)
    let precip = pick_u64(bv, offset + 276, 3) as u8;
    let precipitation_type = if precip == 7 { None } else { Some(precip) };
    
    // Salinity: bits 335-343 (9 bits), 0.1%
    let salinity_raw = pick_u64(bv, offset + 279, 9) as u16;
    let salinity = if salinity_raw >= 511 {
        None
    } else {
        Some((salinity_raw as f32) * 0.1)
    };
    
    // Ice: bits 344-345 (2 bits)
    let ice_raw = pick_u64(bv, offset + 288, 2) as u8;
    let ice = if ice_raw == 3 { None } else { Some(ice_raw) };
    
    Some(MeteoHydroData11 {
        latitude,
        longitude,
        day,
        hour,
        minute,
        wind_speed_avg,
        wind_gust,
        wind_direction,
        wind_gust_direction,
        air_temperature,
        humidity,
        dew_point,
        air_pressure,
        pressure_tendency,
        visibility,
        water_level,
        water_level_trend,
        surface_current_speed,
        surface_current_direction,
        current_speed_2,
        current_direction_2,
        current_depth_2,
        current_speed_3,
        current_direction_3,
        current_depth_3,
        wave_height,
        wave_period,
        wave_direction,
        swell_height,
        swell_period,
        swell_direction,
        sea_state,
        water_temperature,
        precipitation_type,
        salinity,
        ice,
    })
}

// -------------------------------------------------------------------------------------------------

/// Parse DAC=1, FID=31 payload (360 bits starting at bit_offset)
fn parse_meteo_hydro_31(bv: &BitVec, offset: usize) -> Option<MeteoHydroData31> {
    // Need 360 bits for complete message
    if bv.len() < offset + 360 {
        return None;
    }
    
    // This is a simplified implementation - full implementation would follow the same
    // pattern as FID=11 but with the field layout from FID=31 specification
    
    // For now, return None to indicate parsing not yet complete
    // TODO: Implement full FID=31 parsing following specification
    None
}
