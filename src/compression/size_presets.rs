use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TargetSize {
    Size1MB = 1,
    Size5MB = 5,
    Size10MB = 10,
    Size30MB = 30,
    Size50MB = 50,
    Size100MB = 100,
    Size250MB = 250,
    Size500MB = 500,
    Size1000MB = 1000,
}

impl TargetSize {
    pub const ALL: [TargetSize; 9] = [
        TargetSize::Size1MB,
        TargetSize::Size5MB,
        TargetSize::Size10MB,
        TargetSize::Size30MB,
        TargetSize::Size50MB,
        TargetSize::Size100MB,
        TargetSize::Size250MB,
        TargetSize::Size500MB,
        TargetSize::Size1000MB,
    ];
    
    pub fn as_mb(&self) -> f32 {
        match self {
            TargetSize::Size1MB => 1.0,
            TargetSize::Size5MB => 5.0,
            TargetSize::Size10MB => 10.0,
            TargetSize::Size30MB => 30.0,
            TargetSize::Size50MB => 50.0,
            TargetSize::Size100MB => 100.0,
            TargetSize::Size250MB => 250.0,
            TargetSize::Size500MB => 500.0,
            TargetSize::Size1000MB => 1000.0,
        }
    }
    
    pub fn from_mb(mb: f32) -> Self {
        // Find the closest preset
        let mb = mb as u32;
        match mb {
            0..=3 => TargetSize::Size1MB,
            4..=7 => TargetSize::Size5MB,
            8..=20 => TargetSize::Size10MB,
            21..=40 => TargetSize::Size30MB,
            41..=75 => TargetSize::Size50MB,
            76..=175 => TargetSize::Size100MB,
            176..=375 => TargetSize::Size250MB,
            376..=750 => TargetSize::Size500MB,
            _ => TargetSize::Size1000MB,
        }
    }
    
    pub fn from_index(index: usize) -> Option<TargetSize> {
        Self::ALL.get(index).copied()
    }
    
    pub fn to_index(&self) -> usize {
        Self::ALL.iter().position(|&s| s == *self).unwrap_or(2)
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetSize::Size1MB => "1 MB - Ultra Small",
            TargetSize::Size5MB => "5 MB - Small", 
            TargetSize::Size10MB => "10 MB - Compact",
            TargetSize::Size30MB => "30 MB - Medium",
            TargetSize::Size50MB => "50 MB - Standard",
            TargetSize::Size100MB => "100 MB - Large",
            TargetSize::Size250MB => "250 MB - Extra Large",
            TargetSize::Size500MB => "500 MB - HD Quality",
            TargetSize::Size1000MB => "1 GB - Full Quality",
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
            TargetSize::Size100MB => "HD streaming",
            TargetSize::Size250MB => "High quality sharing",
            TargetSize::Size500MB => "Professional use",
            TargetSize::Size1000MB => "Archive quality",
        }
    }
    
}