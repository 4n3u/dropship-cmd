# dropship-cmd

`dropship-cmd` is a CLI for Samsung Dropship web flows.

It supports:

- browser login and session reuse
- local share settings via `dropship-settings.json`
- file sharing with `share-file`
- share inspection and download with `receive-info` and `receive`
- room and invitation commands

## Quick Start

The examples below assume you are running a built executable from the current directory.

### Windows

```powershell
.\dropship-cmd.exe login
.\dropship-cmd.exe settings-init
.\dropship-cmd.exe whoami
.\dropship-cmd.exe share-file --path C:\path\to\file.txt
.\dropship-cmd.exe receive --code https://g2sh.me/keyword/123456
```

### Linux

```bash
./dropship-cmd login
./dropship-cmd settings-init
./dropship-cmd whoami
./dropship-cmd share-file --path /path/to/file.txt
./dropship-cmd receive --code https://g2sh.me/keyword/123456
```

## Files Created

These files are created in the current working directory by default:

- `dropship-session.json`
- `dropship-settings.json`
- `received/<code>/`

## Notes

- authenticated commands retry once with a session refresh when the backend returns an authentication failure
- `receive` skips already-downloaded files by default and can prompt for a password when the server marks a share as secure
- advanced and low-level commands are documented separately

## Command Reference

- [COMMANDS.md](./COMMANDS.md)
