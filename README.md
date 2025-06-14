# Mouse Overlay

This is a super basic bevy app that creates a moues overlay when buttons
are pressed. It works by

1. Creating a full screen bevy app
2. Setting the background of the app to transparent
3. Stopping the app from receiving hits
4. Registering global event handlers that pipe inputs to bevy even when its
   in the background.


## Limitations

It shows a full black screen on startup on Windows 11. Just click and the
background should go transparent. If it doesnt, that's probabyl because...

Transparent window support is a bit hit or miss on different platforms. For
this to work on Windows 11 with an Nvidia GPU I had to follow this:

> [https://github.com/bevyengine/bevy/issues/7544#issuecomment-2840720210]

If you right and left click at the same time the indicators overlap :shrug:

## License

MIT / Apache
