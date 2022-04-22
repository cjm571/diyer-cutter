# Requirements
## Hardware
* The device shall be able to feed small-gauge wire to a cutting head.
* The device shall have a safety covering over the cutting head.
* The device shall be able to cut small-gauge wire.
* The device shall be able to cut small-gauge wire with an accuracy of ±1/16in
* The device shall have a single power on/off switch.
* The device shall display information and user prompts on a 16x2 LCD.
* The device shall accept user input from a 3x4 matrix keypad.
* The device shall drop cut wire into a receptacle.
* The device shall use a BBC micro:bit V2 as a system controller.
* The device shall enter a safe "fail state" if any errors are encountered.

## Software
* The software shall allow the user to select the desired size of cut wire.
* The software shall allow the user to select the number of wires to cut.
* The software shall be able to detect when a spool of wire has run out.

# Architecture
## State Machine
The micro:bit must maintain a set of states and transitions in order to provide a straightforward experience.

## Initialization
Upon receiving power, the peripheral devices attached to the micro:bit must be initialized (and optionally tested). This module will be responsible for performing these initialization sequences.

## Input Parsing
Inputs from peripheral devices must be received and parsed in order to determine the status of the system and perform to user specification.

## Motor Driver(s)
The motor(s) must be driven to user specification.

# Design
## Statechart
![Statechart](./uml/statechart_top.png)

## Initialization
Upon power-up, the micro:bit will take the following sequence of actions:

1. Verify communications with LCD
2. Verify communications with keypad
3. Verify communications with motor(s) 
4. Enter Idle state

## Idle
A largely quiescent state where the micro:bit will sit idle until the user begins providing input via the keypad.

## Preparation
![Statechart](./uml/statechart_prep.png)
This is the input-collection state. A series of prompts will be presented to the user on the LCD to determine the parameters of the cuts that should be made. The user will respond to the prompt and press "`#`" to confirm the response; "`*`" can be used as a backspace button. The prompts will be:

1. Cut length
```
0123456789012345
CUT LENGTH (in):
-> _
```
2. Number of cuts
```
0123456789012345
NUMBER OF CUTS:
-> _
```
3. Final confirmation
```
0123456789012345
XXin x YYYY
OK? (#=Y, *=N) _
```

If the user chooses "`*`" for "No" at the final prompt, the micro:bit will return to the first prompt. If the inputs are confirmed, the micro:bit will progress to the Operation stage.

## Operation
![Statechart](./uml/statechart_op.png)
This is the cutting stage. Before cutting can begin, safety sensors are checked to ensure that the system is ready for safe operation. If any safety sensors are not in the right state, the micro:bit will prompt the user to check the associated safety device. The user may enter an override code to bypass the sensors in the event of an undetected error preventing the sensor from reporting correctly.

Safety checks include:

* Wire is properly fed
* Cutter guard is in place
* Cut wire receptacle is in place

Once safety checks have passed (or been overridden) the micro:bit will perform the following sequence of actions

1. Command feed stepper motor to advance the appropriate number of steps to reach the user-specified cut length.
2. Command cutter servo motor to engage the cutting head for one cycle
3. Increment cut counter
4. Repeat steps 1-3 until user-specified cut count is reached

Upon successful completion of all cuts, the micro:bit will return the system to the Idle state, where the user may begin the process again with more wire.