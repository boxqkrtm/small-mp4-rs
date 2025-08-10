use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TargetSize {
    Size1MB = 1,
    Size5MB = 5,
    Size10MB = 10,
    Size30MB = 30,
    Size50MB = 50,
}

impl TargetSize {
    pub const ALL: [TargetSize; 5] = [
        TargetSize::Size1MB,
        TargetSize::Size5MB,
        TargetSize::Size10MB,
        TargetSize::Size30MB,
        TargetSize::Size50MB,
    ];
    
    pub fn as_mb(&self) -> f32 {
        *self as u8 as f32
    }
    
    pub fn from_mb(mb: f32) -> Self {
        // Find the closest preset
        let mb = mb as u8;
        match mb {
            0..=3 => TargetSize::Size1MB,
            4..=7 => TargetSize::Size5MB,
            8..=20 => TargetSize::Size10MB,
            21..=40 => TargetSize::Size30MB,
            _ => TargetSize::Size50MB,
        }
    }
    
    pub fn from_index(index: usize) -> Option<TargetSize> {
        Self::ALL.get(index).copied()
    }
    
    pub fn to_index(&self) -> usize {
        Self::ALL.iter().position(|&s| s == *self).unwrap_or(4)
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetSize::Size1MB => "1 MB - Ultra Small",
            TargetSize::Size5MB => "5 MB - Small", 
            TargetSize::Size10MB => "10 MB - Compact",
            TargetSize::Size30MB => "30 MB - Medium",
            TargetSize::Size50MB => "50 MB - Standard",
        }
    }
    
    /// Get the typical use case for this size preset
    pub fn use_case(&self) -> &'static str {
        match self {
            TargetSize::Size1MB => "Social media, messaging",
            TargetSize::Size5MB => "Email attachments", 
            TargetSize::Size10MB => "Quick sharing",
            TargetSize::Size30MB => "Presentations, demos",
            TargetSize::Size50MB => "General purpose",
        }
    }
    
}