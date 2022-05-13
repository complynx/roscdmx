use std::{ops::Add, num::ParseIntError, process::exit, env::args};

use ftd2xx::Device;
use windows::Win32::Media::timeBeginPeriod;

mod ftd2xx;
mod timer;

const DMX_SIZE: usize = 512;
const DMX_BAUDRATE: u32 = 250000;
const DMX_BREAK_TIME: u64 = 92;
const DMX_MAB_TIME: u64 = 12;
const DMX_IDLE_TIME: u64 = 5000;

struct DMX<'a> {
  device: &'a ftd2xx::Device,
  timer: timer::Timer,
  pub break_time: std::time::Duration,
  pub mab_time: std::time::Duration,
  pub idle_time: std::time::Duration,
  next: std::time::Instant,
  pub data: Vec<u8>,
}
impl DMX<'_> {
  fn new<'a>(device: &'a ftd2xx::Device, size: usize) -> DMX {
    return DMX{
        device: &device,
        timer: timer::Timer::new(),
        break_time: std::time::Duration::from_micros(DMX_BREAK_TIME),
        mab_time: std::time::Duration::from_micros(DMX_MAB_TIME),
        idle_time: std::time::Duration::from_micros(DMX_IDLE_TIME),
        data: vec![0; size+1],
        next: std::time::Instant::now(),
    };
  }
  fn send_data(&mut self) -> Result<(), ftd2xx::FTError> {
    self.device.set_break_on()?;
    self.timer.sleep_for(self.break_time);
    self.device.set_break_off()?;
    self.timer.sleep_for(self.mab_time);
    self.device.write(&self.data)?;
    Ok(())
  }
  fn wait_and_send(&mut self) -> Result<(), ftd2xx::FTError> {
    self.timer.sleep(self.next);
    let ret = self.send_data();
    self.next = std::ops::Add::add(std::time::Instant::now(), self.idle_time);
    return ret;
  }
}

fn get_shift(starter: &str, addr:&str) -> Result<usize, ParseIntError> {
  return addr[starter.len()..].parse::<usize>()
}

fn main() {
  let mut dmx_size = DMX_SIZE;
    let dmx_size_help = format!("DMX size (1-512) default {}", dmx_size);
  let mut dmx_baudrate = DMX_BAUDRATE;
    let dmx_baudrate_help = format!("DMX baudrate, default {}", dmx_baudrate);
  let mut dmx_break_time: u64 = DMX_BREAK_TIME;
    let dmx_break_time_help = format!("DMX break time in microseconds, default {}us", dmx_break_time);
  let mut dmx_mab_time: u64 = DMX_MAB_TIME;
    let dmx_mab_time_help = format!("DMX MAB time in microseconds, default {}us", dmx_mab_time);
  let mut dmx_idle_time: u64 = DMX_IDLE_TIME;
    let dmx_idle_time_help = format!("DMX idle time in microseconds, default {}us", dmx_idle_time);

  let mut addr = "0.0.0.0".to_string();
    let addr_help = format!("sets listen address for OSC, default {}", addr);
  let mut port = 7701;
    let port_help = format!("sets port for OSC, default {}", port);
  let mut universe = 0;
    let universe_help = format!("sets universe for OSC, default {}", universe);

  let mut device_index = 0;
    let device_index_help = format!("select FTD2XX device by index, default {}", device_index);
  let mut device_serial = "".to_string();
  let mut device_description = "".to_string();
  let mut device_location_index = -1;

  let mut list_devices= false;

  {
    let mut ap = argparse::ArgumentParser::new();
    ap.set_description(r#"OSC driver for DMX USB based on FTD2XX chip.
Listens to OSC DMX and streams it to DMX.
  - OSC messages will be:
    * Address: "/<universe_number>/dmx/<dmx_address>"
    * Data: <list of integers> -> part of updated DMX data, starting from the dmx_address
"#);
    
    ap.refer(&mut dmx_size)
      .add_option(&["-d", "--dmx_size"], argparse::Store, &dmx_size_help);
    ap.refer(&mut dmx_baudrate)
      .add_option(&["-R", "--baud_rate"], argparse::Store, &dmx_baudrate_help);
    ap.refer(&mut addr)
      .add_option(&["-a", "--address"], argparse::Store, &addr_help);
    ap.refer(&mut port)
      .add_option(&["-p", "--port"], argparse::Store, &port_help);
    ap.refer(&mut universe)
      .add_option(&["-u", "--universe"], argparse::Store, &universe_help);
    ap.refer(&mut device_index)
      .add_option(&["-i", "--device_index"], argparse::Store, &device_index_help);
    ap.refer(&mut dmx_break_time)
      .add_option(&["-B", "--break_time"], argparse::Store, &dmx_break_time_help);
    ap.refer(&mut dmx_mab_time)
      .add_option(&["-M", "--mab_time"], argparse::Store, &dmx_mab_time_help);
    ap.refer(&mut dmx_idle_time)
      .add_option(&["-I", "--idle_time"], argparse::Store, &dmx_idle_time_help);
    ap.refer(&mut device_serial)
      .add_option(&["-s", "--device_serial"], argparse::Store, "select FTD2XX device by serial instead of index");
    ap.refer(&mut device_description)
      .add_option(&["-D", "--device_description"], argparse::Store, "select FTD2XX device by its description instead of index");
    ap.refer(&mut device_location_index)
      .add_option(&["-l", "--device_location"], argparse::Store, "select FTD2XX device by its location index instead of index");
    ap.refer(&mut list_devices)
      .add_option(&["-L", "--list_devices"], argparse::StoreTrue, "list all available FTD2XX devices");

    ap.parse_args_or_exit();
  }
  {
    if list_devices {
      let devices = ftd2xx::device_info_list().unwrap();
      println!("Found {} FTD2XX devices", devices.len());
      for i in 0..devices.len() {
        let device = &devices[i];
        println!("{})", i);
        println!("    id:            {}", device.id);
        println!("    description:   {}", device.description);
        println!("    serial number: {}", device.serial_number);
        println!("    type:          {}", device.device_type);
        println!("    location ID:   {}", device.location_id);
        println!("    flags:         {:?}", device.flags);
      }
      std::process::exit(0);
    }
    if dmx_size < 1 || dmx_size > 512 {
      println!("DMX size has to be between 1 and 512.");
      std::process::exit(1);
    }
    if dmx_break_time < 1 || dmx_break_time > 1000000 {
      println!("DMX break time has to be between 1us and 1000000us.");
      std::process::exit(1);
    }
    if dmx_mab_time < 1 || dmx_mab_time > 1000000 {
      println!("DMX MAB time has to be between 1us and 1000000us.");
      std::process::exit(1);
    }
    if dmx_idle_time < 1 || dmx_idle_time > 10000000000 {
      println!("DMX idle time has to be between 1us and 10000000000us.");
      std::process::exit(1);
    }
  }

  let dev = if device_location_index>=0 {
    ftd2xx::Device::open_by_location(device_location_index).unwrap()
  } else if device_serial != "" {
    ftd2xx::Device::open_by_serial(&device_serial).unwrap()
  } else if device_description != "" {
    ftd2xx::Device::open_by_description(&device_description).unwrap()
  } else {
    ftd2xx::Device::open(device_index).unwrap()
  };
  dev.reset().unwrap();
  dev.set_data_characteristics(ftd2xx::WordLength::Bits8, ftd2xx::StopBits::Bits2, ftd2xx::Parity::None).unwrap();
  dev.set_flow_control(ftd2xx::FlowControl::None).unwrap();
  dev.set_baud_rate(dmx_baudrate).unwrap();
  dev.set_latency_timer(2).unwrap();
  dev.purge(ftd2xx::PurgeFlags::RX | ftd2xx::PurgeFlags::TX).unwrap();
  dev.clear_rts().unwrap();
  let addr_port_str = addr.to_owned() + ":" + &port.to_string();
  let osc_address_starter = "/".to_owned() + &universe.to_string() + "/dmx/";

  let mut dmx = DMX::new(&dev, dmx_size);
  let data = std::sync::Arc::new(std::sync::Mutex::new(vec![0;dmx_size]));
  let data_t = data.clone();

  let osc_thread = std::thread::spawn(move ||{
    let sock = std::net::UdpSocket::bind(addr_port_str).unwrap();
    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
      match sock.recv_from(&mut buf) {
        Ok((size, addr)) => {
          let packet_err = rosc::decoder::decode_udp(&buf[..size]);
          match packet_err {
            Ok((_, packet)) => match packet {
              rosc::OscPacket::Message(msg) if msg.addr.starts_with(&osc_address_starter) => {
                match get_shift(&osc_address_starter, &msg.addr) {
                  Ok(shift) => {
                    let a_size = if shift + msg.args.len() < dmx_size {
                      msg.args.len()
                    } else {
                      dmx_size-shift
                    };
                    {
                      let mut data_u = data.lock().unwrap();
                      for i in 0..a_size {
                        (*data_u)[i+shift] = match msg.args[i] {
                          rosc::OscType::Int(a) => a.clamp(0, 0xff) as u8,
                          rosc::OscType::Long(a) => a.clamp(0, 0xff) as u8,
                          rosc::OscType::Float(f) => (f.clamp(0_f32, 1_f32)*255_f32) as u8,
                          rosc::OscType::Double(f) => (f.clamp(0_f64, 1_f64)*255_f64) as u8,
                          rosc::OscType::Char(a) => a as u8,
                          rosc::OscType::Bool(b) => b as u8,
                          _ => (*data_u)[i+shift],
                        };
                      }
                    }
                  },
                  _ => continue,
                }
              }
              _ => continue
            }
            _ => continue
          }
          
        }
        Err(e) => {
          println!("Error receiving from socket: {}", e);
          std::process::exit(1);
        }
      }
    }
  });
  
  loop {
    {
      let data = data_t.lock().unwrap();
      dmx.data[1..].copy_from_slice(&*data);
    }
    dmx.wait_and_send().unwrap();
  }
}
