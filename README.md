# sub-batch
Download and match subtitle files to video files

```
USAGE:
    sub-batch [FLAGS] [OPTIONS] [url]

FLAGS:
    -h, --help       Prints help information
    -r, --rename     If subs should be renamed to match the corresponding video file.
    -V, --version    Prints version information

OPTIONS:
    -e, --encoding <encoding>      Needed to parse text-based subtitle formats (only needed when adjusting timing).
                                   [default: utf-8]
        --fps <fps>                Needed for MicroDVD .sub files. Specifies the FPS that the video file is encoded in
                                   (only needed when adjusting timing). [default: 25]
    -p, --path <path>              The path to download to and look for subs in. [default: .]
    -s, --subarea <subarea>        Specifies a regular expression that defines the part of the subtitle filename where
                                   episode number should be extracted from.
    -t, --timing <timing>          Adjusts the timing of all subs. The value is specified in seconds, and can be
                                   negative and fractional.
    -v, --videoarea <videoarea>    Specifies a regular expression that defines the part of the video filename where
                                   episode number should be extracted from.

ARGS:
    <url>    The kitsunekko.net url to download subs from. May be omitted.
```
