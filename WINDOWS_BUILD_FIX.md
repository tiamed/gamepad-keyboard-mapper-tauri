# Windows Build Troubleshooting

## Fix for "link.exe extra operand" error

This error occurs due to Windows path length limitations (260 character MAX_PATH limit).

### Solution 1: Enable Long Path Support in Windows

1. Open Group Policy Editor (gpedit.msc)
2. Navigate to: Computer Configuration > Administrative Templates > System > Filesystem
3. Enable "Enable Win32 long paths"
4. Reboot

Or via registry (as admin):
```powershell
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

### Solution 2: Clean Build Cache

Delete the corrupted target directory:
```bash
# In PowerShell or Command Prompt
cd src-tauri
cargo clean
```

Then rebuild:
```bash
npm run tauri dev
```

### Solution 3: Use Shorter Project Path

Move project to a shorter path like `C:\project\gamepad-mapper` instead of deep nested paths.

### Solution 4: Enable Cargo Long Paths

Set environment variable:
```powershell
$env:CARGO_NET_GIT_FETCH_WITH_CLI = "true"
```

Or permanently in System Properties > Environment Variables:
- Variable: `CARGO_NET_GIT_FETCH_WITH_CLI`
- Value: `true`

### Solution 5: Use GNU Linker Instead

If MSVC linker continues to fail, install LLVM and use lld:

1. Install LLVM from https://releases.llvm.org/
2. Set environment variable:
```powershell
$env:RUSTFLAGS = "-C linker=lld"
```

Then rebuild.

## Recommended Fix Order

1. **Try Solution 2 first** (cargo clean) - quickest
2. If still failing, **Solution 3** (move to shorter path)
3. Then **Solution 1** (enable long paths in Windows)

## Verification

After applying fixes, verify with:
```bash
cd src-tauri
cargo check
```

If cargo check passes, the Tauri build should work:
```bash
npm run tauri dev
```
