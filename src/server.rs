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

macro_rules! unwrap_cont {
    ($option:expr, Option) => {
        match $option {
            Some(option) => option,
            None => {
                eprintln!("None value at {}:{}!", file!(), line!());
                continue;
            }
        }
    };
    ($result:expr, Result) => {
        match $result {
            Ok(result) => result,
            Err(why) => {
                eprintln!("Error value at {}:{}! {}", file!(), line!(), why);
                continue;
            }
        }
    };
}

macro_rules! check_stream {
    ($streams:expr, $stream:expr) => {
        match $stream.take_error() {
            Ok(option) => match option {
                Some(error) => match error.kind() {
                    std::io::ErrorKind::BrokenPipe => {
                        $streams.push($stream);
                    }
                    _ => unimplemented!(),
                },
                None => (),
            },
            Err(why) => {
                eprintln!("Failed to get error value from stream: {}", why);
            }
        }
    };
}

macro_rules! retain_write {
    ($stream:expr, $fmt:expr, $($arg:tt)*) => {
        match writeln!($stream, $fmt, $($arg)*) {
            Ok(_) => true,
            Err(why) => {
                eprintln!("Error writing to stream, removing it: {}", why);
                match $stream.shutdown(std::net::Shutdown::Both) {
                    Ok(_) => (),
                    Err(why) => {
                        eprintln!("Failed to shut down stream: {}", why);
                    }
                }
                false
            }
        }
    };
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
                // Sleep for half a second to avoid unnecessary looping
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

        // Parse the information given by the client
        let mut split = line.split(" ");
        let output = unwrap_cont!(split.next(), Option);
        let sub_type = unwrap_cont!(split.next(), Option);

        // Determine the type of the subscription to data
        let (sub_type, sub_data) = match sub_type {
            common::TAG_CMD => {
                let number: u16 = unwrap_cont!(unwrap_cont!(split.next(), Option).parse(), Result);
                (SubscribeType::Tag, SubscribeData::Tag(number))
            }
            common::TITLE_CMD => (SubscribeType::Title, SubscribeData::None),
            common::LAYOUT_CMD => (SubscribeType::Layout, SubscribeData::None),
            _ => unimplemented!(),
        };
        // Lock the global streams value
        let mut streams = streams_clone.lock().unwrap();

        let entry = streams.entry(output.to_string()).or_insert(HashMap::new());

        // Add the stream to the list
        entry
            .entry(sub_type)
            .or_insert(Vec::new())
            .push((stream, sub_data));
    });

    // Loop through the lines of the stdin from DWL
    for line in BufReader::new(stdin()).lines() {
        // Parse all of the values
        let line = unwrap_cont!(line, Result);
        let mut split = line.splitn(3, " ");
        let output = unwrap_cont!(split.next(), Option);
        let name = unwrap_cont!(split.next(), Option);
        let value = unwrap_cont!(split.next(), Option);

        let mut streams = streams.lock().unwrap();

        // If the output is not registered by any clients, ignore it
        if !streams.contains_key(output) {
            continue;
        }

        // Get all streams for the current output
        let map = match streams.get_mut(output) {
            Some(streams) => streams,
            None => continue,
        };

        // Work based on the data from DWL
        match name {
            "tags" => {
                // Get all the streams subscribed to Tag data
                let streams = match map.get_mut(&SubscribeType::Tag) {
                    Some(streams) => streams,
                    None => continue,
                };
                // Parse the tag values from the DWL output
                let mut tags_split = value.split(" ");
                let active: u16 =
                    unwrap_cont!(unwrap_cont!(tags_split.next(), Option).parse(), Result);
                let selected: u16 =
                    unwrap_cont!(unwrap_cont!(tags_split.next(), Option).parse(), Result);
                let urgent: u16 = unwrap_cont!(
                    unwrap_cont!(tags_split.skip(1).next(), Option).parse(),
                    Result
                );

                streams.retain_mut(|(stream, data)| {
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

                    retain_write!(
                        stream,
                        "{{ \"text\": \" {} \", \"class\": [\"{}\"] }}",
                        tag + 1,
                        classes.join("\",\"")
                    )
                });
            }
            "layout" => {
                let streams = match map.get_mut(&SubscribeType::Layout) {
                    Some(streams) => streams,
                    None => continue,
                };
                streams.retain_mut(|(stream, _)| {
                    retain_write!(stream, "{{ \"text\": \"{}\" }}", value)
                });
            }
            "title" => {
                let streams = match map.get_mut(&SubscribeType::Title) {
                    Some(streams) => streams,
                    None => continue,
                };
                streams.retain_mut(|(stream, _)| {
                    retain_write!(stream, "{{ \"text\": \"{}\" }}", value)
                });
            }
            _ => (),
        }
    }
    tx.send(()).unwrap();

    listener_thread.join().unwrap();

    std::fs::remove_file(common::SOCKET_PATH).unwrap();
}
