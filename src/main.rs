use std::collections::VecDeque;
use std::io::Write;
use tokio_stream::StreamExt;

enum Turn {
    Skip(usize),
    CaptureMeta,
    Capture(usize),
}

impl Turn {
    fn this_much(&self) -> usize {
        match self {
            Turn::Skip(n) | Turn::Capture(n) => *n,
            Turn::CaptureMeta => 1,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mqtt_uri = args.next().expect("missing mqtt uri");
    let mqtt_user = args.next().expect("missing mqtt username");
    let mqtt_pass = args.next().expect("missing mqtt password");
    let url = args.next().expect("missing url");
    let topic = args.next().expect("missing topic");
    if let Some(_) = args.next() {
        panic!("extra arguments provided");
    }
    let client = reqwest::Client::new();
    let res = client.get(&url).header("Icy-MetaData", "1").send().await?;
    let metaint: usize = res
        .headers()
        .get("Icy-MetaInt")
        .map_or(Ok(format!("{}", std::usize::MAX)), |v| {
            v.to_str().map(|s| s.to_string())
        })?
        .parse()?;
    let mqtt = paho_mqtt::CreateOptionsBuilder::new()
        .server_uri(mqtt_uri)
        .client_id("umiarkonowy")
        .create_client()?;
    mqtt.connect(
        paho_mqtt::ConnectOptionsBuilder::new()
            .user_name(mqtt_user)
            .password(mqtt_pass)
            .automatic_reconnect(
                std::time::Duration::new(1, 0),
                std::time::Duration::new(120, 0),
            )
            .finalize(),
    )
    .await
    .expect("connection to broker failed");
    let mut stream = res.bytes_stream();
    let mut bytes: VecDeque<u8> = VecDeque::new();
    let mut turn = Turn::Skip(metaint);
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        bytes.extend(chunk);
        while bytes.len() >= turn.this_much() {
            turn = match turn {
                Turn::Skip(n) => {
                    stdout.write_all(&bytes.drain(0..n).collect::<Vec<_>>())?;
                    Turn::CaptureMeta
                }
                Turn::CaptureMeta => Turn::Capture(bytes.pop_front().unwrap() as usize * 16),
                Turn::Capture(n) => {
                    let metadata = String::from_utf8(bytes.drain(0..n).collect())
                        .expect("invalid utf8 in metadata");
                    for (key, value) in metadata
                        .split("';")
                        .map(|s| s.trim().trim_matches('\0'))
                        .filter(|s| !s.is_empty())
                        .filter(|s| s.contains("='"))
                        .map(|s| s.splitn(2, "='"))
                        .map(|mut s| {
                            let k = s.next().unwrap();
                            let v = s.next().unwrap();
                            (k, v)
                        })
                    {
                        mqtt.publish(
                            paho_mqtt::MessageBuilder::new()
                                .topic(format!("{}/{}", &topic, key))
                                .payload(value)
                                .qos(2)
                                .retained(true)
                                .finalize(),
                        )
                        .await?
                    }
                    Turn::Skip(metaint)
                }
            }
        }
    }
    Ok(())
}
