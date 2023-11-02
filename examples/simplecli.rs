use aranet_btle;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let device = aranet_btle::connect().await?;

    let data = device.read_data().await?;

    println!("co2: {}ppm", data.co2);
    println!("temperature: {}C", data.temperature);
    println!("pressure: {}hPa", data.pressure);
    println!("humidity: {}%", data.humidity);
    println!("battery: {}%", data.battery);
    println!("status: {}", data.status);
    println!("interval: {}", data.interval);
    println!("age: {}s", data.age);

    Ok(())
}