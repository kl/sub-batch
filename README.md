# sub-batch
Match and rename subtitles to videos and perform other batch operations on subtitles.

## Install

### Precompiled binaries (Linux and Windows)
https://github.com/kl/sub-batch/releases

### Nix package (credit to: https://github.com/erictapen)
https://github.com/NixOS/nixpkgs/blob/master/pkgs/applications/video/sub-batch/default.nix

### Cargo
```cargo install sub-batch```

## Usage

To see all subcommands and options, run ``sub-batch help``.

### Problem: I want to rename subtitle files to match their corresponding video file

---

Use the ```sub-batch rename``` command to match and rename subtitles to videos.
It works by scanning the subtitle and video file names for numbers, and renaming the subtitle to the
video file that contains the same number. For example, if we have the following files in a directory:

```
NameOfSeries_E01.srt
NameOfSeries_E02.srt
NameOfSeries_E03.srt
1_NameOfSeries.mkv
2_NameOfSeries.mkv
3_NameOfSeries.mkv
```

run ``sub-batch rename``:
```
> sub-batch rename
NameOfSeries_E01.srt -> 1_NameOfSeries.mkv
NameOfSeries_E02.srt -> 2_NameOfSeries.mkv
NameOfSeries_E03.srt -> 3_NameOfSeries.mkv
```

### Renaming when numbers are in different positions

Running ``sub-batch rename`` on the following files will fail. 
```
1337_NameOfSeries_E01.srt
1337_NameOfSeries_E02.srt
1337_NameOfSeries_E03.srt
1_NameOfSeries.mkv
2_NameOfSeries.mkv
3_NameOfSeries.mkv
```

This is because the first number in each sub (1337) is not present in the video files, so no match can be made.
The easiest way to solve this is to pass the ``--rev`` flag:

```
> sub-batch rename --rev
1337_NameOfSeries_E01.srt -> 1_NameOfSeries.mkv
1337_NameOfSeries_E02.srt -> 2_NameOfSeries.mkv
1337_NameOfSeries_E03.srt -> 3_NameOfSeries.mkv
```
by default, ``sub-batch`` looks for the first number form left-to-right in the file name,
but ``--rev`` changes the scan direction to right-to-left.
You can also use the ``--rs`` (reverse sub) and ``--rv`` (reverse video) flags to only change
the direction of sub or video files.

When the episode number isn't the first or last number in the file name, you can define a 
subsection of the file name and ``sub-batch`` will only look for numbers in this area.

This is done with the ``--subarea`` and ``--videoarea`` options, which take a regular expression that defines
the area to scan for numbers in. For example, if you have the following files:

```
1sub_file_08_xx2.srt
video8.mp4
```
run:
```
> sub-batch rename --subarea "file.*"
1sub_file_08_xx2.srt -> video8.mp4
```
Note that the regex does not have to match 
a number, it just defines the sub area in the file 
name where the number scan starts at.

### Problem: I want to change subtitle timings

---

To change the subtitle timings __for all subtitles in the directory__ use the `time` subcommand, for example:
```
sub-batch time 100
```
which moves all timings forward by 100 ms, or:
```
sub-batch time -50
```
which moves all timings back by 50 ms.

### Adjusting subtitle timings with `alass`

alass (https://github.com/kaegi/alass) can automatically adjust timings of a subtitle file and fix things such as gaps for commercial breaks
given the video file of the subtitle. To run `alass-cli` on all subtitle/video matches in parallel, run:
```
sub-batch alass
```
Arguments to `alass-cli` can be given by putting them after the alass subcommand in quotes:
```
sub-batch alass "--split-penalty 10"
```

### Adjusting subtitle timings interactively with `mpv`

If mpv (https://mpv.io) is installed sub-batch can use `mpv` to adjust timings interactively and have the updated subtitles auto-refresh in mpv. To enter this mode run:
```
sub-batch time-mpv
```
and follow the on-screen instructions to adjust the timings.

Note that at least one matched (same name except for extension) video file/subtilte file pair must exist in the target directory.
``sub-batch`` will only use the first matched pair it finds when adjusting,
but the timing adjustment is applied to __all subtitle files in the directory__, same as ```sub-batch time``` command.

### Problem: I want to target only certain subtitle/video files

---

You can give a regular expression to filter the subs/videos that should be included when running any of the subcommands.
For example, to only change timings of subtitles with the ``.srt`` extension in the target directory, run:
```
sub-batch --filter-sub "\.srt" time -50
```
Any other subtile files in the target directory are ignored. Video files can be filtered the same way with the ```--filter-video``` option.

### Problem: I want to match more than one subtitle to a single video file using secondary extensions

---

Let's say we have the following files:

```
NameOfSeries_E01.srt
NameOfSeries_E01.jp.srt
NameOfSeries_E01.en.srt
1_NameOfSeries.mkv
```

Running ``sub-batch rename`` will match all three subtitle files:
```
> sub-batch rename
NameOfSeries_E01.en.srt -> 1_NameOfSeries.mkv
NameOfSeries_E01.jp.srt -> 1_NameOfSeries.mkv
NameOfSeries_E01.srt    -> 1_NameOfSeries.mkv
```

and after renaming the secondary extensions are not deleted:
```
1_NameOfSeries.en.srt
1_NameOfSeries.jp.srt
1_NameOfSeries.mkv
1_NameOfSeries.srt
```

Secondary extensions are treated as part of the file extension only if the
following __two conditions are true__:

1. The secondary extension is no longer than 3 characters long.
2. The secondary extension doesn't contain a number.

The length check exists because mpv (by default) doesn't recognize secondary extensions longer than 3 characters,
and the number check exists because a number in the secondary extension could be used as the match number.

If any of the above conditions are false, the secondary extension is treated as part of the file stem
which means that the subtitle can only be uniquely matched to a single video.

You can change this default behavior with the ``--sec-always`` and ``--sec-never`` flags which can be 
given to the ``rename`` and ``alass`` subcommands. ``--sec-always`` unconditionally enables secondary 
extensions no matter how long or what characters they contain, while ```--sec-never``` disables handling of secondary 
extensions completely.
