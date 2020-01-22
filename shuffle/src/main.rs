mod io;

use crate::io::files::FileReader;
use crate::io::{DataSource, ReadState};

fn main() {
    let matches = clap::App::new("")
        .name("shuffle")
        .about("Shuffle line of files or stream io")
        .get_matches();
    /* TODO:
        - File io
        - Stream io
        - CLI io
        - Output file
        - Output stream
        - Custom line termination
        - Line can occur multiple times
        - Custom random source
        - Input range
        - Amount of lines to output
    *
}
