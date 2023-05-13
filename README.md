## Lotus OAST
lua-oast is a Rust client library for getting interaction logs from Interactsh servers. It has been created specifically for the Lotus Project (available at https://github.com/Bugblocker/lotus) as an additional library.

### Install
To use lua-oast, you need to have Lua (version 5.1 or higher) and the luarocks-build-rust-mlua package installed. Once you have those dependencies, you can install lua-oast with the following command:

```bash
$ luarocks install lua-oast
```


### Usage

```lua
local interactsh = require("interactsh")

local client = interactsh.client() -- This will choose a random Interactsh server with a timeout of 30 seconds.
local client = interactsh.client{server = "myserver.local", timeout = 5} -- Use this to specify a custom server and timeout.

 -- Refreshes the page to see if there are any new requests.
for k,v in paris(client:poll()) do 
    print(k,v)
end
client:host() -- Returns the OAST host.

```


### License
lua-oast is licensed under the GPL2. For more information, see the LICENSE file in this repository.
