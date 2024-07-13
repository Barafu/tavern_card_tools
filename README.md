# Tools for working with SillyTavern cards

## Currently supported functions:

* `tavern_card_tools.exe baya_get URL` - extract a character card from "Backyard AI" URL. Supports URLs that require registration. 

Obviously, more functions planned in the future. 

## Installation

Windows folks - download .EXE from [releases](https://github.com/Barafu/tavern_card_tools/releases/latest). No need to install, should just work. 

Linux crowd - you better build it from source. Download this repository. Install `cargo` and `rustc` packages. 
Type `cargo build --release` in the root folder of the repo. It will download dependencies and build.  Here is your app in `target/release` folder. 
