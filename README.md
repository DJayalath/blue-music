# blue-music
A super light-weight music player written in Rust. This is just a project that delves into learning Rust and extends the example given in [Rust Programming By Example](https://www.packtpub.com/application-development/rust-programming-example) hopefully until it becomes a fully featured music player.

###### Note to self: Qt may be worth using over GTK+ in a future reiteration of this project

![Now playing](/screenshots/v0-1-2.png?raw=true "Optional Title")

## Testing Instructions
### Download
Grab the latest binaries [here](https://github.com/armytricks/blue-music/releases/latest)
### Linux
1. Extract the archive
2. Navigate to 'blue-music' directory
3. Make sure you own the binary file: `$ chmod +x blue-music`
4. Execute: `$ ./blue-music`
### Windows
1. Double-click

## Goals
- A decent desktop music player
- Smoother transitions between songs (closest moods, genres, volume, etc.)

## Complete
- CMD-line shuffling and playing
- Support for FLACs
- Genre-scored shuffling by hill climb

## Todo
- ~~GUI~~ ~~A better looking GUI~~
- ~~More accurate song timing~~
- ~~Opening multiple files / open a folder~~ Options for both
- ~~Refactor code to use relm patterns~~ ~~Refactor to be safer~~
- ~~Fast-forwarding~~ Faster fast-forwading
- Support for MP3, WAV, WEBM, OGG
- NN-based intelligent shuffling
- Actually getting to a releasable state

## Known Issues
- Failure to display certain genre strings in now playing bar due to special characters
