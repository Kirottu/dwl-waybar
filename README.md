# dwl-waybar
A program to use waybar with DWL

# Why
The shell script I used before for waybar in DWL, was horribly inefficient. Changing workspaces or focus would cause my CPU to spike up to **80%**. It also caused horrible stuttering in games.

# Usage
Start by installing the binaries into `~/.cargo/bin` running `cargo install --path .` in the projects folder.

Then you must make sure that when DWL is started, the output is piped into the `dwl-waybar-server` binary like this: `/path/to/your/dwl | /home/<your user>/.cargo/bin/dwl-waybar-server`

For the waybar config here is an example:
```
    "modules-left": ["custom/dwl_tag#0", "custom/dwl_tag#1", "custom/dwl_tag#2", "custom/dwl_tag#3", "custom/dwl_tag#4", "custom/dwl_tag#5", "custom/dwl_tag#6", "custom/dwl_tag#7", "custom/dwl_tag#8", "custom/dwl_layout", "custom/dwl_title"],
    "custom/dwl_tag#0": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 tag 0",
     "format": "{}",
     "return-type": "json"
    },
    "custom/dwl_tag#1": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 tag 1",
     "format": "{}",
     "return-type": "json"
    },
    "custom/dwl_tag#2": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 tag 2",
     "format": "{}",
     "return-type": "json"
    },
    "custom/dwl_tag#3": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 tag 3",
     "format": "{}",
     "return-type": "json"
    },
    "custom/dwl_tag#4": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 tag 4",
     "format": "{}",
     "return-type": "json"
    },
    "custom/dwl_tag#5": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 tag 5",
     "format": "{}",
     "return-type": "json"
    },
    "custom/dwl_tag#6": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 tag 6",
     "format": "{}",
     "return-type": "json"
    },
    "custom/dwl_tag#7": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 tag 7",
     "format": "{}",
     "return-type": "json"
    },
    "custom/dwl_tag#8": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 tag 8",
     "format": "{}",
     "return-type": "json"
    },
    "custom/dwl_layout": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 layout",
     "format": "{}",
     "escape": true,
     "return-type": "json"
    },
    "custom/dwl_title": {
     "exec": "/home/kirottu/.cargo/bin/dwl-waybar-client eDP-1 title",
     "format": "{}",
     "escape": true,
     "return-type": "json"
    },
```
Of course replace the path to your binaries with the correct path, and replace the output (in the example eDP-1) with the output you want data from.

For the CSS here is an example containing the classes as well:
```
#custom-dwl_layout {
}

#custom-dwl_title {
}

#custom-dwl_tag {
}

#custom-dwl_tag.selected {
}

#custom-dwl_tag.urgent {
}

#custom-dwl_tag.active {
}
```

# How it works
The server reads from STDIN all the status messages that DWL outputs in it's STDOUT. The clients will connect to it via a unix socket, and the server will constantly feed the clients with new data as it is received, so updates to the bar are instant and efficient.

# Why not a patch
I am far more comfortable with writing rust than C, and I already had experience working with unix sockets and strings in general with rust. It also keeps the DWL source code more simple.
