# umiarkowanie-nowy-swiat

This is a result of my frustration frustration with the website of [Radio Nowy
Świat](https://nowyswiat.online) which loads very slowly and shows currently
playing song by fetching a .txt file from the server every 15 or 30 seconds.

I thought there must be a better way which is how I arrived at this thing
which listens to the stream and consumes only the metadata, pushing them to
designated MQTT server. Then, assuming you're using Mosquitto or some other
MQTT server that can have a WebSocket listener, you can consume the metadata
in realtime from your web browser.

This repository does not contain the web frontend (yet).

## how to run

```shell
cargo run tcp://my-mqtt-server:port mqttuser mqttpass https://stream.nowyswiat.online/mp3 basetopic
```

This will connect to things according to arguments provided, and if
`StreamTitle` metadata shows up in the stream, it will be published to
`basetopic/StreamTitle`.
