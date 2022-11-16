# image-organizer

```
Simple command to organize your photo/video library

Usage: image-organizer [OPTIONS] <SOURCE> [TARGET]

Arguments:
  <SOURCE>  Source folder, where not so well organized media live
  [TARGET]  Target folder where image should be copied to, default to <SOURCE> (move files instead)

Options:
  -d, --dry-run  Don't actually rename
  -h, --help     Print help information
  -V, --version  Print version information
```

Images will be organized, in an edempotent way (You can re-run with the same source, and only new images will be added) in the following folder structure and file naming:
```
YEAR/YEAR-MM-DD/YEARMMDD-HHIISS.ext
```
Exif data is used for images, and the brightness value is added before the extension, to ensure uniqueness of names.