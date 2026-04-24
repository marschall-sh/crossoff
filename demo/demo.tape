Output demo/demo.gif

Set Width 1200
Set Height 760
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
Sleep 1800ms
Show

# Start a timer on the pinned task
Tab
Sleep 700ms
Type "t"
Sleep 1200ms

# Open details on another task
Down
Sleep 500ms
Tab
Sleep 900ms
Down
Sleep 600ms
Up
Sleep 500ms

# Move the selected task to Work
Tab
Sleep 700ms
Type "m"
Sleep 700ms
Down
Sleep 400ms
Enter
Sleep 1200ms

# Search across tasks
Type "/"
Sleep 400ms
Type "release"
Sleep 1000ms
Escape
Sleep 600ms

Type "q"
Sleep 500ms
