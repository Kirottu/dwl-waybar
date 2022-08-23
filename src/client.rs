mod common;
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

fn main() {
    let mut args = std::env::args().skip(1);

    let output = match args.next() {
        Some(output) => output,
        None => {
            eprintln!("No output specified!");
            return;
        }
    };
    let name = match args.next() {
        Some(name) => name,
        None => {
            eprintln!("No name specified!");
            return;
        }
    };

    let mut stream = match UnixStream::connect(common::SOCKET_PATH) {
        Ok(stream) => stream,
        Err(why) => {
            eprintln!("Failed to connect to dwl-waybar daemon: {}", why);
            return;
        }
    };

    match match name.as_str() {
        common::TAG_CMD => {
            let tag: u16 = match args.next() {
                Some(tag) => match tag.parse::<u16>() {
                    Ok(tag) => tag,
                    Err(why) => {
                        eprintln!("Failed to parse tag number: {}", why);
                        return;
                    }
                },
                None => {
                    eprintln!("No tag number specified!");
                    return;
                }
            };
            writeln!(stream, "{} {} {}", output, common::TAG_CMD, tag)
        }
        common::TITLE_CMD => writeln!(stream, "{} {}", output, common::TITLE_CMD),
        common::LAYOUT_CMD => writeln!(stream, "{} {}", output, common::LAYOUT_CMD),
        _ => unimplemented!(),
    } {
        Ok(_) => (),
        Err(why) => {
            eprintln!("Failed to send inital info: {}", why);
            return;
        }
    };

    for line in BufReader::new(stream).lines() {
        match line {
            Ok(line) => {
                println!("{}", line);
            }
            Err(why) => {
                eprintln!("Failed to retrieve next value: {}", why);
                return;
            }
        }
    }
}
