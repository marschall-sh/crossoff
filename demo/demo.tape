Output demo/demo.gif

Set Width 1400
Set Height 820
Set FontSize 16
Set Padding 20
Set PlaybackSpeed 1.0
Set TypingSpeed 70ms

Hide
Type "rm -rf /tmp/crossoff-demo && mkdir -p /tmp/crossoff-demo/data/crossoff /tmp/crossoff-demo/config && cp demo/seed.json /tmp/crossoff-demo/data/crossoff/data.json"
Enter
Sleep 300ms
Type "XDG_DATA_HOME=/tmp/crossoff-demo/data XDG_CONFIG_HOME=/tmp/crossoff-demo/config ./target/release/crossoff"
Enter
Sleep 1600ms
Show

# Highlight the board and priority/task metadata chips
Sleep 1200ms

# Start a timer on the priority task
Type "t"
Sleep 1200ms

# Move a task into In Progress
Down
Sleep 500ms
Type "L"
Sleep 1200ms

# Open the detail panel, then edit from details
Enter
Sleep 1200ms
Type "e"
Sleep 1000ms
Down
Sleep 400ms
Down
Sleep 400ms
Escape
Sleep 700ms

# Search across tasks
Type "/"
Sleep 400ms
Type "docs"
Sleep 1000ms
Escape
Sleep 600ms

Type "q"
Sleep 500ms
