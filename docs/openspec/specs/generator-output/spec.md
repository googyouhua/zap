# generator-output Specification

## Purpose
TBD - created by archiving change fix-conpty-hex-leak. Update Purpose after archive.
## Requirements
### Requirement: OSC 9277 protocol uses inline hex format
**SHALL** send hex payload entirely within the OSC boundary using `\e]9277;A;<hex>\a\e]9277;B\a`

#### Scenario: Legacy format causes visible garbage on ConPTY
Old format `\e]9277;A\a<len>;<hex>\e]9277;B\a` lets ConPTY consume the first `\a` as OSC terminator, leaving `<len>;<hex>` as visible text. The new inline format keeps all payload inside the OSC boundary.

#### Scenario: New format is backwards compatible
Rust side retains the old `start_in_band_command_output` path. Shell scripts emit only the new format. Old sessions in progress at upgrade time continue to be decoded correctly.

### Requirement: Content length prefix removed
**SHALL** no longer include `<content_length>;` prefix before the hex payload

#### Scenario: Content length was only needed for chunked protocol
With the inline format, the OSC boundary itself delimits the payload, so length prefix is redundant.

### Requirement: Payload decode format unchanged
**SHALL** continue to decode hex payload as `<command_id>;<output>;<exit_code>`

#### Scenario: Rust side hex decode produces same byte format
The post-hex-decode format is unchanged from the old protocol. Only the transport format (with/without inline hex) differs.

