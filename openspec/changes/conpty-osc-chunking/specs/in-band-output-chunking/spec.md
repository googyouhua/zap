## ADDED Requirements

### Requirement: Large generator output SHALL be split into chunks

When in-band generator command output exceeds ConPTY-safe size, the shell SHALL split the hex-encoded payload into multiple OSC sequences. The Rust receiver SHALL accumulate the chunks and reconstruct the full payload before decoding.

#### Scenario: Small output (≤3KB hex) sent as single OSC
- **WHEN** `_warp_execute_command` produces hex output ≤ 3000 bytes
- **THEN** the shell SHALL send a single OSC sequence ending with `\e]9277;B\a`
- **THEN** the Rust receiver SHALL decode the payload immediately

#### Scenario: Large output (>3KB hex) split into chunks
- **WHEN** `_warp_execute_command` produces hex output > 3000 bytes
- **THEN** the shell SHALL split the hex into chunks of ≤ 3000 bytes each
- **THEN** non-final chunks SHALL end with `\e]9277;C\a`
- **THEN** the final chunk SHALL end with `\e]9277;B\a`
- **THEN** the Rust receiver SHALL accumulate hex data from `C` chunks
- **THEN** on receiving `B`, the Rust receiver SHALL concatenate all accumulated hex and decode as a single payload

### Requirement: Interleaved OSC sequences SHALL NOT corrupt chunk accumulation

The chunk accumulation SHALL be scoped to a single generator command. Receiving a new `\e]9277;A\a` before the current sequence terminates SHALL reset the accumulator.

#### Scenario: New start resets accumulator
- **WHEN** `\e]9277;A\a` is received while `IsReceivingInBandCommandOutput::Yes` with `accumulated_hex` is already set
- **THEN** the `accumulated_hex` SHALL be discarded
- **THEN** a new `InBandCommandOutputReceiver` SHALL be created with empty `accumulated_hex`

### Requirement: Backward compatibility with `C` marker

Old Rust clients that do not handle `\e]9277;C\a` SHALL NOT crash or corrupt data. The unrecognized marker SHALL fall through to a warning log.

#### Scenario: Old client receives unknown marker
- **WHEN** a `\e]9277;C\a` OSC is received by code that does not handle the `C` byte
- **THEN** the OSC dispatch SHALL log a warning and ignore the marker
