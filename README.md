# sub-batch
Download and match subtitle files to video files

### Install
```cargo install sub-batch```

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
    -t, --timing <timing>          Adjusts the timing of all subs. The value is specified in milliseconds, and can be
                                   negative.
    -v, --videoarea <videoarea>    Specifies a regular expression that defines the part of the video filename where
                                   episode number should be extracted from.

ARGS:
    <url>    The kitsunekko.net url to download subs from. May be omitted.
```
### Matching subtile files and video files
Put the subs and the videos in the same directory, for example:
```
> ls
 Fullmetal_Alchemist_Brotherhood_001.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E01 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 Fullmetal_Alchemist_Brotherhood_002.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E02 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 Fullmetal_Alchemist_Brotherhood_003.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E03 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 Fullmetal_Alchemist_Brotherhood_004.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E04 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 ```
Run sub-batch with the --rename flag to rename the subtitle files to have the same name as the corresponding video file:
```
> sub-batch --rename
./[Reaktor] Fullmetal Alchemist Brotherhood - E01 v2 [1080p][x265][10-bit][Dual-Audio].mkv -> ./Fullmetal_Alchemist_Brotherhood_001.ass
./[Reaktor] Fullmetal Alchemist Brotherhood - E02 v2 [1080p][x265][10-bit][Dual-Audio].mkv -> ./Fullmetal_Alchemist_Brotherhood_002.ass
./[Reaktor] Fullmetal Alchemist Brotherhood - E03 v2 [1080p][x265][10-bit][Dual-Audio].mkv -> ./Fullmetal_Alchemist_Brotherhood_003.ass
./[Reaktor] Fullmetal Alchemist Brotherhood - E04 v2 [1080p][x265][10-bit][Dual-Audio].mkv -> ./Fullmetal_Alchemist_Brotherhood_004.ass
Ok? (y/n)
```
 By deafult sub-batch extracts the first number (searching from left to right) in the subtitle file name and then tries to find another non-subtitle file that contains that number.
 
### Limiting the number match area
The previous example matches the files correctly by default but sometimes it is needed to restrict the area of the file names that the number is extracted from. Consider for example:
```
> ls
 1337_Fullmetal_Alchemist_Brotherhood_001.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E01 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 1337_Fullmetal_Alchemist_Brotherhood_002.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E02 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 1337_Fullmetal_Alchemist_Brotherhood_003.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E03 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 1337_Fullmetal_Alchemist_Brotherhood_004.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E04 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 > sub-batch --rename
 found no match for any sub file
```
No matches are found because the first number in each subtitle is now 1337. To fix this specify the subtitle area like this:
```
sub-batch --rename --subarea "hood_.+"
```
The regular expression given to --subarea (and --videoarea) limits the number extraction to only the part of the file name that is matched by the regular expression.

### Adjusting the subtitle timings

sub-batch can also batch adjust the timings for all subtitle files. To do this you use the --timing option, for example:
```
sub-batch --timing 100
```
which moves all subtitles forward by 100 ms, or:
```
sub-batch --timing -50
```
which moves all subtitles back by 50 ms.
