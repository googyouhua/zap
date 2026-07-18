## Root Cause

In `url_at_point()` (`grid_handler.rs:598`), the forward scan uses `UrlLocator::advance()` to detect URL boundaries. When a URL wraps across rows via terminal auto-wrap (`WRAPLINE` flag), the last cell of the wrapping row has `DEFAULT_CHAR` (`'\0'`) content. `UrlLocator` treats `'\0'` as a reset control char (`\u{00}-\u{1F}` range), terminating URL detection at the wrap boundary. Only the first row's URL portion is captured.

Same issue affects the backward scan: `'\0'` cells are not in `URL_SEPARATORS`, so the backward scan counts them and may set `starting_point` to an empty cell.

## Fix

In `url_at_point()` both scans — skip cells where `cell.c == DEFAULT_CHAR` (`'\0'`). These cells represent empty/unwritten grid space and are never part of valid URL content. Skipping them lets the cursor pass through to the next row's actual content.

No behavior change for non-wrapping URLs (no `'\0'` cells in content range). No change for hard line-breaks (UrlLocator still resets on real `'\n'`).

## Files

- `app/src/terminal/model/grid/grid_handler.rs` — add `'\0'` skip in forward and backward scan loops
