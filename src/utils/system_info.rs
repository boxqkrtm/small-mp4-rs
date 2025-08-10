use sysinfo::{System, Disks};
use std::collections::HashMap;
use log::{debug, info};

#[derive(Debug, Clone)]
pub struct SystemInformation {
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub cpu_info: CpuInformation,
    pub memory_info: MemoryInformation,
    pub disk_info: Vec<DiskInformation>,
    pub process_info: ProcessInformation,
}

#[derive(Debug, Clone)]
pub struct CpuInformation {
    pub name: String,
    pub cores: usize,
    pub frequency_mhz: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone)]
pub struct MemoryInformation {
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone)]
pub struct DiskInformation {
    pub name: String,
    pub mount_point: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone)]
pub struct ProcessInformation {
    pub total_processes: usize,
    pub current_process_memory_mb: u64,
    pub current_process_cpu_percent: f32,
}

impl SystemInformation {
    pub fn collect() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        // Wait a bit and refresh again for more accurate CPU measurements
        std::thread::sleep(std::time::Duration::from_millis(200));
        system.refresh_cpu();
        
        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let architecture = std::env::consts::ARCH.to_string();
        
        let cpu_info = Self::collect_cpu_info(&system);
        let memory_info = Self::collect_memory_info(&system);
        let disk_info = Self::collect_disk_info(&system);
        let process_info = Self::collect_process_info(&system);
        
        debug!("System information collected: {} {}, {} CPU cores, {:.1}GB RAM", 
               os_name, os_version, cpu_info.cores, memory_info.total_gb);
        
        Self {
            os_name,
            os_version,
            architecture,
            cpu_info,
            memory_info,
            disk_info,
            process_info,
        }
    }
    
    fn collect_cpu_info(system: &System) -> CpuInformation {
        let cpus = system.cpus();
        let name = if !cpus.is_empty() {
            cpus[0].brand().to_string()
        } else {
            "Unknown CPU".to_string()
        };
        
        let cores = cpus.len();
        let frequency_mhz = if !cpus.is_empty() {
            cpus[0].frequency()
        } else {
            0
        };
        
        let usage_percent = system.global_cpu_info().cpu_usage();
        
        CpuInformation {
            name,
            cores,
            frequency_mhz,
            usage_percent,
        }
    }
    
    fn collect_memory_info(system: &System) -> MemoryInformation {
        let total_bytes = system.total_memory();
        let used_bytes = system.used_memory();
        let available_bytes = system.available_memory();
        
        let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let used_gb = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let available_gb = available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        
        let usage_percent = if total_bytes > 0 {
            (used_bytes as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };
        
        MemoryInformation {
            total_gb,
            used_gb,
            available_gb,
            usage_percent,
        }
    }
    
    fn collect_disk_info(_system: &System) -> Vec<DiskInformation> {
        let disks = Disks::new_with_refreshed_list();
        disks.iter().map(|disk| {
            let total_bytes = disk.total_space();
            let available_bytes = disk.available_space();
            let used_bytes = total_bytes - available_bytes;
            
            let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let available_gb = available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            
            let usage_percent = if total_bytes > 0 {
                (used_bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };
            
            DiskInformation {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_gb,
                used_gb,
                available_gb,
                usage_percent,
            }
        }).collect()
    }
    
    fn collect_process_info(system: &System) -> ProcessInformation {
        let total_processes = system.processes().len();
        
        let current_pid = std::process::id();
        let current_process = system.process(sysinfo::Pid::from(current_pid as usize));
        
        let (current_process_memory_mb, current_process_cpu_percent) = if let Some(process) = current_process {
            let memory_mb = process.memory() / (1024 * 1024);
            let cpu_percent = process.cpu_usage();
            (memory_mb, cpu_percent)
        } else {
            (0, 0.0)
        };
        
        ProcessInformation {
            total_processes,
            current_process_memory_mb,
            current_process_cpu_percent,
        }
    }
    
    /// Get a summary string of system information
    pub fn summary(&self) -> String {
        format!(
            "{} {} ({}) - {} cores, {:.1}GB RAM, {:.1}% CPU usage",
            self.os_name,
            self.os_version,
            self.architecture,
            self.cpu_info.cores,
            self.memory_info.total_gb,
            self.cpu_info.usage_percent
        )
    }
    
    /// Check if system has enough resources for video encoding
    pub fn check_encoding_readiness(&self) -> EncodingReadiness {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        
        // Check available memory
        if self.memory_info.available_gb < 1.0 {
            issues.push("Low available memory (< 1GB)".to_string());
            recommendations.push("Close other applications to free memory".to_string());
        } else if self.memory_info.available_gb < 2.0 {
            recommendations.push("Consider closing some applications for better performance".to_string());
        }
        
        // Check CPU usage
        if self.cpu_info.usage_percent > 90.0 {
            issues.push("High CPU usage".to_string());
            recommendations.push("Wait for CPU usage to decrease or close CPU-intensive applications".to_string());
        }
        
        // Check disk space on all drives
        let mut has_low_disk_space = false;
        for disk in &self.disk_info {
            if disk.available_gb < 1.0 {
                issues.push(format!("Low disk space on {} (< 1GB)", disk.mount_point));
                has_low_disk_space = true;
            }
        }
        
        if has_low_disk_space {
            recommendations.push("Free up disk space before encoding".to_string());
        }
        
        // Determine overall readiness
        let readiness = if issues.is_empty() {
            if recommendations.is_empty() {
                ReadinessLevel::Optimal
            } else {
                ReadinessLevel::Good
            }
        } else if issues.len() == 1 && self.memory_info.available_gb >= 0.5 {
            ReadinessLevel::Marginal
        } else {
            ReadinessLevel::Poor
        };
        
        EncodingReadiness {
            readiness,
            issues,
            recommendations,
        }
    }
    
    /// Get estimated encoding performance based on system specs
    pub fn estimate_encoding_performance(&self) -> EncodingPerformance {
        // Base performance on CPU cores and frequency
        let cpu_score = (self.cpu_info.cores as f32) * (self.cpu_info.frequency_mhz as f32 / 1000.0);
        
        // Memory score (more memory = better performance for large files)
        let memory_score = (self.memory_info.total_gb as f32).min(32.0) / 32.0;
        
        // Combined performance score
        let performance_score = (cpu_score * 0.7 + memory_score * 0.3).min(100.0);
        
        let estimated_speed = if performance_score > 50.0 {
            "Fast"
        } else if performance_score > 25.0 {
            "Medium"  
        } else {
            "Slow"
        };
        
        let concurrent_jobs = if self.cpu_info.cores >= 8 && self.memory_info.total_gb >= 16.0 {
            (self.cpu_info.cores / 4).max(1)
        } else if self.cpu_info.cores >= 4 && self.memory_info.total_gb >= 8.0 {
            2
        } else {
            1
        };
        
        EncodingPerformance {
            performance_score,
            estimated_speed: estimated_speed.to_string(),
            recommended_concurrent_jobs: concurrent_jobs,
            memory_per_job_gb: (self.memory_info.available_gb / concurrent_jobs as f64 * 0.8).max(1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EncodingReadiness {
    pub readiness: ReadinessLevel,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadinessLevel {
    Optimal,   // Perfect conditions for encoding
    Good,      // Good conditions with minor recommendations
    Marginal,  // Adequate but with some concerns
    Poor,      // Not recommended for encoding
}

#[derive(Debug, Clone)]
pub struct EncodingPerformance {
    pub performance_score: f32,          // 0-100 score
    pub estimated_speed: String,         // "Fast", "Medium", "Slow"
    pub recommended_concurrent_jobs: usize,
    pub memory_per_job_gb: f64,
}

impl EncodingReadiness {
    pub fn is_ready_for_encoding(&self) -> bool {
        matches!(self.readiness, ReadinessLevel::Optimal | ReadinessLevel::Good | ReadinessLevel::Marginal)
    }
    
    pub fn summary(&self) -> String {
        let status = match self.readiness {
            ReadinessLevel::Optimal => "✅ Optimal",
            ReadinessLevel::Good => "✅ Good",
            ReadinessLevel::Marginal => "⚠️ Marginal",
            ReadinessLevel::Poor => "❌ Poor",
        };
        
        format!("{} - {} issues, {} recommendations", 
                status, self.issues.len(), self.recommendations.len())
    }
}

/// Get system load information
pub fn get_system_load() -> SystemLoad {
    let mut system = System::new();
    system.refresh_cpu();
    system.refresh_memory();
    system.refresh_processes();
    
    let cpu_usage = system.global_cpu_info().cpu_usage();
    let memory_usage = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;
    
    let high_cpu_processes: Vec<String> = system
        .processes()
        .values()
        .filter(|p| p.cpu_usage() > 10.0)
        .map(|p| format!("{} ({:.1}%)", p.name(), p.cpu_usage()))
        .take(5)
        .collect();
    
    SystemLoad {
        cpu_usage_percent: cpu_usage,
        memory_usage_percent: memory_usage,
        high_cpu_processes,
        process_count: system.processes().len(),
    }
}

#[derive(Debug, Clone)]
pub struct SystemLoad {
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f64,
    pub high_cpu_processes: Vec<String>,
    pub process_count: usize,
}