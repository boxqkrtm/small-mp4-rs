use clap::ValueEnum;
use crate::compression::hardware::{HardwareEncoder, HardwarePreset, HardwareQuality};

#[derive(Clone, ValueEnum, Debug)]
pub enum HardwareEncoderCli {
    #[value(name = "nvenc-h264")]
    NvencH264,
    #[value(name = "nvenc-h265")]
    NvencH265,
    #[value(name = "nvenc-av1")]
    NvencAv1,
    #[value(name = "amf-h264")]
    AmfH264,
    #[value(name = "amf-h265")]
    AmfH265,
    #[value(name = "qsv-h264")]
    QsvH264,
    #[value(name = "qsv-h265")]
    QsvH265,
    #[value(name = "qsv-av1")]
    QsvAv1,
    #[value(name = "vaapi")]
    Vaapi,
    #[value(name = "videotoolbox")]
    VideoToolbox,
    #[value(name = "auto")]
    Auto,
    #[value(name = "software")]
    Software,
}

impl HardwareEncoderCli {
    pub fn to_hardware_encoder(&self) -> HardwareEncoder {
        match self {
            HardwareEncoderCli::NvencH264 => HardwareEncoder::NvencH264,
            HardwareEncoderCli::NvencH265 => HardwareEncoder::NvencH265,
            HardwareEncoderCli::NvencAv1 => HardwareEncoder::NvencAV1,
            HardwareEncoderCli::AmfH264 => HardwareEncoder::AmfH264,
            HardwareEncoderCli::AmfH265 => HardwareEncoder::AmfH265,
            HardwareEncoderCli::QsvH264 => HardwareEncoder::QsvH264,
            HardwareEncoderCli::QsvH265 => HardwareEncoder::QsvH265,
            HardwareEncoderCli::QsvAv1 => HardwareEncoder::QsvAV1,
            HardwareEncoderCli::Vaapi => HardwareEncoder::Vaapi,
            HardwareEncoderCli::VideoToolbox => HardwareEncoder::VideoToolbox,
            HardwareEncoderCli::Auto => HardwareEncoder::Software, // Will be resolved later
            HardwareEncoderCli::Software => HardwareEncoder::Software,
        }
    }
}

#[derive(Clone, ValueEnum, Debug)]
pub enum HardwarePresetCli {
    #[value(name = "ultrafast")]
    UltraFast,
    #[value(name = "faster")]
    Faster,
    #[value(name = "fast")]
    Fast,
    #[value(name = "medium")]
    Medium,
    #[value(name = "slow")]
    Slow,
    #[value(name = "slower")]
    Slower,
    #[value(name = "highest")]
    Highest,
}

impl HardwarePresetCli {
    pub fn to_hardware_preset(&self) -> HardwarePreset {
        match self {
            HardwarePresetCli::UltraFast => HardwarePreset::UltraFast,
            HardwarePresetCli::Faster => HardwarePreset::Faster,
            HardwarePresetCli::Fast => HardwarePreset::Fast,
            HardwarePresetCli::Medium => HardwarePreset::Medium,
            HardwarePresetCli::Slow => HardwarePreset::Slow,
            HardwarePresetCli::Slower => HardwarePreset::Slower,
            HardwarePresetCli::Highest => HardwarePreset::Highest,
        }
    }
}

#[derive(Clone, ValueEnum, Debug)]
pub enum HardwareQualityCli {
    #[value(name = "auto")]
    Auto,
    #[value(name = "constant")]
    Constant,
    #[value(name = "variable")]
    Variable,
    #[value(name = "constrained")]
    Constrained,
}

impl HardwareQualityCli {
    pub fn to_hardware_quality(&self) -> HardwareQuality {
        match self {
            HardwareQualityCli::Auto => HardwareQuality::Auto,
            HardwareQualityCli::Constant => HardwareQuality::Constant,
            HardwareQualityCli::Variable => HardwareQuality::Variable,
            HardwareQualityCli::Constrained => HardwareQuality::Constrained,
        }
    }
}
