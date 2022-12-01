# gb - A Game Boy emulator
Another Game Boy emulator.

## Status of the project
- Many games are somewhat playable
- Audio is unimplemented
- GBC support is unimplemented
- PPU has some minor bugs
- Interrupts have some minor bugs
- Functionality for savegames is there but not being written to/read from a file yet.

## Usage
- Use ``cargo run --`` followed by the subcommand.
- Each of the following subheaders is a subcommand in gb, alternatively try ``--help`` for more information.
### cart-info
Prints the cartridge header of the given rom.
#### Required
```
-c / --cart <PATH>
```
### emu
Opens a window and runs the emulator.
#### Controls
- Directions are WASD
- ',' and '.' are A and B
- Enter is START and RShift is Select
- 'p' to pause the emulator
- 'r' to reset the emulator
- ESCAPE to exit the program
#### Required
```
-c / --cart <PATH>
```
#### Optional
```
--bios <PATH>  // Optional Game Boy bios rom
--genie <PATH> // Optional Game Genie rom
```
### trace
Traces the emulator printing debug information to stdout for each instruction.
This feature is from when the emulator could not yet boot graphically and is not very useful anymore.
#### Required
```
-c / --cart <PATH>
```
#### Optional
```
--cycles <u64> // Stop running the trace after this many cycles
--verbose      // Print extra information for each instruction
```

## Known Issues
- See issues for games that hang or crash