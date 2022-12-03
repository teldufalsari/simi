# Simi

This project aims to implement a simple yet reliable tool of secure communication.
It allows its users to send peer-to-peer messages containing either plain text with no encryption, or
png images with encrypted text embedded. To those unfamiliar with the protocol, captured messages
will not appear as regular unsuspicipus image sharing. Even those familiar with LSB encoding may
not found anything thanks to the use of encryption.

## Building
Nothing is to be built yet, but we are using Cargo, so I suppose just make sure you have Rust
toolchain installed, and then type
`cargo build`

## Usage
simi runs in interactive mode only. Type `simi` in your terminal to run it. Below is an example of a typical session:

```sh
sh-5.1$ simi
<simi>: Welcome. Type help to display available commands
[you]: add Saul 192.168.0.12:1337
<simi>: contact 'Saul' with address '192.168.0.12:1337' has been saved
[you]: remove Saul
<simi>: contact 'Saul' has been removed
[you]: add Saul 192.168.0.14:1337
<simi>: contact 'Saul' with address '192.168.0.14:1337' has been saved
[you]: list
<simi>: Lena = 192.168.0.12:1337
<simi>: Saul = 192.168.0.14:1337
[you]: dial Lena
<simi>: Lena is offline. Wait until they connect to you or return
<simi>: Lena is now online.
[you]: Hey, wanna check out this new meme about Rust?
[Lena]: Prorgamming language? Sure, shoot it out
[you]: --secret file=~/Pictures/rust-meme.png
<simi>: Using auto session key, embedding secret into '~/Pictures/rust-meme.png'
<simi>: Enter secret message:
[you, whispering]: I'm expelled
<simi>: secret successfully sent
[Lena]: omfg lol
[Lena, whispering]: OMG is that for real?
[you]: Yeah, I knew that you'd like that
[Lena]: Sorry, I have to go now. meet you later this evening
<simi>: Lena is offline. Wait until they connect to you or return
[you]: --exit
<simi>: back to menu
[you]: exit
sh-5.1$
```

### Commands in menu:

- `list`: this list all contacts saved in the file `~/.simi/conf.toml`. Contacts can be added either by editing the file `conf.toml` manually or via `add` command
- `add <alias> <ip:port>`: this adds record `alias=ip:port` to the contact list. Note that all changes to the contact list are saved to `conf.ini` only after exiting normally
- `remove <alias>`: this removes record specified by alias from the contact list
- `dial <alias>` or dial `<ip:port>`: switches to the dialog the contact
- `exit`: this exits the application. If any changes to contact list are made, write them on the disk

### Command in the dialog
You should wait until your peer becomes online to start messaging. To send a plain text message, just type it in the terminal. It cannot start with `--`, because it will be interpreted as a command then and you will likely get an error.
Commands in the dialog should be escaped with `--`. There are only two available commands:

- `--secret [--path=/path/to/file.png]`: initiate a secret transmission. `--path` is an optional argument; if it's present, the application will check whethet it points to a suitable png file and report back if it can't be used to carry the message. If not stated, an image from the folder specified in config (see config section for details) is chosen. If everything is okay, the app prints the name of the chosen file and prompts you to enter you secret message. Press `enter` to send it. Recieved and sent secret messages are marked with the word "whispering" in the command line prompt.
- `--exit`: this exits the dialog and returns to the menu

## Configuration file

```toml
# Port to listen to
# Other users should specify this port number
# when dialing
port=1337

# Path the directory with .png images
# If --secret command is invoked without --path argument,
# Images are picked from here
assets="~/.simi/assets"

# If true, imamges will be deleted from the directory
# after use.
# Images specified by --path are never deleted
delete_images=false

# If true, images will be picked randomly from the directory
# If false, the first image in alphabetical order is picked
# False is recommended only with delete_images=true
pick_randomly=true

[Contacts]
Lena="192.168.0.12:1337"
Saul="192.168.0.14:1337"

```
