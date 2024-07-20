# Technical notes

MPV socket, loading a file.

We send `loadfile`, we get back a success message immediately.

Seeks are not allowed until the `file-loaded` event. Any sooner and you'll just get ignored.

```
> { "command": ["loadfile", "https://youtu.be/HpUVmORmmpQ"] }
< {"data":{"playlist_entry_id":1},"request_id":0,"error":"success"}
< {"event":"start-file","playlist_entry_id":1}
> { "command": ["seek", 948.6638, "absolute"] }
< {"request_id":0,"error":"error running command"}
< {"event":"audio-reconfig"}
< {"event":"audio-reconfig"}
< {"event":"file-loaded"}
< {"event":"audio-reconfig"}
< {"event":"video-reconfig"}
< {"event":"video-reconfig"}
< {"event":"playback-restart"}
```