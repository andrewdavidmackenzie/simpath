[![Build Status](https://travis-ci.org/andrewdavidmackenzie/simpath.svg?branch=master)](https://travis-ci.org/andrewdavidmackenzie/simpath)

# Simpath
A small and simple crate (in the spirit of my "simp*" crates) for search paths, like `$PATH` and `$LD_PATH`.

# Example
Create a `Simpath` that loads from the `$PATH` environment variable using:

```
let path = simppath::new("PATH");
```

Then find a file called `filename` by searching the directories in `PATH` in order - as a `PathBuf`
```
let file = path.find("filename");
```

## Methods
* add a new directory to the search path
* get the name of the path 
* get the list of directories in the path
* find a file in the path
* find a file of type `File` or type `Directory` on the path
* check if the search path already contains a directory entry
* add to the search path, loading the entries from an environment variable
* validate that all directory entries in the path are valid, exist and can be read

# Traits
* implements the `fmt::Display` trait
* Derives `Clone`
* Derives `Debug`