# Tools for working with SillyTavern cards

## Currently supported functions:

* `tavern_card_tools.exe baya_get --url <URL>` - extract a character card from "Backyard AI" URL. Supports URLs that require registration. Will automatically convert all instances of word `User`
into `{{user}}`
* `tavern_card_tools.exe de8 --path <filename.png>` - remove paired asterisks from all primary text fields of the card. Creates a new file for the output, named de8.filename.png, and leaves original as it is. 
Add `--force` flag to overwrite output file even if it already exists. 
* `tavern_card_tools.exe print --path <filename.png>` - print the meaningfull content of the character data to the terminal.
* `tavern_card_tools.exe print_all --path <filename.png>` - print all character data as JSON to the terminal.


Obviously, more functions planned in the future. 

## Installation

Windows folks - download .EXE from [releases](https://github.com/Barafu/tavern_card_tools/releases/latest). No need to install, should just work. 

Linux crowd - you better build it from source. Download this repository. Install `cargo` and `rustc` packages. 
Type `cargo build --release` in the root folder of the repo. It will download dependencies and build.  Here is your app in `target/release` folder. 
