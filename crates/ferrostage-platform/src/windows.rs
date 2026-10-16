//! Safe Windows adapters. Never persist HMONITOR or GDI enumeration indexes.
use std::collections::BTreeMap;

use winsafe::{
    DEVMODE, EnumDisplayDevices, EnumDisplaySettings, GmidxEnum, HDC, HMONITOR, HWND, SetLastError,
    co,
};

use crate::{MonitorInfo, PlatformError, ScaleSource, monitor::identity};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Device {
    connection: String,
    path: String,
    name: String,
    primary: bool,
}

struct Geometry {
    connection: String,
    size: (u32, u32),
    position: (i32, i32),
    scale: f32,
    rotation: u16,
    primary: bool,
}

pub(crate) fn enumerate() -> Result<Vec<MonitorInfo>, PlatformError> {
    let devices = devices()?;
    let geometry = geometry()?;
    // The providers are not an atomic OS snapshot. Reject a connection change
    // during the query; the watcher retries while projection stays unavailable.
    let after = self::devices()?;
    if devices != after {
        return Err(PlatformError::IncompleteTopology {
            devices: devices.into_keys().collect(),
            geometry: after.into_keys().collect(),
        });
    }
    combine(devices, geometry)
}

fn devices() -> Result<BTreeMap<String, Device>, PlatformError> {
    let mut devices = BTreeMap::new();
    // winsafe's iterator reuses one native structure. Copy every value before
    // advancing; never collect its references or return them from this module.
    SetLastError(co::ERROR::SUCCESS);
    for adapter in EnumDisplayDevices(None, None) {
        let adapter =
            adapter.map_err(|error| native_error("EnumDisplayDevices(adapter)", error))?;
        if !adapter
            .StateFlags
            .has(co::DISPLAY_DEVICE::ATTACHED_TO_DESKTOP)
            || adapter.StateFlags.has(co::DISPLAY_DEVICE::MIRRORING_DRIVER)
        {
            continue;
        }
        let connection = adapter.DeviceName();
        let primary = adapter.StateFlags.has(co::DISPLAY_DEVICE::PRIMARY_DEVICE);
        let mut active = Vec::new();
        SetLastError(co::ERROR::SUCCESS);
        for monitor in
            EnumDisplayDevices(Some(&connection), Some(co::EDD::GET_DEVICE_INTERFACE_NAME))
        {
            let monitor =
                monitor.map_err(|error| native_error("EnumDisplayDevices(monitor)", error))?;
            // DISPLAY_DEVICE_ACTIVE is the same bit as ATTACHED_TO_DESKTOP for
            // the monitor (rather than adapter) query.
            if monitor
                .StateFlags
                .has(co::DISPLAY_DEVICE::ATTACHED_TO_DESKTOP)
            {
                active.push((monitor.DeviceID(), monitor.DeviceString()));
            }
        }
        if active.len() != 1 || devices.contains_key(&connection) {
            return Err(PlatformError::AmbiguousConnection { connection });
        }
        let (path, name) = active.remove(0);
        devices.insert(
            connection.clone(),
            Device {
                connection,
                path,
                name,
                primary,
            },
        );
        // Nested calls may leave LastError set although adapter enumeration
        // completed successfully. winsafe consults LastError at end-of-list.
        SetLastError(co::ERROR::SUCCESS);
    }
    Ok(devices)
}

fn friendly_registry_value(path: &str) -> Option<String> {
    let subkey = crate::friendly::display_enum_subkey(path)?;
    let value = winsafe::HKEY::LOCAL_MACHINE
        .RegGetValue(
            Some(&subkey),
            Some("FriendlyName"),
            co::RRF::RT_REG_SZ | co::RRF::RT_REG_EXPAND_SZ,
        )
        .ok()?;
    match value {
        winsafe::RegistryValue::Sz(text) | winsafe::RegistryValue::ExpandSz(text) => Some(text),
        _ => None,
    }
}

fn native_error(operation: &'static str, error: co::ERROR) -> PlatformError {
    PlatformError::NativeEnumeration {
        operation,
        code: u32::from(error),
    }
}

fn geometry() -> Result<Vec<Geometry>, PlatformError> {
    // Keep a real desktop owner: winsafe 0.0.29 skips ReleaseDC for a NULL HWND.
    // This DC still enumerates all attached monitors with per-monitor callback
    // DCs; complete device/geometry matching below rejects partial results.
    // The guard releases on every exit and callback handles never escape.
    let desktop = HWND::GetDesktopWindow()
        .GetDC()
        .map_err(|e| native_error("GetDC(desktop)", e))?;
    let mut result = Vec::new();
    desktop
        .EnumDisplayMonitors(None, |monitor, dc, _| {
            result.push(read_geometry(monitor, dc));
            true
        })
        .map_err(|e| native_error("EnumDisplayMonitors", e))?;
    result.into_iter().collect()
}

fn read_geometry(monitor: HMONITOR, dc: HDC) -> Result<Geometry, PlatformError> {
    let info = monitor
        .GetMonitorInfo()
        .map_err(|e| native_error("GetMonitorInfo", e))?;
    let connection = info.szDevice();
    let mut mode = DEVMODE::default();
    let found = EnumDisplaySettings(
        Some(&connection),
        GmidxEnum::Enum(co::ENUM_SETTINGS::CURRENT),
        &mut mode,
    )
    .map_err(|e| native_error("EnumDisplaySettings", e))?;
    if !found {
        return Err(PlatformError::InvalidGeometry { connection });
    }
    geometry_from_mode(
        connection,
        &mode,
        dc.GetDeviceCaps(co::GDC::DESKTOPHORZRES),
        dc.GetDeviceCaps(co::GDC::HORZRES),
        info.dwFlags == co::MONITORINFOF::PRIMARY,
    )
}

fn geometry_from_mode(
    connection: String,
    mode: &DEVMODE,
    physical: i32,
    logical: i32,
    primary: bool,
) -> Result<Geometry, PlatformError> {
    let required =
        co::DM::POSITION | co::DM::DISPLAYORIENTATION | co::DM::PELSWIDTH | co::DM::PELSHEIGHT;
    // winsafe's `has` accepts any matching bit; every required field is needed.
    if mode.dmFields & required != required
        || physical <= 0
        || logical <= 0
        || u32::try_from(physical).ok() != Some(mode.dmPelsWidth)
    {
        return Err(PlatformError::InvalidGeometry { connection });
    }
    let rotation = match mode.dmDisplayOrientation() {
        co::DMDO::DEFAULT => 0,
        co::DMDO::D90 => 90,
        co::DMDO::D180 => 180,
        co::DMDO::D270 => 270,
        _ => return Err(PlatformError::InvalidGeometry { connection }),
    };
    Ok(Geometry {
        connection,
        size: (mode.dmPelsWidth, mode.dmPelsHeight),
        position: (mode.dmPosition().x, mode.dmPosition().y),
        scale: physical as f32 / logical as f32,
        rotation,
        primary,
    })
}

fn combine(
    devices: BTreeMap<String, Device>,
    geometry: Vec<Geometry>,
) -> Result<Vec<MonitorInfo>, PlatformError> {
    let mut infos = BTreeMap::new();
    for info in geometry {
        let connection = info.connection.clone();
        if infos.insert(connection.clone(), info).is_some() {
            return Err(PlatformError::AmbiguousConnection { connection });
        }
    }
    if !devices.keys().eq(infos.keys()) {
        return Err(PlatformError::IncompleteTopology {
            devices: devices.into_keys().collect(),
            geometry: infos.into_keys().collect(),
        });
    }
    let mut result = Vec::with_capacity(devices.len());
    for (connection, device) in devices {
        let info = &infos[&connection];
        if info.size.0 == 0
            || info.size.1 == 0
            || !info.scale.is_finite()
            || info.scale <= 0.0
            || ![0, 90, 180, 270].contains(&info.rotation)
            || device.primary != info.primary
        {
            return Err(PlatformError::InvalidGeometry { connection });
        }
        let driver_name = device.name;
        let friendly = friendly_registry_value(&device.path);
        let resolved = crate::friendly::human_name(&driver_name, friendly.as_deref());
        let (id, identity_source, identity_confidence) =
            identity(&device.path, &driver_name, info.size);
        result.push(MonitorInfo {
            id,
            name: resolved.name,
            connection_name: connection,
            native_size: info.size,
            position: info.position,
            scale_factor: info.scale,
            scale_source: ScaleSource::GdiDeviceCapsRatio,
            rotation_degrees: info.rotation,
            is_primary: info.primary,
            identity_source,
            identity_confidence,
            built_in: resolved.built_in,
        });
    }
    result.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(result)
}

#[cfg(test)]
mod tests;
