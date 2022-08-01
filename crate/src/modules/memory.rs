// Memory module pinout
// [0]     Enable (active high, tri-state output)
// [1]     Write (low = read, high = write)
// [2]     AddrSelect (active high)
// [3]     Clk (rising edge)
// [4-20]  Bus (16 bit)

// Full read cycles
// - Set En, Write, AddrSelect, write address to bus.
// - Set En, read data from bus.

// Full write cycles
// - Set En, Write, AddrSelect, write address to bus.
// - Set En, Write, write data to bus.

// Enable: activates the memory module. If write is high, the bus will be driven by the module,
// if write is low then the bus will be read. The bus is floating when enable is false, and clk
// is ignored.

// Write: puts the module into write mode, which can either mean writing a memory cell, or the
// address register is AddrSelect is also high. Write being low will result in either the memory
// cell or the address register being written to the bus (module will drive the bus).

// AddrSelect: allows the module to be treated as a register, where data can be clocked into or out
// of the 16-bit address select register. This register is then used for subsequent read/write ops.

// Clk: rising-edge triggered. Ignored completely is enable is false.

// Bus: 16-bit little-endian data bus with tri-state (floating) buffers. The bus will be drive only
// when En is high and Write is low.

// Timing: timing diagrams can be omitted as the module is 'magic' and has 0 tick delays. For
// example, setting En high and Write low will immediately (before the next tick) cause the bus to
// be driven with the memory cell's value. Data is also atomic, there is no way to read an erroneous
// value from a cell because of a race condition.
