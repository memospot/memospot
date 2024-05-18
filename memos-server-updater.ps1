
# Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/memospot/memospot/main/memos-server-updater.ps1'))


Write-Host @"
         __  __ _____ __  __  ___  ____  ____   ___ _____
        |  \/  | ____|  \/  |/ _ \/ ___||  _ \ / _ \_   _|
        | |\/| |  _| | |\/| | | | \___ \| |_) | | | || |
        | |  | | |___| |  | | |_| |___) |  __/| |_| || |
        |_|  |_|_____|_|  |_|\___/|____/|_|    \___/ |_|
                                                 _       _
 ___  ___ _ ____   _____ _ __    _   _ _ __   __| | __ _| |_ ___ _ __
/ __|/ _ \ '__\ \ / / _ \ '__|  | | | | '_ \ / _` |/ _` | __/ _ \ '__|
\__ \  __/ |   \ V /  __/ |     | |_| | |_) | (_| | (_| | ||  __/ |
|___/\___|_|    \_/ \___|_|      \__,_| .__/ \__,_|\__,_|\__\___|_|
                                      |_|
"@ -f DarkCyan

$GitHubRepo = "memospot/memos-builds"

$DataPath = [IO.Path]::Combine($Env:LocalAppData, "memospot")

$MemospotPath = {
  $searchPaths = @(
    [IO.Path]::Combine($Env:ProgramFiles, "Memospot"),
    $DataPath
  )
  foreach ($dir in $searchPaths) {
    $memosBin = [IO.Path]::Combine($dir, "memos.exe")
    if ([IO.File]::Exists($memosBin)) {
      return (Resolve-Path $dir).Path
    }
  }
}.Invoke()

if ([String]::IsNullOrEmpty($MemospotPath)) {
  Write-Host "Memospot not found. Execution halted." -f Red
  Exit 1
}
else {
  Write-Host "Memospot found at: ``$MemospotPath``" -f Green
}

if ($MemospotPath.StartsWith($Env:ProgramFiles)) {
  $LastDebugPreference = $DebugPreference
  $DebugPreference = 'SilentlyContinue'
  if (!([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host @"

    Memospot was installed system-wide with the MSI installer.

    Administrator access is required to update files on the Program Files folder.

    Please launch Powershell or Windows Terminal as Administrator and run this script again.

"@ -f Yellow
    Write-Host "`n-> Press any key to exit <-`n" -f Cyan
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    Exit 1
  }
  $DebugPreference = $LastDebugPreference
  Write-Host "This script is running with administrator access." -f Green
}

if ($PSVersionTable.PSVersion.Major -lt 6) {
  $global:IsWindows = ([Environment]::OSVersion.Platform -eq "Win32NT")
}
if (-not $IsWindows) {
  Write-Host "This script only supports Windows." -f Red
  Exit 1
}

$HostArch = {
  $archMap = @{
    "x86_64" = "amd64", "i386_64"
  }

  $platform = {
    if ($IsWindows) {
      if (![String]::IsNullOrEmpty($Env:PROCESSOR_ARCHITEW6432)) {
        return $Env:PROCESSOR_ARCHITEW6432
      }
      return $Env:PROCESSOR_ARCHITECTURE
    }
    return $(uname -p)
  }.Invoke().ToLower()

  foreach ($k in $archMap.Keys) {
    foreach ($v in $archMap[$k]) {
      if ($platform.Equals($v)) {
        return $k
      }
    }
  }
  return $platform
}.Invoke()

if ([String]::IsNullOrEmpty($HostArch)) {
  Write-Host "Unsupported CPU architecture" -f Red
  Exit 1
}

$HostOS = {
  $osMap = @{
    "linux" = "mingw64_nt", "mingw32_nt"
  }

  $platform = {
    if ($IsWindows) {
      return "windows"
    }
    return $(uname -s)
  }.Invoke().ToLower()

  foreach ($k in $osMap.Keys) {
    foreach ($v in $osMap[$k]) {
      if ($platform.StartsWith($v)) {
        return $k
      }
    }
  }
  return $platform.ToLower()
}.Invoke()

if ([String]::IsNullOrEmpty($HostOS)) {
  Write-Host "Unsupported OS: $([Environment]::OSVersion.Platform)" -f Red
  Exit 1
}

$BackupsPath = [IO.Path]::Combine($DataPath, "server-updater-backups")
if (-not [IO.Directory]::Exists($BackupsPath)) {
  Write-Host "Creating backups directory: $BackupsPath" -f Cyan
  New-Item -Path $BackupsPath -ItemType Directory -Force -ErrorAction Stop
  if ($null -eq $?) {
    Write-Host "Failed to create directory. Make sure you have write permissions to: $DataPath" -f Red
    Exit 1
  }
}

##
$ProgressPreference = 'SilentlyContinue'
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072;
$releases = "https://api.github.com/repos/$GitHubRepo/releases/latest"
$latest = (Invoke-WebRequest -Uri $releases -UseBasicParsing -ErrorAction Stop | ConvertFrom-Json)[0]
$tagName = $latest.tag_name
$latestAssets = $latest.assets
$sha256Sums = $latestAssets | Where-Object { $_.name.ToLower().EndsWith("sha256sums.txt") } | Select-Object -First 1 | Select-Object -ExpandProperty browser_download_url
$latestZip = $latestAssets | Where-Object { $_.name.ToLower().EndsWith("$($HostOS)-$($HostArch).zip") -or $_.name.ToLower().EndsWith("$($HostOS)-$($HostArch)_v1.zip") }
$matchedAsset = @($latestZip).Where({ ![String]::IsNullOrEmpty($_) }, "First")
$downloadUrl = $matchedAsset.browser_download_url
if ([String]::IsNullOrEmpty($downloadUrl)) {
  Write-Host "Unable to find a valid release URL!" -f Magenta
  Write-Host "Unable to match OS: $HostOS, CPU: $HostArch" -f Magenta
  Write-Host "Please open an issue at: https://github.com/$GitHubRepo/issues" -f Magenta
  Write-Host "Remember to provide your OS version, CPU details and Powershell version." -f Magenta
  Exit 1
}

Write-Host "Latest Memos release found: $tagName" -f Green
Write-Host "`nRelease URL: $downloadUrl"
Write-Host "Release SHA256SUMS: $($sha256Sums)"

Write-Host "`nNote that this script will not check whether you actually need to update the Memos server.
The latest available version is always downloaded." -f Yellow

Write-Host "`n-> Press any key to continue <-`n" -f Cyan
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

$ZippedRelease = [IO.Path]::Combine($MemospotPath, $matchedAsset.name)
Remove-Item -Path $ZippedRelease -Force -ErrorAction SilentlyContinue

Write-Host "Downloading release ``$tagName``... This can take a while." -f Green

Invoke-WebRequest -OutFile $ZippedRelease -Uri $downloadUrl -UseBasicParsing -ErrorAction Stop
if ($null -eq $?) {
  Write-Host "Failed to download file at: $downloadUrl" -f Red
  Exit 1
}

if (![IO.File]::Exists($ZippedRelease)) {
  Write-Host "Failed to download file to: $ZippedRelease" -f Red
  Exit 1
}

if ($IsWindows) {
  Write-Host "Unblocking file: $ZippedRelease"
  Unblock-File -Path $ZippedRelease -ErrorAction SilentlyContinue
}

if (-not [IO.File]::Exists($ZippedRelease)) {
  Write-Host "Unable to find downloaded file: $ZippedRelease" -f Red
  Write-Host @"
  Make sure your antivirus is not blocking the download.

  If you are using Windows Defender and it shows a threat warning, rapidly click on it, expand the most recent threat and click on "Allow on device".

  You have little time to do this before Defender takes action on its own and makes your life harder.

  After allowing the file, try to run this script again.
"@ -f Yellow
  Exit 1
}

if ([String]::IsNullOrEmpty($sha256Sums)) {
  Write-Host "Unable to find SHA256SUMS!" -f Red
  Exit 1
}

$hashes = Invoke-WebRequest -Uri $sha256Sums -UseBasicParsing -ErrorAction Stop
if ($null -eq $?) {
  Write-Host "Failed to download SHA256SUMS file at: $sha256Sums" -f Red
  Exit 1
}

$hashes = [System.Text.Encoding]::UTF8.GetString($hashes.Content)

$hash = $null
foreach ($line in $hashes.Split("`n")) {
  if ([String]::IsNullOrEmpty($line)) {
    continue
  }
  $parts = $line.Split("  ")
  if ($parts[-1].ToLower().Equals($matchedAsset.name.ToLower())) {
    $hash = $parts[0]
    break
  }
}

if ([String]::IsNullOrEmpty($hash)) {
  Write-Host "Unable to find hash for file: $ZippedRelease" -f Red
  Exit 1
}
Write-Host "Expected hash: $hash"

# calculate sha256 hash for $ZippedRelease
$hasher = [Security.Cryptography.SHA256]::Create()
$downloadedZipHash = [System.BitConverter]::ToString($hasher.ComputeHash([IO.File]::ReadAllBytes($ZippedRelease)))
$downloadedZipHash = $downloadedZipHash.Replace("-", "").ToLower()

if ($hash.ToLower().Equals($downloadedZipHash)) {
  Write-Host "Hashes match!" -f Green
}
else {
  Write-Host "Hashes do not match!" -f Red
  Write-Host "Expected: $($hash[0])" -f Red
  Write-Host "Actual:   $downloadedZipHash" -f Red
  Write-Host "Try to run this script again." -f Yellow
  Exit 1
}

$dateString = $(Get-Date -Format "yyyyMMdd_HHmmss")

$databasePath = [IO.Path]::Combine($DataPath, "memos_prod.db")
$databasePathWAL = [IO.Path]::Combine($DataPath, "memos_prod.db-wal")
$databasePathSHM = [IO.Path]::Combine($DataPath, "memos_prod.db-shm")
if ([IO.File]::Exists($databasePath)) {
  $databaseBackup = [IO.Path]::Combine($BackupsPath, "db_${dateString}_before_${tagName}.zip")
  Write-Host "Backing up current database"

  $fileList = [System.Collections.ArrayList]@($databasePath)
  if ([IO.File]::Exists($databasePathWAL)) {
    $fileList += $databasePathWAL
  }
  if ([IO.File]::Exists($databasePathSHM)) {
    $fileList += $databasePathSHM
  }

  Compress-Archive -Path $fileList -DestinationPath "$databaseBackup" -Force -ErrorAction Stop
  if ($null -eq $?) {
    Write-Host "Failed to backup file. Make sure Memos is stopped and that you have write permissions to: $databaseBackup" -f Red
    Exit 1
  }
}

$distPath = [IO.Path]::Combine($MemospotPath, "dist");
$memosBin = [IO.Path]::Combine($MemospotPath, "memos.exe")
$memosBinBackup = [IO.Path]::Combine($BackupsPath, "memos_${dateString}_before_${tagName}.zip");
if ([IO.File]::Exists($memosBin)) {
  Write-Host "Backing up current memos.exe"
  if ([IO.Directory]::Exists($distPath)) {
    $fileList = [System.Collections.ArrayList]@($memosBin, $distPath)
  }
  else {
    $fileList = [System.Collections.ArrayList]@($memosBin)
  }

  Compress-Archive -Path $fileList -DestinationPath "$memosBinBackup" -Force -ErrorAction Stop
  if ($null -eq $?) {
    Write-Host "Failed to backup file. Make sure Memos is stopped and that you have write permissions to: $BackupsPath" -f Red
    Exit 1
  }
}

Remove-Item -Path $distPath -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "Extracting Memos to: $MemospotPath"
Expand-Archive -Path $ZippedRelease -DestinationPath $MemospotPath -Force
if ($null -eq $?) {
  Write-Host "Failed to extract file." -f Red
  if ([IO.File]::Exists($memosBinBackup)) {
    Write-Host "Restoring Memos backup" -f Cyan
    Remove-Item -Path $distPath -Recurse -Force -ErrorAction SilentlyContinue
    Expand-Archive -Path $memosBinBackup -DestinationPath $MemospotPath -Force
  }
  Exit 1
}
Remove-Item -Path $ZippedRelease -Force -ErrorAction SilentlyContinue

# If MSI installer was used, copy dist folder to LocalAppData
# This should increase compatibility with multiple versions of Memospot
if ($MemospotPath.StartsWith($Env:ProgramFiles)) {
  $distPathLocal = [IO.Path]::Combine($DataPath, "dist");
  if ([IO.Directory]::Exists($distPathLocal)) {
    Write-Host "Removing old dist folder: $distPathLocal" -f Cyan
    Remove-Item -Path $distPathLocal -Recurse -Force -ErrorAction SilentlyContinue
  }
  if ([IO.Directory]::Exists($distPath)) {
    Write-Host "Copying dist folder to: $distPathLocal" -f Cyan
    Copy-Item -Path $distPath -Destination $distPathLocal -Recurse -Force -ErrorAction Stop
  }
}

$readmeFile = [IO.Path]::Combine($MemospotPath, "README.md")
Remove-Item -Path $readmeFile -Force -ErrorAction SilentlyContinue

$licenseFile = [IO.Path]::Combine($MemospotPath, "LICENSE")
Remove-Item -Path $licenseFile -Force -ErrorAction SilentlyContinue

Write-Host "Unblocking file: $memosBin" -f Cyan
Unblock-File -Path $memosBin -ErrorAction SilentlyContinue

Write-Host "`nMemos server successfully updated to $tagName" -f Green

Write-Host "`n-> Press any key to exit <-`n" -f Cyan
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
