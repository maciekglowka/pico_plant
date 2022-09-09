# PICO PLANT

**Tiny WIP Rasperry Pi Pico W sensor monitor.**

This code utilises the built-in Pico W wifi chip to send the connected sensor data to a remote API endpoint. A simple POST request, containing JSON body with the data, is sent at a specified interval. (by default every hour)

The code is build on top of the Embassy framework:
https://github.com/embassy-rs/embassy
(as at the moment it seems to be te only available Pico wifi drive for Rust.)

## Sensors
### DHT-22

DHT-22 is one of the two sensors supported in this repo. The driver has been written to be compatibile with the Embassy framework and uses it's `FlexPin` for serial communication.
(the communication requires the very same pin to be both read and written to)

Even though Embassy is an async framework, the DHT-22 is read in a blocking manner. During my initial tests I have realised that the serial communication pulses are to short in time to let the code return to the async loop. (packets were lost)

WARNING: the sub-zero temperatures has not been tested in real life and are potentially not implemented correctly. The documentation for my version of the DHT-22 sensor did not mention how the negative values are encoded. On the web you can find at least a few contradictory pieces of information on the subject (probably due to different sensor versions).
I had to pick one, but could not test it yet.

### SOIL MOISTURE

A simple capacitive soil moisture sensor is a second part of equipment included in the code. (although it should work also with the resistive one's - as its just a simple analog read).

The ADC has not been implemented yet for the Pico W in the Embassy Hal, so the implementation here is done rather at the PAC level.

## CONFIG AND BUILD

In order to build or run the code a number of env variables has to be set:

```
WIFI_NAME=SSID
WIFI_PASS=SECRET
HOST_NAME=0.0.0.0
HOST_IP=0.0.0.0
HOST_PORT=80
HOST_PATH=/api/report_endpoint
```

The `HOST_NAME` variable can often be a domain rather than IP - if the server is public. The DNS resolving is not implemented though - so the IP has to be specified in the POST request regardless.

As it is not handy to write those down every time, they can be stored in an `.env` file and loaded while running `Cargo` via:

```
env $(cat .env) cargo run --release
```

A `run.sh` script containing the above command is also included in the repo.