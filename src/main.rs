use anyhow::Result;
use clap::Parser;
use env_logger;
use log::{error, info, warn};

mod compression;
mod cli;
mod utils;

#[cfg(feature = "gui")]
mod gui;

use cli::Cli;
use compression::hardware::HardwareCapabilities;
use compression::{CompressionEngine, CompressionSettings};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    // Handle hardware listing command
    if let Some(cli::Commands::ListHardware) = cli.command {
        return list_hardware_capabilities().await;
    }

    // Detect hardware capabilities
    let hw_capabilities = match HardwareCapabilities::detect().await {
        Ok(caps) => {
            info!("Hardware detection successful");
            info!("Available encoders: {:?}", caps.available_encoders);
            caps
        }
        Err(e) => {
            warn!("Hardware detection failed: {}", e);
            info!("Falling back to software encoding");
            HardwareCapabilities::software_only()
        }
    };

    // Execute compression based on CLI arguments
    match &cli.command {
        Some(cli::Commands::Compress { input, output, settings }) => {
            let compression_settings = CompressionSettings::from_cli_settings(settings, &hw_capabilities)?;
            let mut engine = CompressionEngine::new(hw_capabilities);
            
            info!("Starting compression: {} -> {:?}", input.display(), output);
            info!("Using encoder: {:?}", compression_settings.hardware_encoder);
            
            engine.compress(input, output.as_ref().map(|p| p.as_path()), &compression_settings, None).await?;
            
            info!("Compression completed successfully!");
        }
        #[cfg(feature = "gui")]
        Some(cli::Commands::Gui) => {
            info!("Launching GUI interface");
            return launch_gui(hw_capabilities).await;
        }
        #[cfg(not(feature = "gui"))]
        Some(cli::Commands::Gui) => {
            eprintln!("❌ GUI support not compiled in this build");
            eprintln!("Please use the CLI interface or recompile with --features gui");
            std::process::exit(1);
        }
        Some(cli::Commands::ListHardware) => {
            // Already handled above
        }
        None => {
            // Interactive mode or default behavior
            info!("Starting Small MP4 in interactive mode");
            #[cfg(feature = "gui")]
            {
                return launch_gui(hw_capabilities).await;
            }
            #[cfg(not(feature = "gui"))]
            {
                eprintln!("Interactive mode not yet implemented. Use --help for CLI options.");
            }
        }
    }

    Ok(())
}

#[cfg(feature = "gui")]
async fn launch_gui(hw_capabilities: HardwareCapabilities) -> Result<()> {
    use gui::SmallMp4App;
    
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([500.0, 600.0])
            .with_min_inner_size([400.0, 550.0])
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Small MP4 - Video Compressor",
        options,
        Box::new(move |cc| Ok(Box::new(SmallMp4App::new_with_context(cc, hw_capabilities)))),
    ) {
        eprintln!("Failed to run GUI: {}", e);
        return Err(anyhow::anyhow!("GUI failed to start: {}", e));
    }
    
    Ok(())
}

async fn list_hardware_capabilities() -> Result<()> {
    println!("🔍 Detecting hardware acceleration capabilities...\n");
    
    match HardwareCapabilities::detect().await {
        Ok(capabilities) => {
            println!("✅ Hardware Detection Results:\n");
            
            // CUDA devices
            if !capabilities.cuda_devices.is_empty() {
                println!("🔥 NVIDIA CUDA Devices:");
                for (i, device) in capabilities.cuda_devices.iter().enumerate() {
                    println!("  [{}] {} - {}MB VRAM (Compute {}.{})", 
                        i, device.name, device.memory_mb, 
                        device.compute_capability.0, device.compute_capability.1);
                    println!("      NVENC Support: {}", if device.nvenc_support { "✅" } else { "❌" });
                    println!("      Max Sessions: {}", device.max_concurrent_sessions);
                }
                println!();
            }
            
            // Available encoders
            println!("⚡ Available Hardware Encoders:");
            for encoder in &capabilities.available_encoders {
                let speed_boost = capabilities.speed_improvement(encoder);
                println!("  • {:?} - {:.1}x faster encoding", encoder, speed_boost);
            }
            println!();
            
            // Preferred encoder
            if let Some(preferred) = &capabilities.preferred_encoder {
                println!("🎯 Recommended Encoder: {:?}", preferred);
                println!("   Memory Usage: {}MB", capabilities.memory_usage_mb);
                println!("   Speed Multiplier: {:.1}x", capabilities.encoding_speed_multiplier);
            }
        },
        Err(e) => {
            eprintln!("❌ Hardware detection failed: {:?}", e);
            println!("💻 Falling back to software encoding only");
        }
    }
    
    Ok(())
}