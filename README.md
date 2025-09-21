# Compass
Embedded compass with extra features!

## Embassy Tasks
### Deep Sleep
Task that handeles the initiation and execution of deep sleep when a button is pressed.
### GPS
Handles receiving packets and parsing them.
### Magnetometer
Task that handles the reading of the compass unit and the processing of the raw data recieved.
### Main
Uses information from GPS to determine aplication state (Has fix or Has no Fix).
Final processing of compass heading.
Handles updating the display.

## GPIO
TODO

## Assets
All sprites are stored in the assets folder.

## Parts Manifest
1x Esp32c6
1x Push Button
1x GPS+Compass Module

## Features
Deep sleep and wakeup via push button.
Gps NavPvt packet receiving and parsing.
Hmc5883I (Magnetometer) reading and processing.

## Future Features
Batery Monitor
