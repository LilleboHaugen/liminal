# Source loader fixtures

Repositories read by `xtask/src/repo/sources.rs::load` in its unit tests.

| Fixture | What it contains | Tested behaviour |
|---|---|---|
| `nested` | member `outer/inner` inside member `outer`; `inner` is listed first in `members` | a `.rs` file below both members belongs to the inner one, whatever the member order |
