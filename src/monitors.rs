use std::mem;
use std::ptr;

use winapi::{
    shared::basetsd::UINT32,
    um::{
        physicalmonitorenumerationapi::{
            GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR,
        },
        wingdi::{
            DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_DEVICE_INFO_HEADER,
            DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_MODE_INFO_TYPE_TARGET, DISPLAYCONFIG_PATH_INFO,
            DISPLAYCONFIG_TARGET_DEVICE_NAME, DISPLAYCONFIG_TOPOLOGY_ID, DISPLAY_DEVICEW,
            QDC_ALL_PATHS,
        },
        winnt::{HANDLE, LONG},
        winuser::{EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW},
    },
};

pub struct Monitor {
    id: i32,
    name: String,
    _handle: HANDLE,
}

impl Monitor {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub fn get_monitors() -> Vec<Monitor> {
    let display_devices = get_display_devices();
    let display_monitors = get_display_monitors();

    let mut monitors = Vec::new();

    for (id, device) in (1..).zip(display_devices.into_iter()) {
        let monitor = display_monitors
            .iter()
            .find(|monitor| device.device_name.starts_with(&monitor.device_name))
            .unwrap();

        monitors.push(Monitor {
            id,
            name: device.friendly_name,
            _handle: monitor.handle,
        });
    }

    monitors
}

#[derive(Debug)]
struct DisplayDevice {
    friendly_name: String,
    device_name: String,
}

fn get_display_devices() -> Vec<DisplayDevice> {
    let device_map = get_device_map();

    let mut num_paths = 0;
    let mut num_modes = 0;

    unsafe {
        GetDisplayConfigBufferSizes(QDC_ALL_PATHS, &mut num_paths, &mut num_modes);
    }

    let mut paths = Vec::with_capacity(num_paths as usize);
    let mut modes = Vec::with_capacity(num_modes as usize);

    unsafe {
        QueryDisplayConfig(
            QDC_ALL_PATHS,
            &mut num_paths,
            paths.as_mut_ptr(),
            &mut num_modes,
            modes.as_mut_ptr(),
            ptr::null_mut(),
        );

        paths.set_len(paths.capacity());
        modes.set_len(modes.capacity());
    }

    let mut devices = vec![];

    for mode in modes {
        if mode.infoType == DISPLAYCONFIG_MODE_INFO_TYPE_TARGET {
            let mut name: DISPLAYCONFIG_TARGET_DEVICE_NAME = unsafe { mem::zeroed() };

            name.header = DISPLAYCONFIG_DEVICE_INFO_HEADER {
                _type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                size: mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
                adapterId: mode.adapterId,
                id: mode.id,
            };

            unsafe {
                DisplayConfigGetDeviceInfo(
                    (&mut name as *mut DISPLAYCONFIG_TARGET_DEVICE_NAME).cast(),
                );
            }

            let friendly_name = utf16_nt_to_string(&name.monitorFriendlyDeviceName);
            let device_path = name.monitorDevicePath;
            let (_, device_name) = device_map
                .iter()
                .find(|(id, _)| &id[..] == &device_path[..])
                .unwrap();

            devices.push(DisplayDevice {
                friendly_name,
                device_name: utf16_nt_to_string(device_name),
            });
        }
    }

    devices
}

fn get_device_map() -> Vec<([u16; 128], [u16; 32])> {
    let mut map = Vec::new();

    let mut device = unsafe {
        let mut device: DISPLAY_DEVICEW = mem::zeroed();
        device.cb = mem::size_of::<DISPLAY_DEVICEW>() as u32;
        device
    };

    let mut i = 0;
    unsafe {
        while EnumDisplayDevicesW(ptr::null(), i, &mut device, 0) != 0 {
            let name = device.DeviceName.to_owned();
            if EnumDisplayDevicesW(name.as_ptr(), 0, &mut device, 1) != 0 {
                map.push((device.DeviceID.to_owned(), device.DeviceName.to_owned()));
            };

            i += 1
        }
    }

    map
}

#[derive(Debug)]
struct DisplayMonitor {
    device_name: String,
    handle: HANDLE,
}

fn get_display_monitors() -> Vec<DisplayMonitor> {
    let mut monitors = Vec::<DisplayMonitor>::new();

    unsafe {
        EnumDisplayMonitors(
            ptr::null_mut(),
            ptr::null_mut(),
            Some(enum_monitor_proc),
            mem::transmute(&mut monitors),
        );
    }

    monitors
}

unsafe extern "system" fn enum_monitor_proc(
    hmonitor: winapi::shared::windef::HMONITOR,
    _hdc: winapi::shared::windef::HDC,
    _rect: *mut winapi::shared::windef::RECT,
    data: isize,
) -> i32 {
    let monitors = data as *mut Vec<DisplayMonitor>;

    let mut info: MONITORINFOEXW = mem::zeroed();
    info.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, (&mut info as *mut MONITORINFOEXW).cast()) == 0 {
        panic!("failed to get monitor info");
    }

    let mut count = 0;
    if GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor, &mut count) == 0 {
        panic!("failed to get number of physical monitors");
    }

    let mut physical_monitors = Vec::with_capacity(count as usize);
    if GetPhysicalMonitorsFromHMONITOR(hmonitor, count, physical_monitors.as_mut_ptr()) == 0 {
        panic!("failed to get physical monitors");
    }
    physical_monitors.set_len(physical_monitors.capacity());

    for monitor in physical_monitors {
        (*monitors).push(DisplayMonitor {
            device_name: utf16_nt_to_string(&info.szDevice),
            handle: monitor.hPhysicalMonitor,
        });
    }

    1
}

/// Converts a null terminated buffer of utf16 characters to a `String`.
fn utf16_nt_to_string(buf: &[u16]) -> String {
    let len = buf.iter().take_while(|&&i| i != 0).count();
    String::from_utf16_lossy(&buf[0..len])
}

extern "system" {
    pub fn GetDisplayConfigBufferSizes(
        flags: UINT32,
        numPathArrayElements: *mut UINT32,
        numModeInfoArrayElements: *mut UINT32,
    ) -> LONG;

    pub fn QueryDisplayConfig(
        flags: UINT32,
        numPathArrayElements: *mut UINT32,
        pathArray: *mut DISPLAYCONFIG_PATH_INFO,
        numModeInfoArrayElements: *mut UINT32,
        modeInfoArray: *mut DISPLAYCONFIG_MODE_INFO,
        currentTopologyId: *mut DISPLAYCONFIG_TOPOLOGY_ID,
    ) -> LONG;

    pub fn DisplayConfigGetDeviceInfo(requestPacket: *mut DISPLAYCONFIG_DEVICE_INFO_HEADER)
        -> LONG;
}
