//! Audio configuration: SampleRate, BufferSize and AudioConfig validation
use crate::errors::InvalidAudioConfigError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleRate {
    Rate44100 = 44100,
    Rate48000 = 48000,
    Rate88200 = 88200,
    Rate96000 = 96000,
}

impl SampleRate {
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn as_f64(self) -> f64 {
        self as u32 as f64
    }

    pub fn try_from_u32(rate: u32) -> Result<Self, InvalidAudioConfigError> {
        match rate {
            44100 => Ok(Self::Rate44100),
            48000 => Ok(Self::Rate48000),
            88200 => Ok(Self::Rate88200),
            96000 => Ok(Self::Rate96000),
            other => Err(InvalidAudioConfigError::UnsupportedSampleRate(other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferSize {
    Size32 = 32,
    Size64 = 64,
    Size128 = 128,
    Size256 = 256,
    Size512 = 512,
}

impl BufferSize {
    pub fn as_usize(self) -> usize {
        self as usize
    }

    pub fn try_from_usize(size: usize) -> Result<Self, InvalidAudioConfigError> {
        match size {
            32 => Ok(Self::Size32),
            64 => Ok(Self::Size64),
            128 => Ok(Self::Size128),
            256 => Ok(Self::Size256),
            512 => Ok(Self::Size512),
            other => Err(InvalidAudioConfigError::InvalidBufferSize(other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioConfig {
    pub sample_rate: SampleRate,
    pub buffer_size: BufferSize,
}

impl AudioConfig {
    pub fn new(sample_rate: SampleRate, buffer_size: BufferSize) -> Result<Self, InvalidAudioConfigError> {
        Ok(Self {
            sample_rate,
            buffer_size,
        })
    }

    pub fn from_raw(sample_rate: u32, buffer_size: usize) -> Result<Self, InvalidAudioConfigError> {
        let sr = SampleRate::try_from_u32(sample_rate)?;
        let bs = BufferSize::try_from_usize(buffer_size)?;
        Self::new(sr, bs)
    }

    /// Tempo de deadline do buffer em microssegundos: (buffer_size / sample_rate) * 1_000_000
    pub fn deadline_micros(&self) -> u64 {
        ((self.buffer_size.as_usize() as f64 / self.sample_rate.as_f64()) * 1_000_000.0) as u64
    }
}
