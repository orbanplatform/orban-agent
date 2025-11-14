// GPU 偵測與監控模組

mod detector;
mod device;

#[cfg(feature = "nvidia")]
mod nvidia;

#[cfg(feature = "amd")]
mod amd;

#[cfg(target_os = "macos")]
mod apple;

pub use detector::GPUDetector;
pub use device::{GPUDevice, DeviceType};

use crate::types::{GPUInfo, GPUStatus, MemoryInfo, HardwareInfo, TaskRequirements};
use crate::error::Result;
