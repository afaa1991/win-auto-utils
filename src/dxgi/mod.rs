//! Ultra-high-performance DXGI screen capture module
//!
//! Self-contained implementation using native Windows API.
//! Zero external dependencies beyond windows-rs.

#![cfg(windows)]

use std::sync::{Mutex, OnceLock};
use std::mem;

use windows::{
    core::Interface,
    Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_9_1},
    Win32::Graphics::Direct3D11::{
        D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
        D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11CreateDevice, ID3D11Device,
        ID3D11DeviceContext, ID3D11Texture2D,
    },
    Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, DXGI_ERROR_ACCESS_DENIED, DXGI_ERROR_ACCESS_LOST,
        DXGI_ERROR_NOT_FOUND, DXGI_ERROR_WAIT_TIMEOUT, DXGI_MAP_READ, DXGI_MAPPED_RECT,
        DXGI_OUTPUT_DESC, IDXGIAdapter1, IDXGIFactory1, IDXGIOutput, 
        IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource, IDXGISurface1,
    },
};

/// DXGI error types
#[derive(Debug)]
pub enum DxgiError {
    CaptureFailed(String),
    RegionOutOfBounds,
    InvalidRegionDimensions,
    InitializationFailed(String),
    LockFailed,
}

impl std::fmt::Display for DxgiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DxgiError::CaptureFailed(msg) => write!(f, "Capture failed: {}", msg),
            DxgiError::RegionOutOfBounds => write!(f, "Region out of bounds"),
            DxgiError::InvalidRegionDimensions => write!(f, "Invalid region dimensions"),
            DxgiError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            DxgiError::LockFailed => write!(f, "Lock acquisition failed"),
        }
    }
}

impl std::error::Error for DxgiError {}

fn map_error(e: windows::core::Error) -> DxgiError {
    let code = e.code();
    if code == DXGI_ERROR_ACCESS_LOST {
        DxgiError::CaptureFailed("Access lost".to_string())
    } else if code == DXGI_ERROR_WAIT_TIMEOUT {
        DxgiError::CaptureFailed("Timeout".to_string())
    } else if code == DXGI_ERROR_ACCESS_DENIED {
        DxgiError::CaptureFailed("Access denied".to_string())
    } else {
        DxgiError::CaptureFailed(e.to_string())
    }
}

struct DxgiManager {
    device: ID3D11Device,
    device_context: ID3D11DeviceContext,
    output_duplication: IDXGIOutputDuplication,
    width: usize,
    height: usize,
}

impl DxgiManager {
    fn new() -> Result<Self, DxgiError> {
        let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }
            .map_err(|e| DxgiError::InitializationFailed(format!("Create factory: {}", e)))?;

        let mut adapter_opt: Option<IDXGIAdapter1> = None;
        let mut output_opt: Option<IDXGIOutput1> = None;
        
        for i in 0.. {
            let adapter: IDXGIAdapter1 = match unsafe { factory.EnumAdapters1(i) } {
                Ok(a) => a,
                Err(e) if e.code() == DXGI_ERROR_NOT_FOUND => break,
                Err(_) => continue,
            };

            for j in 0.. {
                let output: IDXGIOutput = match unsafe { adapter.EnumOutputs(j) } {
                    Ok(o) => o,
                    Err(_) => break,
                };

                let desc: DXGI_OUTPUT_DESC = unsafe { output.GetDesc() }
                    .map_err(|e| DxgiError::InitializationFailed(format!("Get desc: {}", e)))?;

                if desc.AttachedToDesktop.as_bool() {
                    adapter_opt = Some(adapter);
                    output_opt = Some(output.cast().map_err(|e| {
                        DxgiError::InitializationFailed(format!("Cast output: {}", e))
                    })?);
                    break;
                }
            }

            if output_opt.is_some() {
                break;
            }
        }

        let adapter = adapter_opt.ok_or_else(|| {
            DxgiError::InitializationFailed("No suitable adapter found".to_string())
        })?;

        let output = output_opt.ok_or_else(|| {
            DxgiError::InitializationFailed("No attached output found".to_string())
        })?;

        let desc: DXGI_OUTPUT_DESC = unsafe { output.GetDesc() }
            .map_err(|e| DxgiError::InitializationFailed(format!("Get desc: {}", e)))?;
        let width = (desc.DesktopCoordinates.right - desc.DesktopCoordinates.left) as usize;
        let height = (desc.DesktopCoordinates.bottom - desc.DesktopCoordinates.top) as usize;

        let feature_levels: [D3D_FEATURE_LEVEL; 1] = [D3D_FEATURE_LEVEL_9_1];
        let (device, device_context) = unsafe {
            let mut dev: Option<ID3D11Device> = None;
            let mut ctx: Option<ID3D11DeviceContext> = None;
            
            D3D11CreateDevice(
                Some(&adapter.cast().unwrap()),
                D3D_DRIVER_TYPE_UNKNOWN,
                Default::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&feature_levels),
                D3D11_SDK_VERSION,
                Some(&mut dev),
                None,
                Some(&mut ctx),
            ).map_err(|e| DxgiError::InitializationFailed(format!("Create device: {}", e)))?;
            
            (dev.unwrap(), ctx.unwrap())
        };

        let output_duplication = unsafe { output.DuplicateOutput(&device) }
            .map_err(|e| DxgiError::InitializationFailed(format!("Duplicate output: {}", e)))?;

        Ok(Self { device, device_context, output_duplication, width, height })
    }

    fn capture_region(&mut self, x: i32, y: i32, w: i32, h: i32) -> Result<Vec<u8>, DxgiError> {
        let mut frame_info = unsafe { mem::zeroed() };
        let mut resource: Option<IDXGIResource> = None;
        
        unsafe {
            self.output_duplication
                .AcquireNextFrame(100, &mut frame_info, &mut resource)
                .map_err(map_error)?;
        }

        let resource = resource.ok_or_else(|| DxgiError::CaptureFailed("No resource".to_string()))?;
        let texture: ID3D11Texture2D = resource.cast()
            .map_err(|e| DxgiError::CaptureFailed(format!("Cast texture: {}", e)))?;

        let mut desc = D3D11_TEXTURE2D_DESC::default();
        unsafe { texture.GetDesc(&mut desc) };
        desc.Usage = D3D11_USAGE_STAGING;
        desc.BindFlags = 0;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
        desc.MiscFlags = 0;

        let mut staged: Option<ID3D11Texture2D> = None;
        unsafe {
            self.device.CreateTexture2D(&desc, None, Some(&mut staged))
                .map_err(|e| DxgiError::CaptureFailed(format!("Create staging: {}", e)))?;
        }
        let staged = staged.unwrap();
        unsafe { self.device_context.CopyResource(&staged, &texture) };

        unsafe { 
            self.output_duplication.ReleaseFrame()
                .map_err(|e| DxgiError::CaptureFailed(format!("Release: {}", e)))?;
        };

        let surface: IDXGISurface1 = staged.cast()
            .map_err(|e| DxgiError::CaptureFailed(format!("Cast surface: {}", e)))?;

        let mut rect = DXGI_MAPPED_RECT::default();
        unsafe { 
            surface.Map(&mut rect, DXGI_MAP_READ)
                .map_err(|e| DxgiError::CaptureFailed(format!("Map: {}", e)))?;
        };

        let pitch = rect.Pitch as usize;
        let source = rect.pBits;
        let row_bytes = (w * 4) as usize;
        let total_bytes = row_bytes * h as usize;
        let mut data = vec![0u8; total_bytes];

        unsafe {
            for row in 0..h as usize {
                let src_offset = ((y as usize + row) * pitch) + (x as usize * 4);
                let dst_offset = row * row_bytes;
                std::ptr::copy_nonoverlapping(
                    source.add(src_offset),
                    data.as_mut_ptr().add(dst_offset),
                    row_bytes,
                );
            }
        }

        unsafe { 
            surface.Unmap().map_err(|e| DxgiError::CaptureFailed(format!("Unmap: {}", e)))?;
        };

        Ok(data)
    }

    fn geometry(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

static MANAGER: OnceLock<Mutex<Option<DxgiManager>>> = OnceLock::new();

fn get_manager() -> Result<&'static Mutex<Option<DxgiManager>>, DxgiError> {
    Ok(MANAGER.get_or_init(|| {
        match DxgiManager::new() {
            Ok(m) => Mutex::new(Some(m)),
            Err(e) => {
                eprintln!("DXGI init failed: {}", e);
                Mutex::new(None)
            }
        }
    }))
}

pub fn capture_region_bytes(x: i32, y: i32, width: i32, height: i32) -> Result<Vec<u8>, DxgiError> {
    if width <= 0 || height <= 0 {
        return Err(DxgiError::InvalidRegionDimensions);
    }

    let manager = get_manager()?;
    let mut guard = manager.lock().map_err(|_| DxgiError::LockFailed)?;
    let mgr = guard.as_mut()
        .ok_or_else(|| DxgiError::InitializationFailed("Not initialized".to_string()))?;

    let (sw, sh) = mgr.geometry();
    if x < 0 || y < 0 || x + width > sw as i32 || y + height > sh as i32 {
        return Err(DxgiError::RegionOutOfBounds);
    }

    mgr.capture_region(x, y, width, height)
}

pub fn capture_full_screen() -> Result<(Vec<u8>, (usize, usize)), DxgiError> {
    let manager = get_manager()?;
    let mut guard = manager.lock().map_err(|_| DxgiError::LockFailed)?;
    let mgr = guard.as_mut()
        .ok_or_else(|| DxgiError::InitializationFailed("Not initialized".to_string()))?;

    let (width, height) = mgr.geometry();
    let data = mgr.capture_region(0, 0, width as i32, height as i32)?;
    
    Ok((data, (width, height)))
}

/// Ultra-fast region capture
pub fn get_screen_size() -> Result<(usize, usize), DxgiError> {
    let manager = get_manager()?;
    let guard = manager.lock().map_err(|_| DxgiError::LockFailed)?;
    let mgr = guard.as_ref()
        .ok_or_else(|| DxgiError::InitializationFailed("Not initialized".to_string()))?;
    Ok(mgr.geometry())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert!(DxgiError::CaptureFailed("test".to_string()).to_string().contains("test"));
        assert!(DxgiError::RegionOutOfBounds.to_string().contains("bounds"));
    }

    #[test]
    fn test_invalid_dimensions() {
        assert!(capture_region_bytes(0, 0, 0, 100).is_err());
        assert!(capture_region_bytes(0, 0, 100, 0).is_err());
    }
}
