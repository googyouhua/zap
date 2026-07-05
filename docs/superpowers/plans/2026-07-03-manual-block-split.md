---
change: manual-block-split
design-doc: docs/superpowers/specs/2026-07-03-manual-block-split-design.md
base-ref: 5c145ccd27925c6710d6f6194a0f6817866dbc52
---

# Implementation Plan: Manual Block Split

## Task 1: Add `SplitBlock` variant to `TerminalAction` enum

**Files:** `app/src/terminal/view/action.rs`

### Step 1.1 — Add enum variant (2 min)

After `OpenCLIAgentRichInput` at line 400 (last variant), add:

```rust
    /// Split the active block at the cursor position (manual override when
    /// shell integration fails to detect the boundary).
    SplitBlock,
```

### Step 1.2 — Add Debug impl entry (2 min)

In the manual `fmt::Debug for TerminalAction` impl (line 404), before the closing `}` of the match (before line 649), add:

```rust
            SplitBlock => f.write_str("SplitBlock"),
```

### Step 1.3 — Add accessibility content match (2 min)

In `action_accessibility_contents` in `app/src/terminal/view.rs`, in the large match arm starting with `Scroll { .. }` at line 23413 (which maps to `ActionAccessibilityContent::from_debug()` or `Empty`), add `SplitBlock` to the appropriate group:

```rust
            SplitBlock => Empty,
```

Add it to the `Scroll { .. } | AltScroll { .. } | ...` group at line 23413 (or simply as a separate arm returning `ActionAccessibilityContent::from_debug()`—check which pattern fits with surrounding arms at that location).

### Verify 1.4

```bash
cargo check -p warp 2>&1 | head -30
```

Expect: no errors (variant is defined but not yet matched in `handle_action`—that's okay because the match is exhaustive at compile time... actually no, it WILL fail because `handle_action`'s match in view.rs must be exhaustive). So step 1.3 must include adding `SplitBlock` to the `handle_action` match even if it's just a `todo!()` or no-op placeholder first. We'll add a real handler in Task 3.

---

## Task 2: Implement `BlockList::split_active_block_at_cursor()` in blocks.rs

**Files:** `app/src/terminal/model/blocks.rs`, `app/src/terminal/model/blocks_test.rs`

### Step 2.1 — Write unit test first (5 min)

In `app/src/terminal/model/blocks_test.rs`, add a test function:

```rust
#[test]
fn test_split_active_block_at_cursor() {
    let (_proxy, channel_event_proxy) = ChannelEventListener::new(
        Box::new(warp_terminal::ansi::TerminalModel::default()),
    );
    let mut block_list = new_bootstrapped_block_list(None, None, channel_event_proxy);

    // Insert a block with command "echo hello" and multi-line output
    let block_idx = insert_block(&mut block_list, "echo hello", "line1\nline2\nline3");

    // Verify we have the block
    assert_eq!(block_list.blocks().len(), 4); // 3 bootstrap + 1 new = 4
    let block = block_list.block_at(block_idx).unwrap();
    let output_grid = block.output_grid();
    let total_output_rows = output_grid.len();
    assert!(total_output_rows > 0, "output grid should have content");

    // Split at cursor line = 1 (second line of output, 0-indexed)
    // This should split the output grid at row 1 (NonZeroUsize::new(1))
    // The top gets "line1\n", the bottom gets "line2\nline3"
    let result = block_list.split_active_block_at_cursor(1);
    assert!(result.is_ok(), "split should succeed: {:?}", result.err());

    // We should now have 5 blocks (original block finished, new block created)
    assert_eq!(block_list.blocks().len(), 5);

    // The original block's output should now only contain "line1"
    let orig_block = block_list.block_at(block_idx).unwrap();
    let orig_output = orig_block.output_grid().contents_to_string(false, None);
    assert_eq!(orig_output.trim(), "line1", "original block output should be truncated");

    // The new block should have the remaining output as its command
    let new_block_idx = BlockIndex(block_idx.0 + 1);
    let new_block = block_list.block_at(new_block_idx).unwrap();
    // New block's command grid should contain "line2\nline3"
    let new_block_output = new_block.output_grid().contents_to_string(false, None);
    assert_eq!(new_block_output.trim(), "line2\nline3",
        "new block output should contain remaining lines");
}
```

Then run to see it fail:

```bash
cargo nextest run -p warp test_split_active_block_at_cursor --no-fail-fast 2>&1 | tail -20
```

### Step 2.2 — Implement `split_active_block_at_cursor()` (8 min)

In `app/src/terminal/model/blocks.rs`, after `blocks_mut()` (around line 1875), add:

```rust
    /// Manually split the active block's output grid at the given cursor line.
    ///
    /// This is a fallback when shell integration fails to detect the command
    /// boundary. The line N (and everything after it) becomes the command text
    /// of a new block inserted after the current one.
    pub fn split_active_block_at_cursor(
        &mut self,
        cursor_line: usize,
    ) -> Result<(), &'static str> {
        let active_idx = self.active_block_index();
        let block = self.block_at(active_idx).ok_or("no active block")?;

        if !block.output_grid().finished() {
            return Err("output grid is not yet finished");
        }

        let output_len = block.output_grid().len();
        if cursor_line >= output_len {
            return Err("cursor line is outside output grid bounds");
        }

        // The split row must be NonZeroUsize — row 0 means "split before first
        // line", which is invalid for this operation. Cursor line is 0-indexed;
        // BlockGrid::split takes a 0-indexed row. But we need NonZeroUsize for
        // the split to make sense: split at cursor_line + 1 gives the top grid
        // rows [0, cursor_line) and bottom grid rows [cursor_line, end).
        let split_row = NonZeroUsize::new(cursor_line + 1)
            .ok_or("cursor_line + 1 overflowed")?;

        // Extract the text of the bottom (split-off) portion as the new command
        let command_text = block.output_grid().contents_to_string(false, None);
        let lines: Vec<&str> = command_text.split('\n').collect();
        let remaining_text = lines[cursor_line..].join("\n");
        if remaining_text.trim().is_empty() {
            return Err("split-off command text is empty");
        }

        // Borrow `block` again mutably to get/set its output_grid.
        // We need to split the output grid.
        let (top_grid, bottom_grid_opt) = {
            let block = self.block_at(active_idx).ok_or("no active block")?;
            block.output_grid().split(split_row)
        };

        // Replace the active block's output grid with the top portion
        {
            let block = self.block_mut_at(active_idx).ok_or("no active block")?;
            block.set_output_grid(top_grid);
            block.finish_output_grid();
        }

        // If there's a bottom grid, create a new block with it
        if let Some(bottom_grid) = bottom_grid_opt {
            let new_block_id = BlockId::new_random();
            self.create_new_block(
                new_block_id,
                self.bootstrap_stage,
                None, // precmd_value
                None, // restored_block_was_local
            );

            // Set the new block's output to the bottom grid content
            let new_idx = BlockIndex(active_idx.0 + 1);
            if let Some(new_block) = self.block_mut_at(new_idx) {
                new_block.set_command_from_grid(&bottom_grid);
                new_block.set_output_grid(bottom_grid);
            }
        }

        self.update_block_height_indices(
            BlockHeightUpdate::Insertion(TotalIndex(active_idx.0)),
            false,
        );

        Ok(())
    }
```

**Note:** This implementation uses helper methods on `Block` that may not exist yet (`block_mut_at`, `set_output_grid`, `set_command_from_grid`, `finish_output_grid`). We'll need to add or adjust based on what's available. The next step verifies and adjusts.

### Step 2.3 — Add/verify needed `Block` helper methods (5 min)

Check if these exist on `Block` (search `app/src/terminal/model/block.rs`):

- `output_grid(&self)` → already exists (field accessor)
- `set_output_grid(...)` → may not exist, need to add
- `finish_output_grid()` → calls `self.output_grid.finish()`
- `set_command_from_grid(&BlockGrid)` → sets the header command grid

Add missing helpers in `block.rs`:

```rust
    pub fn set_output_grid(&mut self, grid: BlockGrid) {
        self.output_grid = grid;
    }

    pub fn finish_output_grid(&mut self) {
        self.output_grid.finish();
    }
```

Also add `block_mut_at` on `BlockList` if it doesn't exist (it mirrors `block_at`):

```rust
    pub fn block_mut_at(&mut self, index: BlockIndex) -> Option<&mut Block> {
        self.blocks.get_mut(index.0)
    }
```

### Step 2.4 — Run test (1 min)

```bash
cargo nextest run -p warp test_split_active_block_at_cursor --no-fail-fast 2>&1 | tail -20
```

Fix any compilation or logic errors iteratively.

---

## Task 3: Wire action handler in `TerminalView::handle_action`

**Files:** `app/src/terminal/view.rs`

### Step 3.1 — Add match arm (3 min)

In `handle_action` (line 23480), after `SplitUp` (around line 23708), add:

```rust
            SplitBlock => {
                let mut model = self.model.lock();
                let block_list = model.block_list_mut();
                let active_idx = block_list.active_block_index();
                let block = block_list.block_at(active_idx);
                if let Some(block) = block {
                    // Get the cursor line from the output grid.
                    // The cursor point's row gives us the split position.
                    if let Some(cursor_point) = block.output_grid().cursor_display_point() {
                        let cursor_line = match cursor_point {
                            CursorDisplayPoint::Visible(p) => p.row,
                            CursorDisplayPoint::HiddenCache(p) => p.row,
                        };
                        if let Err(e) = block_list.split_active_block_at_cursor(cursor_line) {
                            log::warn!("SplitBlock failed: {e}");
                        }
                    }
                }
                drop(model);
                ctx.notify();
            }
```

### Step 3.2 — Add imports if needed (1 min)

Add to the top block of `view.rs` if not already present:

```rust
use crate::terminal::model::blockgrid::CursorDisplayPoint;
```

### Verify 3.3

```bash
cargo check -p warp 2>&1 | head -30
```

Expect: clean compilation.

---

## Task 4: Register keybinding `ctrl-shift-\` → `SplitBlock`

**Files:** `app/src/terminal/view/init.rs`

### Step 4.1 — Add FixedBinding (3 min)

In `pub fn init(app: &mut AppContext)` at around line 206 (before the `if cfg!(target_os = "macos")` block), add:

```rust
    // Manual block split: Ctrl+Shift+\ splits the active block at the cursor.
    app.register_fixed_bindings([FixedBinding::new(
        "ctrl-shift-\\",
        TerminalAction::SplitBlock,
        id!("Terminal") & !id!("IMEOpen"),
    )]);
```

> **Rationale for `FixedBinding`:** This is not user-customizable — it's a fixed feature keybinding, similar to `SplitRight`/`SplitDown` on lines 258-270. The context predicate `id!("Terminal") & !id!("IMEOpen")` ensures it only fires when the terminal is focused and no IME is open.

### Verify 4.2

```bash
cargo check -p warp 2>&1 | head -10
```

Expect: clean.

---

## Task 5: Verify complete feature

### Step 5.1 — Full cargo check (2 min)

```bash
cargo check 2>&1
```

### Step 5.2 — Run unit tests (3 min)

```bash
cargo nextest run -p warp --no-fail-fast 2>&1 | tail -30
```

Ensure existing tests still pass. Fix any regressions.

---

## Files Changed Summary

| File | Change |
|------|--------|
| `app/src/terminal/view/action.rs` | +1 variant to `TerminalAction` enum, +1 arm to `Debug` impl |
| `app/src/terminal/model/blocks.rs` | +1 method `split_active_block_at_cursor()`, +1 helper `block_mut_at()` |
| `app/src/terminal/model/block.rs` | +2 helpers `set_output_grid()` and `finish_output_grid()` |
| `app/src/terminal/model/blocks_test.rs` | +1 unit test `test_split_active_block_at_cursor` |
| `app/src/terminal/view.rs` | +1 match arm in `handle_action`, +import, +accessibility entry |
| `app/src/terminal/view/init.rs` | +1 `FixedBinding` for `ctrl-shift-\` |
