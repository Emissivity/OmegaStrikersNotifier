# Omega Strikers Notifier
Simple Rust application to send a desktop notification when you find a match in Omega Strikers.

## Usage
**Windows**: `OmegaStrikersNotifier.exe` should work for most people. This will use the log file located in `%UserProfile\AppData\Local\OmegaStrikers\Saved\Logs\`.

**Linux**: `./OmegaStrikersNotifier` should work for most people. This will attempt to autodetect the location of the log file.

## Troubleshooting
### Can't find log file
- Use the `--log-path` argument to manually specify the location.
  - Example: `OmegaStrikersNotifier.exe --log-path "C:\\Users\\username\\AppData\\Local\\OmegaStrikers\\Saved\\Logs\\OmegaStrikers.log"`
- Ensure the app is allowed to read the log file
- Use the `--debug` argument to get more information.
  - Example: `OmegaStrikersNotifer.exe --debug`

## Is this bannable?
Probably not, as it doesn't modify or access any files besides the log file and `steamlibraryfolders.vdf` (used to autodetect the log path on Linux, unused on Windows). In the event I find out this is bannable, this readme will be updated.