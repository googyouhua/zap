## Why

URL that wraps across multiple terminal rows (due to content exceeding terminal width) gets truncated when clicking to open — only the first row's portion is detected as the URL.

## What Changes

- Skip DEFAULT_CHAR (`'\0'`) cells in `url_at_point` forward and backward scans, so URL detection spans wrapped rows instead of stopping at the empty cell at the WRAPLINE boundary

## Capabilities

### New Capabilities
- `<none>`

### Modified Capabilities
- `<none>`

## Impact

- `app/src/terminal/model/grid/grid_handler.rs` — modify `url_at_point` scan logic
