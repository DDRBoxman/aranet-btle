# Aranet btle

A simple library to get readings from an Aranet4 co2 device

```
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
```

### Roadmap:
- [x] Connect to one device
- [x] Get readings
- [ ] Allow connecting to multiple devices
- [ ] Connect to a specific device
- [ ] Fetch sensor history data
- [ ] Better management of the btle code so we can play nice with other libraries.
