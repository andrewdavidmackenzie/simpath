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

# Platforms
The following platforms are supported and tested in CI
* Linux
* MacOS
* Windows (with the ";" separator character used for parsing paths from environment variables)

# Methods
* create a search path, initialized from an environment variable
* create a search path, initialized form an environment variable, using a custom separator character
* add a new directory to the search path
* get the name of the path 
* get the list of directories in the path
* find a file in the path
* find a file of type `File` or type `Directory` on the path
* check if the search path already contains a directory entry
* add to the search path, loading the entries from an environment variable
* add to the search path, loading the entries from an environment variable, using a custom separator character
* validate that all directory entries in the path are valid, exist and can be read

## Optional methods
These methods are activated by the "urls" feature, which is included by default. 

To remove that code and dependencies disable all default features using the `cargo` command line option
`--no-default-features` or including `default-features = false` in your `Cargo.toml` section for `Simpath`

* Add a URL to the search path

# Traits
* implements the `fmt::Display` trait
* Derives `Clone`
* Derives `Debug`