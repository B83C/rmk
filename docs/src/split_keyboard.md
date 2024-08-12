# Split keyboard

<div class="warning">
This feature is currently not implemented, this document is a draft writeup
</div>

## Design

### Usage design

Defining a split keyboard should be as simple as a normal keyboard. The split-keyboard's type and matrix should be defined in the `keyboard.toml`.

```toml
# Split keyboard definition(draft)
[matrix]
# Total rows & cols, which should be larger than possible row/col number of all splits
rows = 4
cols = 7
layers = 2

[split]
split = true
# Connection type between master & slave
connection = "ble"/"uart"?
main = "left" # or "right"

[split.left]
# Pin assignment
input_pins = ["P1_00", "P1_01", "P1_02", "P1_07"]
output_pins = ["P1_05", "P1_06", "P1_03", "P1_04"]

# Number of row/col
# Should be consistent with input_pins & output_pins
row = 4
col = 4
# If it's main, the both offsets default to 0. However, they can be changed
# col_offset = 0
# row_offset = 0

# If the connection type is ble
ble_addr = ""
# If the connection type is uart
uart_instance = ""

[split.right]
input_pins = ["P1_00", "P1_01", "P1_02", "P1_07"]
output_pins = ["P1_05", "P1_06", "P1_03"]

# Number of row/col
# Should be consistent with input_pins & output_pins
row = 4
col = 3

# Offset of row/col of current board
# The total number of rows and cols should not be larger than both sides' row num + row offset
col_offset = 4
row_offset = 0

# If the connection type is ble
ble_addr = ""
# If the connection type is uart
uart_instance = ""

# In case there's multi-split keyboard
[split.top]

# Other fields are same

```

### Communication between left & right

When the left & right talk to each other, the **debounced key states** are sent. The main(can be either left or right) receives the key states, converts them to actual keycode and then sends keycodes to the host.

That means the main side should have a full keymap stored in the storage/ram. The other side just do matrix scanning, debouncing and sending key states over i2c/uart/ble.

### Split project template

A project of split keyboard should like:

```
src
 - bin
   - right.rs
   - left.rs
keyboard.toml
Cargo.toml
```

### Communication protocol

A single message can be defined like:

```rust
pub enum SplitMessage {
    /// Activated key info (row, col, pressed), from slave to master.
    /// Only key changes are sent in the split message, aka if pressed = true, the actual event is this key state changes from released -> pressed and vice versa.
    Key(u8, u8, bool)
    /// 
    LedState(u8),
}
```

The slave continously scans it's matrix, if there's a key change, or a key pressed. A `SplitMessage` is sent to the master.

In master, there's a key state cache for each slave, and a separate thread running to continously receives the key states from slave and saves key states to cache.

Each slave cache in master runs in different threads, which is an infinite loop that receives all `SplitMessage` from actual slave boards. 

For master, the matrix scanning has the following steps: 

1. Scan the master's own key matrix
2. Read the all slaves' key state caches
3. Merge them to a final key states, finish matrix scanning. If the slave state is different from main key state, `changed` is true.
4. If the keyboard is running in `async_matrix` mode, each received key states triggers matrix scanning. 


### Implementation difference?

- single keyboard <-> split master:

```diff
- matrix scan of all keys
+ receive scanning result from slave
+ master - slave communication initialization & pairing
```

- single keyboard <-> split slave

```diff
- keyboard stuffs such as key processing, host communication, etc
+ send scanning result to split master
+ master - slave communication initialization & pairing
```

### How to establish the connection?

According to the connection type, some more info should be added. For example, if i2c is used, then the i2c instance of both left/right should be set in `keyboard.toml`.

If the communication is over BLE, a pairing step has to be done first, to establish the connection between right & left. In this case, the random addr of right and left should be set in `keyboard.toml`, to make sure that left & right can be paired.


### Types of split keyboard

There are several types of split keyboard that RMK should support:

1. fully wired: the left and right are connected with a cable, and the host is connected to left/right with an usb cable
2. fully wireless: the left and right are connected using BLE, and the host is connected using BLE as well
3. dongle like: there is a main device aka dongle, which connected to both left and right using BLE, and the dongle is connected to host by USB. Note that the dongle can be one of left/right side of the keyboard.
4. partial wireless: the left and right are connected with a cable, and the host is connected using BLE

The following is a simple table for those four types of split keyboard

| left/right connection | wired            | wireless       |
| --------------------- | ---------------- | -------------- |
| USB to host           | fully wired      | dongle like    |
| BLE to host           | partial wireless | fully wireless |
