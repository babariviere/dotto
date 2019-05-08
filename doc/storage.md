# Storage

## Design

Storage is a common interface to read/write files on different kind of file system.

It will provides the following methods:

- open: opens a file with given mode (read only, write only or read and write)
- create: creates a file/directory with given permissions (see chmod). 
- remove: remove file or directory if exists (and if empty for directory).
- copy: copy entry to another entry
- list: entries in directory (or returns nothing for file)

To abstract file path, we are using "virtual path". It will be a common representation
of file path. It will follows unix path design (e.g /home/user/path/to/file).

Each storage will contains a root and cannot be "escaped". For example, if we give this path "/../../..", it will be converted to <storage_root>/.
