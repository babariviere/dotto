# Dot architecture

This document describe current dot architecture. Things may be unclear for now but everything may change in this document.

### Storage

Storage is an interface between the application and a storage. It should emulate a file system. You will be able to open files, directories and links. 

Storage introduce two types for better abstraction:

- virtual path: a path compatible with all storage. virtual path can be converted to system path and vice versa.
- entry: an entry is a path associated to one of these types:
	+ file: an entry that contains data
	+ directory: an entry that can contains multiples entries
	+ link: an entry that points to another entry

Implementation should have these method implements:

- open(vpath): open a file with given key (represent a path)
- list(vpath): list all files in directory (not recursive). returns error if not directory
- remove(entry): delete file from file system, if it's a directory, it will be deleted only if empty and if it's a link, only the pointer will be removed, not the pointed entry.
- checksum(entry1, entry2): compare two files to check if they are equals (this is subject to change).
- create(entry): create entry with given type
- copy(entry1, entry2): copy entry1 to entry2. this should allow copy of directories but not in recursive.
- entry\_kind(vpath, check\_link): give entry kind for given virtual path.
- chdir(vpath): change directory to vpath. this changes must changes all base path for all others methods.

These methods will be implemented by default but can be override:

- create\_all(entries): create all entries in given order
- remove\_all(entries): remove all entries in given order
- copy\_all(entries): copy all entries in given order
- entries(vpaths): returns all entries for all virtual paths

### Synchronization

Synchronization will be made on top of storage.

It will be used to calculate diff between two storage (based on base virtual paths) and to synchronize (one way sync) given diffs from one to another storage.

Two methods are provided:

- sync\_diff(src, dst, settings): calculate differences between source and destination. optional settings are given to control depth or exclude files.
- sync(src, dst, diffs): apply all diffs from diffs list using as roots source and destination.
