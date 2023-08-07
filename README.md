# expert-spoon

Tool to link global shortcuts to commands

# Usage

1. Create configuration: (~/.expert-spoon.yaml`)

```
version: 1

hotkeys:
  - key: 'ALT+1'
    name: "Open terminal"
    action:
      type: open
      command: open
      args:
        - '/Applications/Alacritty.app'
  - key: 'ALT+2'
    name: "Open browser"
    action:
      type: open
      command: open
      args:
        - '/Applications/Google Chrome.app'
```

2. Buind and run: `cargo run`

# Disclaimer

This tool is for my personal use, it work only on MacOs
