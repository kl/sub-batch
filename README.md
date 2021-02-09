# sub-batch
Match and rename subtitle files to video files and perfom other batch operations on subtitle files.

## Install
```cargo install sub-batch```

## Usage
```
USAGE:
    sub-batch [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help          Prints help information
    -y, --no-confirm    If this flag is set sub-batch will not ask for any confirmation before applying operations.
    -V, --version       Prints version information

OPTIONS:
    -p, --path <path>    The path to look for subs in. [default: .]

SUBCOMMANDS:
    alass     Adjusts the timing of all subs that are matched with a video file using `alass`
              (https://github.com/kaegi/alass). This can automatically fix wrong timings due to commercial breaks
              for example.
    help      Prints this message or the help of the given subcommand(s)
    rename    Renames subtitle files to match the corresponding video file
    time      Adjusts the timing of all subs. The value is specified in milliseconds, and can be negative
    time-mpv  Adjusts the timing of all subs interactively using mpv. `mpv` must be installed on the system for
              this command to work.
```
## Renaming subtitle files to match their corresponding video file
Put the subs and the videos in the same directory, for example:
```
> ls
 Fullmetal_Alchemist_Brotherhood_001.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E01 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 Fullmetal_Alchemist_Brotherhood_002.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E02 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 Fullmetal_Alchemist_Brotherhood_003.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E03 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 Fullmetal_Alchemist_Brotherhood_004.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E04 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 ```
Run `sub-batch rename` to rename the subtitle files to have the same name as the corresponding video file:
```
> sub-batch rename
./[Reaktor] Fullmetal Alchemist Brotherhood - E01 v2 [1080p][x265][10-bit][Dual-Audio].mkv -> ./Fullmetal_Alchemist_Brotherhood_001.ass
./[Reaktor] Fullmetal Alchemist Brotherhood - E02 v2 [1080p][x265][10-bit][Dual-Audio].mkv -> ./Fullmetal_Alchemist_Brotherhood_002.ass
./[Reaktor] Fullmetal Alchemist Brotherhood - E03 v2 [1080p][x265][10-bit][Dual-Audio].mkv -> ./Fullmetal_Alchemist_Brotherhood_003.ass
./[Reaktor] Fullmetal Alchemist Brotherhood - E04 v2 [1080p][x265][10-bit][Dual-Audio].mkv -> ./Fullmetal_Alchemist_Brotherhood_004.ass
Ok? (y/n)
```
 By deafult sub-batch extracts the first number (searching from left to right) in the subtitle file name and then tries to find another non-subtitle file that contains that number.
 
### Limiting the number match area
The previous example matches the files correctly automatically but sometimes the area of the file names that the number is extracted from needs to be limited.
Consider for example:
```
> ls
 1337_Fullmetal_Alchemist_Brotherhood_001.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E01 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 1337_Fullmetal_Alchemist_Brotherhood_002.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E02 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 1337_Fullmetal_Alchemist_Brotherhood_003.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E03 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 1337_Fullmetal_Alchemist_Brotherhood_004.ass  '[Reaktor] Fullmetal Alchemist Brotherhood - E04 v2 [1080p][x265][10-bit][Dual-Audio].mkv'
 > sub-batch rename
 found no match for any sub file
```
No matches are found because the first number in each subtitle is now 1337. To fix this specify the subtitle area like this:
```
sub-batch rename --subarea "hood_.+"
```
The regular expression given to --subarea (and --videoarea) limits the number extraction to only the part of the file name that is matched by the regular expression.

## Adjusting subtitle timings

sub-batch can also batch adjust the timings for all subtitle files in the target directory. To do this you use the `time` subcommand, for example:
```
sub-batch time 100
```
which moves all subtitles forward by 100 ms, or:
```
sub-batch time -50
```
which moves all subtitles back by 50 ms.

## Adjusting subtitle timings interactively with `mpv`

If mpv (https://mpv.io) is installed sub-batch can use `mpv` to adjust timings interactively and have the updated subtitles auto-refresh in mpv. To enter this mode run:
```
sub-batch time-mpv
```
and follow the on screen instructions to adjust the timings.

Note that at least one matched (same name except for extension) video file/subtilte file pair must exist in the target directory. sub-batch will only use the first matched pair it finds when adjusting, but the timing adjustment is applied to all subtitle files in the directory, same as ```sub-batch time``` command.  

## Adjusting subtitle timings with `alass`

alass (https://github.com/kaegi/alass) can automatically adjust timings of a subtitle file and fix things such as gaps for commercial breaks 
given the video file of the subtitle. To run `alass-cli` on all subtitle/video matches in parallel, run:
```
sub-batch alass
```
Arguments to `alass-cli` can be given by putting them after the alass subcommand in quotes:
```
sub-batch alass "--split-penalty 10"
```
