use std::{fmt, ffi::{c_void, CString, NulError}, error::Error};
use bitflags::bitflags;

#[derive(Debug)]
pub enum FTD2XXError {
	InvalidHandle,
	DeviceNotFound,
	IOError,
	InsufficientResources,
	InvalidParameter,
	InvalidBaudRate,
	DeviceNotOpenedForErase,
	DeviceNotOpenedForWrite,
	FailedToWriteDevice,
	EEPROMReadFailed,
	EEPROMWriteFailed,
	EEPROMEraseFailed,
	EEPROMNotPresent,
	EEPROMNotProgrammed,
	InvalidArgs,
	NotSupported,
	OtherError(i32),
}
impl fmt::Display for FTD2XXError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      FTD2XXError::InvalidHandle => write!(f, "Invalid Handle"),
      FTD2XXError::DeviceNotFound => write!(f, "Device Not Found"),
      FTD2XXError::IOError => write!(f, "IO Error"),
      FTD2XXError::InsufficientResources => write!(f, "Insufficient Resources"),
      FTD2XXError::InvalidParameter => write!(f, "Invalid Parameter"),
      FTD2XXError::InvalidBaudRate => write!(f, "Invalid BaudRate"),
      FTD2XXError::DeviceNotOpenedForErase => write!(f, "Device Not Opened For Erase"),
      FTD2XXError::DeviceNotOpenedForWrite => write!(f, "Device Not Opened For Write"),
      FTD2XXError::FailedToWriteDevice => write!(f, "Failed To Write Device"),
      FTD2XXError::EEPROMReadFailed => write!(f, "EEPROM Read Failed"),
      FTD2XXError::EEPROMWriteFailed => write!(f, "EEPROM Write Failed"),
      FTD2XXError::EEPROMEraseFailed => write!(f, "EEPROM Erase Failed"),
      FTD2XXError::EEPROMNotPresent => write!(f, "EEPROM Not Present"),
      FTD2XXError::EEPROMNotProgrammed => write!(f, "EEPROM Not Programmed"),
      FTD2XXError::InvalidArgs => write!(f, "Invalid Args"),
      FTD2XXError::NotSupported => write!(f, "Not Supported"),
      FTD2XXError::OtherError(i) => write!(f, "Other Error {}", i),
    }
  }
}
fn get_fterror(i: i32) -> FTD2XXError {
  match i {
    1 => FTD2XXError::InvalidHandle,
    2 => FTD2XXError::DeviceNotFound,
    3 => FTD2XXError::IOError,
    4 => FTD2XXError::InsufficientResources,
    5 => FTD2XXError::InvalidParameter,
    6 => FTD2XXError::InvalidBaudRate,
    7 => FTD2XXError::DeviceNotOpenedForErase,
    8 => FTD2XXError::DeviceNotOpenedForWrite,
    9 => FTD2XXError::FailedToWriteDevice,
    10 => FTD2XXError::EEPROMReadFailed,
    11 => FTD2XXError::EEPROMWriteFailed,
    12 => FTD2XXError::EEPROMEraseFailed,
    13 => FTD2XXError::EEPROMNotPresent,
    14 => FTD2XXError::EEPROMNotProgrammed,
    15 => FTD2XXError::InvalidArgs,
    16 => FTD2XXError::NotSupported,
    _ => FTD2XXError::OtherError(i),
  }
}
impl Error for FTD2XXError {}
impl From<i32> for FTD2XXError {
  fn from(e: i32) -> Self {
    get_fterror(e)
  }
}

#[derive(Debug)]
pub enum FTError{
  FTD2XXError(FTD2XXError),
  InvalidParameter(NulError),
  DeviceClosed,
}
impl fmt::Display for FTError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      FTError::FTD2XXError(e) => write!(f, "FTD2XX Error: {}", e),
      FTError::InvalidParameter(e) => write!(f, "InvalidParameter: {}", e),
      FTError::DeviceClosed => write!(f, "Device is closed"),
    }
  }
}
impl Error for FTError {
  // Implement this to return the lower level source of this Error.
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      FTError::FTD2XXError(e) => Some(e),
      FTError::InvalidParameter(e) => Some(e),
      FTError::DeviceClosed => None,
    }
  }
}
impl From<FTD2XXError> for FTError {
  fn from(e: FTD2XXError) -> Self {
    FTError::FTD2XXError(e)
  }
}
impl From<NulError> for FTError {
  fn from(e: NulError) -> Self {
    FTError::InvalidParameter(e)
  }
}

bitflags! {
  pub struct FtDeviceInfoFlags: u32 {
    const OPENED = 0b00000001;
    const HISPEED = 0b00000010;
  }
}

bitflags! {
  pub struct PurgeFlags: u32 {
    const RX = 1;
    const TX = 2;
  }
}

#[repr(C)]
struct FtDeviceListInfoNode {
	pub flags: u32,
  pub device_type: u32,
	pub id: u32,
	pub location_id: u32,
  pub serial_number: [u8; 16],
  pub description: [u8; 64],
	pub handle: *mut core::ffi::c_void, // won't return
}

#[derive(Debug)]
pub struct DeviceListInfoNode {
	pub flags: FtDeviceInfoFlags,
  pub device_type: u32,
	pub id: u32,
	pub location_id: u32,
  pub serial_number: String,
  pub description: String,
}

const FT_OPEN_BY_SERIAL_NUMBER:u32 =	1;
const FT_OPEN_BY_DESCRIPTION:u32 =		2;
const FT_OPEN_BY_LOCATION:u32 =			  4;

#[derive(Debug)]
pub enum WordLength {
  Bits8 = 8,
  Bits7 = 7,
}
#[derive(Debug)]
pub enum StopBits {
  Bits2 = 2,
  Bits1 = 0,
}
#[derive(Debug)]
pub enum Parity {
  None = 0,
  Odd = 1,
  Even = 2,
  Mark = 3,
  Space = 4,
}

#[derive(Debug, Copy, Clone)]
pub enum FlowControl {
  None,
  RtsCts,
  DtrDsr,
  XonXoff((u8,u8))
}
impl From<FlowControl> for u16 {
  fn from(c: FlowControl) -> Self {
    match c {
      FlowControl::None => 0x0000,
      FlowControl::RtsCts => 0x0100,
      FlowControl::DtrDsr => 0x0200,
      FlowControl::XonXoff(_) => 0x0300,
    }
  }
}

#[link(name = "FTD2XX")]
extern{  
  fn FT_CreateDeviceInfoList(
    lpdwNumDevs: *mut i32
  ) -> i32;

  fn FT_GetDeviceInfoList(
		list: *mut FtDeviceListInfoNode,
    lpdwNumDevs: *mut i32
	) -> i32;

  fn FT_Close(
    pHandle: *mut core::ffi::c_void
  ) -> i32;

  fn FT_ResetDevice(
    pHandle: *mut core::ffi::c_void
  ) -> i32;

  fn FT_SetDataCharacteristics(
    pHandle: *mut core::ffi::c_void,
    uWordLength: u8,
    uStopBits: u8,
    uParity: u8
  ) -> i32;

  fn FT_SetBaudRate(
    pHandle: *mut core::ffi::c_void,
    dwBaudRate: u32,
  ) -> i32;

  fn FT_SetLatencyTimer(
    pHandle: *mut core::ffi::c_void,
    ucTimer: u8,
  ) -> i32;

  fn FT_SetFlowControl(
    pHandle: *mut core::ffi::c_void,
    usFlowControl: u16,
    uXon: u8,
    uXoff: u8
  ) -> i32;

  fn FT_ClrRts(pHandle: *mut core::ffi::c_void) -> i32;
  fn FT_SetRts(pHandle: *mut core::ffi::c_void) -> i32;
  fn FT_ClrDtr(pHandle: *mut core::ffi::c_void) -> i32;
  fn FT_SetDtr(pHandle: *mut core::ffi::c_void) -> i32;
  fn FT_SetBreakOn(pHandle: *mut core::ffi::c_void) -> i32;
  fn FT_SetBreakOff(pHandle: *mut core::ffi::c_void) -> i32;

  fn FT_Write(
    pHandle: *mut core::ffi::c_void,
		lpBuffer: *const u8,
		dwBytesToWrite: u32,
		lpBytesWritten: *mut u32
  ) -> i32;

  fn FT_Purge(
    pHandle: *mut core::ffi::c_void,
    dwMask: u32,
  ) -> i32;

  fn FT_Open(
    deviceNumber: i32,
    pHandle: *mut *mut core::ffi::c_void
  ) -> i32;

  fn FT_OpenEx(
    argument: *mut core::ffi::c_void,
    flags: u32,
    pHandle: *mut *mut core::ffi::c_void
  ) -> i32;
}

pub struct Device {
  handle: *mut core::ffi::c_void
}

impl Device {
  pub fn close(&self) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }
    let e: i32;
    unsafe{
      e = FT_Close(self.handle);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(())
  }
  pub fn write(&self, data: &[u8]) -> Result<usize, FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }

    let e: i32;
    let len = data.len();
    let mut written: u32 = 0;
    unsafe{
      e = FT_Write(self.handle, data.as_ptr(), len as u32, &mut written);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }

    Ok(written as usize)
  }
  pub fn set_data_characteristics(&self, wl: WordLength, sb: StopBits, p: Parity) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }

    let e: i32;
    unsafe{
      e = FT_SetDataCharacteristics(self.handle, wl as u8, sb as u8, p as u8);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }

    Ok(())
  }
  pub fn set_flow_control(&self, flow_control: FlowControl) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }

    let e: i32;
    let c_fc = u16::from(flow_control);
    let (xon,xoff) = match flow_control {
      FlowControl::XonXoff(p)=> p,
      _ => (0,0),
    };
    unsafe{
      e = FT_SetFlowControl(self.handle, c_fc, xon, xoff);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }

    Ok(())
  }
  pub fn set_baud_rate(&self, baud_rate: u32) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }

    let e: i32;
    unsafe{
      e = FT_SetBaudRate(self.handle, baud_rate);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }

    Ok(())
  }
  pub fn set_latency_timer(&self, timer: u8) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }

    let e: i32;
    unsafe{
      e = FT_SetLatencyTimer(self.handle, timer);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }

    Ok(())
  }
  pub fn purge(&self, flags: PurgeFlags) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }

    let e: i32;
    unsafe{
      e = FT_Purge(self.handle, flags.bits());
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }

    Ok(())
  }
  pub fn reset(&self) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }
    let e: i32;
    unsafe{
      e = FT_ResetDevice(self.handle);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(())
  }
  pub fn set_break_on(&self) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }
    let e: i32;
    unsafe{
      e = FT_SetBreakOn(self.handle);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(())
  }
  pub fn set_break_off(&self) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }
    let e: i32;
    unsafe{
      e = FT_SetBreakOff(self.handle);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(())
  }
  pub fn clear_rts(&self) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }
    let e: i32;
    unsafe{
      e = FT_ClrRts(self.handle);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(())
  }
  pub fn clear_dtr(&self) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }
    let e: i32;
    unsafe{
      e = FT_ClrDtr(self.handle);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(())
  }
  pub fn set_rts(&self) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }
    let e: i32;
    unsafe{
      e = FT_SetRts(self.handle);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(())
  }
  pub fn set_dtr(&self) -> Result<(), FTError> {
    if self.handle.is_null() { return Err(FTError::DeviceClosed) }
    let e: i32;
    unsafe{
      e = FT_SetDtr(self.handle);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(())
  }
  pub fn open(index: i32) -> Result<Device, FTError> {
    let mut h: *mut core::ffi::c_void = 0 as *mut core::ffi::c_void;
    let mut e: i32 = 0;
    unsafe{
      e = FT_Open(index, &mut h);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(Device { handle: h })
  }
  pub fn open_by_serial(serial: &str) -> Result<Device, FTError> {
    let mut h: *mut core::ffi::c_void = 0 as *mut core::ffi::c_void;
    let mut e: i32 = 0;
    let serial_c = CString::new(serial)?;
    unsafe{
      e = FT_OpenEx(serial_c.as_ptr() as *mut core::ffi::c_void, FT_OPEN_BY_SERIAL_NUMBER, &mut h);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(Device { handle: h })
  }
  pub fn open_by_description(description: &str) -> Result<Device, FTError> {
    let mut h: *mut core::ffi::c_void = 0 as *mut core::ffi::c_void;
    let mut e: i32 = 0;
    let descr_c = CString::new(description)?;
    unsafe{
      e = FT_OpenEx(descr_c.as_ptr() as *mut core::ffi::c_void, FT_OPEN_BY_DESCRIPTION, &mut h);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(Device { handle: h })
  }
  pub fn open_by_location(location_index: i32) -> Result<Device, FTError> {
    let mut h: *mut core::ffi::c_void = 0 as *mut core::ffi::c_void;
    let mut e: i32 = 0;
    unsafe{
      e = FT_OpenEx(location_index as *mut core::ffi::c_void, FT_OPEN_BY_LOCATION, &mut h);
    }
    if e != 0 {
      return Err(FTError::FTD2XXError(get_fterror(e)));
    }
    Ok(Device { handle: h })
  }
}

pub fn device_info_list() -> Result<Vec<DeviceListInfoNode>, FTError> {
  let mut capacity: i32 = 0;
  let mut ucap: usize;
  let mut e: i32;
  unsafe{
    e = FT_CreateDeviceInfoList(&mut capacity);
  }
  if e != 0 {
    return Err(FTError::FTD2XXError(get_fterror(e)));
  }

  ucap = capacity as usize;
  let mut inner: Vec<FtDeviceListInfoNode> = Vec::with_capacity(ucap);
  for _ in 0..ucap {
    inner.push(FtDeviceListInfoNode {
      flags: 0,
      device_type: 0,
      id: 0,
      location_id: 0,
      serial_number: [0;16],
      description: [0;64],
      handle: 0 as *mut c_void
    });
  }
  unsafe{
    e = FT_GetDeviceInfoList(inner.as_mut_ptr(), &mut capacity);
  }
  if e != 0 {
    return Err(FTError::FTD2XXError(get_fterror(e)));
  }
  ucap = capacity as usize;
  if ucap > inner.len() {
    log::warn!("Capacity increased while retrieving device info, new one: {}", ucap);
    ucap = inner.len();
  }
  let mut ret: Vec::<DeviceListInfoNode> = Vec::with_capacity(ucap);
  for i in 0..ucap {
    let a = &inner[i];
    let serial_len = a.serial_number.iter().position(|&x| x==0).unwrap_or(16); 
    let desc_len = a.description.iter().position(|&x| x==0).unwrap_or(64); 
    ret.push(DeviceListInfoNode {
      flags: FtDeviceInfoFlags{bits: a.flags},
      device_type: a.device_type,
      id: a.id,
      location_id: a.location_id,
      serial_number: String::from_utf8_lossy(&a.serial_number[..serial_len]).to_string(),
      description: String::from_utf8_lossy(&a.description[..desc_len]).to_string()
    });
  }
  Ok(ret)
}
