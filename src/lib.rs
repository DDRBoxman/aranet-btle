use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::api::{CentralEvent, Characteristic};
use btleplug::platform::{Adapter, Manager, Peripheral, PeripheralId};
use byteorder::{LittleEndian, ReadBytesExt};
use thiserror::Error;
use tokio_stream::{Stream, StreamExt};
use uuid::{uuid, Uuid};

use std::collections::HashMap;
use std::io::Cursor;
use std::thread;
use std::time::{Duration, Instant};

const CURRENT_READINGS_CHARACTERISTIC: Uuid = uuid!("f0cd3001-95da-4f4b-9ac8-aa55d312af0c");
const ADVERTISED_SERVICE_UUID: Uuid = uuid!("0000fce0-0000-1000-8000-00805f9b34fb");

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
    let central = adapters
        .into_iter()
        .nth(0)
        .ok_or(ConnectionError::AdapterUnavaliable)?;

    central.start_scan(ScanFilter::default()).await?;
    // todo: improve this hardcoded sleep
    thread::sleep(Duration::from_secs(2));
    let peripheral = find_by_name(&central)
        .await
        .ok_or(ConnectionError::DeviceNotFound)?;

    peripheral.connect().await?;

    // Currently doesn't do anything
    peripheral.discover_services().await?;

    let chars = peripheral.characteristics();
    let current_reading_char = chars
        .iter()
        .find(|c| c.uuid == CURRENT_READINGS_CHARACTERISTIC)
        .ok_or(ConnectionError::CharacteristicNotFound(
            CURRENT_READINGS_CHARACTERISTIC.to_string(),
        ))?;

    Ok(Aranet4 {
        peripheral,
        current_reading_char: current_reading_char.clone(),
    })
}

pub async fn scan() -> Result<impl Stream<Item = ScanResult>, ConnectionError> {
    let mut sensor_reading_times: HashMap<PeripheralId, Instant> = HashMap::new();

    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters
        .into_iter()
        .nth(0)
        .ok_or(ConnectionError::AdapterUnavaliable)?;
    let events = central.events().await?;

    let filter = ScanFilter {
        services: vec![ADVERTISED_SERVICE_UUID],
    };

    // start scanning for devices
    central.start_scan(filter).await?;

    Ok(events.filter_map(move |event| match event {
        CentralEvent::ManufacturerDataAdvertisement {
            id,
            manufacturer_data,
        } => {
            let mut rdr = Cursor::new(&manufacturer_data[&1794]);

            rdr.set_position(8);
            match parse_data_in_cursor(rdr) {
                Ok(data) => {
                    let read_time = Instant::now();

                    if let Some(last_time) = sensor_reading_times.get(&id) {
                        if read_time < *last_time {
                            return None;
                        }
                    }

                    sensor_reading_times.insert(
                        id.clone(),
                        read_time + Duration::new(data.interval as u64 - data.age as u64 + 1, 0),
                    );

                    Some(ScanResult {
                        sensor_data: data,
                        id,
                    })
                }
                Err(_) => None,
            }
        }
        _ => None,
    }))
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

pub struct ScanResult {
    pub sensor_data: SensorData,
    pub id: PeripheralId,
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    BTLEError(#[from] btleplug::Error),
}

impl Aranet4 {
    pub async fn read_data(&self) -> Result<SensorData, DeviceError> {
        let res = self.peripheral.read(&self.current_reading_char).await?;

        let rdr = Cursor::new(&res);
        let data = parse_data_in_cursor(rdr)?;
        Ok(data)
    }

    pub async fn reconnect(&self) -> Result<(), DeviceError> {
        self.peripheral.connect().await?;

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), DeviceError> {
        self.peripheral.disconnect().await?;

        Ok(())
    }
}

fn parse_data_in_cursor(mut cursor: Cursor<&Vec<u8>>) -> Result<SensorData, std::io::Error> {
    let co2 = cursor.read_u16::<LittleEndian>()?;
    let temperature = cursor.read_u16::<LittleEndian>()? as f32 / 20.0;
    let pressure = cursor.read_u16::<LittleEndian>()? / 10;
    let humidity = cursor.read_u8()?;
    let battery = cursor.read_u8()?;
    let status = cursor.read_u8()?;
    let interval = cursor.read_u16::<LittleEndian>()?;
    let age: u16 = cursor.read_u16::<LittleEndian>()?;

    Ok(SensorData {
        co2,
        temperature,
        pressure,
        humidity,
        battery,
        status,
        interval,
        age,
    })
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
