use std::{
    collections::HashMap,
    io::{stdin, BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

mod common;

#[derive(Hash, Eq, PartialEq, Debug)]
enum SubscribeType {
    Tag,
    Title,
    Layout,
}

#[derive(Debug)]
enum SubscribeData {
    Tag(u16),
    None,
}

fn main() {
    // Create the listener and set it to non-blocking to allow for inter-thread communication
    let listener = UnixListener::bind(common::SOCKET_PATH).unwrap();
    listener.set_nonblocking(true).unwrap();

    // The structure for all of the connected streams
    let streams: Arc<
        Mutex<HashMap<String, HashMap<SubscribeType, Vec<(UnixStream, SubscribeData)>>>>,
    > = Arc::new(Mutex::new(HashMap::new()));

    // Clone it for the listener thread
    let streams_clone = Arc::clone(&streams);

    // Create the communication channel for a clean shutdown at a later date
    let (tx, rx) = mpsc::channel::<()>();

    let listener_thread = thread::spawn(move || loop {
        // Get a currently waiting stream, or if there is no stream connecting check for the exit
        // message
        let stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(_) => {
                match rx.try_recv() {
                    Ok(_) => {
                        // Shutdown all of the streams
                        let streams = streams_clone.lock().unwrap();
                        for (_, map) in streams.iter() {
                            for (_, streams) in map.iter() {
                                for (stream, _) in streams {
                                    stream.shutdown(std::net::Shutdown::Both).unwrap();
                                }
                            }
                        }
                        break;
                    }
                    Err(_) => (),
                }
                thread::sleep(Duration::from_millis(500));
                continue;
            }
        };
        // Read the initial information from the client to determine what it wants
        let line = match BufReader::new(&stream).lines().nth(0) {
            Some(line) => match line {
                Ok(line) => line,
                Err(why) => {
                    eprintln!("Failed to fetch client info: {}", why);
                    continue;
                }
            },
            None => {
                eprintln!("Failed to fetch client info");
                continue;
            }
        };

        let mut split = line.split(" ");
        let output = split.next().unwrap();
        let sub_type = split.next().unwrap();

        // Determine the type of the subscription to data
        let (sub_type, sub_data) = match sub_type {
            common::TAG_CMD => {
                let number: u16 = split.next().unwrap().parse().unwrap();
                (SubscribeType::Tag, SubscribeData::Tag(number))
            }
            common::TITLE_CMD => (SubscribeType::Title, SubscribeData::None),
            common::LAYOUT_CMD => (SubscribeType::Layout, SubscribeData::None),
            _ => unimplemented!(),
        };
        let mut streams = streams_clone.lock().unwrap();

        let entry = streams.entry(output.to_string()).or_insert(HashMap::new());

        entry
            .entry(sub_type)
            .or_insert(Vec::new())
            .push((stream, sub_data));
    });

    thread::sleep(Duration::from_secs(5));

    println!("{:?}", streams);

    for line in BufReader::new(stdin()).lines() {
        let line = line.unwrap();
        let mut split = line.splitn(3, " ");
        let output = split.next().unwrap();
        let name = split.next().unwrap();
        let value = split.next().unwrap();

        let mut streams = streams.lock().unwrap();

        let map = match streams.get_mut(output) {
            Some(streams) => streams,
            None => continue,
        };

        match name {
            "tags" => {
                let streams = match map.get_mut(&SubscribeType::Tag) {
                    Some(streams) => streams,
                    None => continue,
                };
                let mut tags_split = value.split(" ");
                let active: u16 = tags_split.next().unwrap().parse().unwrap();
                let selected: u16 = tags_split.next().unwrap().parse().unwrap();
                let urgent: u16 = tags_split.skip(1).next().unwrap().parse().unwrap();

                for (stream, data) in streams.iter_mut() {
                    let tag = *match data {
                        SubscribeData::Tag(number) => number,
                        _ => unreachable!(),
                    };
                    let mask = 1 << tag;

                    let mut classes = Vec::new();
                    if active & mask == mask {
                        classes.push("active");
                    }
                    if selected & mask == mask {
                        classes.push("selected");
                    }
                    if urgent & mask == mask {
                        classes.push("urgent");
                    }
                    writeln!(
                        *stream,
                        "{{ \"text\": \"{}\", \"classes\": [\"{}\"] }}",
                        tag + 1,
                        classes.join("\",\"")
                    )
                    .unwrap();
                }
            }
            _ => (),
        }
    }
    tx.send(()).unwrap();

    listener_thread.join().unwrap();

    std::fs::remove_file(common::SOCKET_PATH).unwrap();
}
