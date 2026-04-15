Output demo/demo.gif

Set Width 900
Set Height 500
Set FontSize 13
Set Padding 0
Set PlaybackSpeed 1.0
Set TypingSpeed 80ms

# Hidden setup: seed isolated demo data (binary must already be built)
Hide
Type "mkdir -p /tmp/crossoff-demo/crossoff && cp demo/seed.json /tmp/crossoff-demo/crossoff/data.json"
Enter
Sleep 500ms
Show

# Launch crossoff with demo data
Type "XDG_DATA_HOME=/tmp/crossoff-demo ./target/release/crossoff"
Enter
Sleep 2000ms

# Switch to Tasks pane
Tab
Sleep 700ms

# Navigate through tasks — detail pane updates live
Down
Sleep 500ms
Down
Sleep 500ms
Up
Sleep 400ms

# Open Detail pane — shows description + checklist
Tab
Sleep 700ms
Down
Sleep 400ms
Down
Sleep 500ms

# Back to Tasks
Tab
Sleep 600ms

# Create a new task
Type "n"
Sleep 500ms
Type "Ship v0.1.0 release"
Sleep 300ms
Ctrl+S
Sleep 1000ms

# Navigate to the overdue task and mark it done
Up
Sleep 400ms
Enter
Sleep 1400ms

# Fuzzy search
Type "/"
Sleep 400ms
Type "doc"
Sleep 1200ms
Escape
Sleep 700ms

# Quit
Type "q"
Sleep 500ms
