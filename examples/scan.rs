use std::error::Error;

use tokio::pin;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream = aranet_btle::scan().await?;
    pin!(stream);

    while let Some(data) = stream.next().await {
        println!("DEVICE: {}", data.id);
        println!("co2: {}ppm", data.sensor_data.co2);
        println!("temperature: {}C", data.sensor_data.temperature);
        println!("pressure: {}hPa", data.sensor_data.pressure);
        println!("humidity: {}%", data.sensor_data.humidity);
        println!("battery: {}%", data.sensor_data.battery);
        println!("status: {}", data.sensor_data.status);
        println!("interval: {}", data.sensor_data.interval);
        println!("age: {}s", data.sensor_data.age);
    }

    Ok(())
}
