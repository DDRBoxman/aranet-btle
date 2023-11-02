use btleplug::api::Characteristic;
use btleplug::platform::{Adapter, Manager, Peripheral};
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use byteorder::{ReadBytesExt, LittleEndian};
use uuid::{uuid, Uuid};
use thiserror::Error;

use std::io::Cursor;
use std::thread;
use std::time::Duration;

const CURRENT_READINGS_CHARACTERISTIC: Uuid = uuid!("f0cd3001-95da-4f4b-9ac8-aa55d312af0c");

pub struct Aranet4 {
    peripheral: Peripheral,
    current_reading_char: Characteristic,
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("failed to get a bluetooth adapter")]
    AdapterUnavaliable,
    #[error("aranet device not found")]
    DeviceNotFound,
    #[error("the Characteristic for UUID {0} was not found")]
    CharacteristicNotFound(String),
    #[error(transparent)]
    BTLEError(#[from] btleplug::Error),
}

pub async fn connect() -> Result<Aranet4, ConnectionError> {
    let manager = Manager::new().await?;

    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).ok_or(ConnectionError::AdapterUnavaliable)?;

    central.start_scan(ScanFilter::default()).await?;
    // todo: improve this hardcoded sleep
    thread::sleep(Duration::from_secs(2));
    let peripheral = find_by_name(&central).await.ok_or(ConnectionError::DeviceNotFound)?;

    peripheral.connect().await?;

    // Currently doesn't do anything
    peripheral.discover_services().await?;

     let chars = peripheral.characteristics();
     let current_reading_char = chars
         .iter()
         .find(|c| c.uuid == CURRENT_READINGS_CHARACTERISTIC)
         .ok_or(ConnectionError::CharacteristicNotFound(CURRENT_READINGS_CHARACTERISTIC.to_string()))?;

    Ok(Aranet4{
        peripheral,
        current_reading_char: current_reading_char.clone(),
    })
}

pub struct SensorData {
    pub co2: u16,
    pub temperature: f32,
    pub pressure: u16,
    pub humidity: u8,
    pub battery: u8,
    pub status: u8,
    pub interval: u16,
    pub age: u16,
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    BTLEError(#[from] btleplug::Error),
}

impl Aranet4 {
    pub async fn read_data(&self) ->Result<SensorData, ReadError> {
        let res = self.peripheral.read(&self.current_reading_char).await?;
    
        let mut rdr = Cursor::new(res);
        let co2 = rdr.read_u16::<LittleEndian>()?;
        let temperature = rdr.read_u16::<LittleEndian>()? as f32 / 20.0;
        let pressure = rdr.read_u16::<LittleEndian>()? / 10;
        let humidity = rdr.read_u8()?;
        let battery = rdr.read_u8()?;
        let status = rdr.read_u8()?;
        let interval = rdr.read_u16::<LittleEndian>()?;
        let age: u16 = rdr.read_u16::<LittleEndian>()?;
    
        Ok(
            SensorData{
                co2,
                temperature,
                pressure,
                humidity,
                battery,
                status,
                interval,
                age,
            }
        )
    }
}

async fn find_by_name(central: &Adapter) -> Option<Peripheral> {
    for p in central.peripherals().await.unwrap() {
        if p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| name.contains("Aranet4"))
        {
            return Some(p);
        }
    }
    None
}