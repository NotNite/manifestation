# manifestation

A command line tool for packaging GDWeave mods.

## Installation

Windows users can download from [the Releases tab](https://github.com/NotNite/manifestation/releases). Linux users can compile from source after installing Rust:

```shell
cargo install --git https://github.com/NotNite/manifestation.git
```

## Usage

First, setup manifestation in the terminal by double clicking it or running it with no arguments. You will be asked to answer a few questions.

Make a config file for your mod (canonically known as `manifestation.toml`):

```toml
id = "ExampleMod"
name = "Example Mod"
description = "Example mod description"
version = "1.0.0"
homepage = "https://github.com/NotNite/manifestation"
author = "You!"

icon = "icon.png"
readme = "README.md"
changelog = "CHANGELOG.md" # optional

 # optional
extra_files = [
  "example_gdnative_lib.dll",
  "example.txt"
]

# both are optional here, pick what you want
[project]
csharp = "./ExampleMod/ExampleMod.csproj"
godot = "./project/project.godot"

[[dependencies]]
thunderstore_version = "NotNet-GDWeave-2.0.12"

# example dependency
[[dependencies]]
id = "Sulayre.Lure"
thunderstore_version = "Sulayre-Lure-3.1.3"
```

Then either drag the config file onto the executable or run it from the command line.
