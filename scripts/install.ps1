[CmdletBinding()]
param(
  [Alias("v")]
  [string]$Version,

  [Alias("t")]
  [string]$InstallDir = "$HOME\.onx\bin"
)

$ErrorActionPreference = "Stop"

$Repo = "StevanFreeborn/onx"
$GithubUrl = "https://github.com/$Repo"
$BinaryName = "onx.exe"

$Arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") {
  "aarch64"
} else {
  "x86_64"
}

$Target = "$Arch-pc-windows-msvc"

if (-not $Version) {
  Write-Host "Finding latest release of $Repo..."

  $ApiUrl = "https://api.github.com/repos/$Repo/releases/latest"

  try {
    $Release = Invoke-RestMethod -Uri $ApiUrl -Headers @{ "User-Agent" = "onx-installer" }
    $Version = $Release.tag_name
  } catch {
    Write-Warning "Could not query GitHub API for latest release, falling back to web request..."

    $Response = Invoke-WebRequest -Uri "$GithubUrl/releases/latest" -MaximumRedirection 0 -ErrorAction SilentlyContinue
    $Location = $Response.Headers["Location"]
    
    if ($Location) {
      $Version = ($Location -split '/')[-1]
    }
  }
}

if ($Version -and -not $Version.StartsWith("v")) {
  $Version = "v$Version"
}

if (-not $Version) {
  Write-Error "Could not determine version to install."

  exit 1
}

$ArchiveName = "onx-$Version-$Target.zip"
$DownloadUrl = "$GithubUrl/releases/download/$Version/$ArchiveName"
$ChecksumUrl = "$DownloadUrl.sha256"

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
  $ArchiveFile = Join-Path $TempDir $ArchiveName
  $ChecksumFile = Join-Path $TempDir "$ArchiveName.sha256"

  Write-Host "Downloading $BinaryName $Version for $Target..."

  Invoke-WebRequest -Uri $DownloadUrl -OutFile $ArchiveFile
  Invoke-WebRequest -Uri $ChecksumUrl -OutFile $ChecksumFile

  Write-Host "Verifying SHA256 checksum..."

  $ExpectedHash = ((Get-Content $ChecksumFile) -split '\s+')[0].Trim().ToLower()
  $ActualHash = (Get-FileHash -Path $ArchiveFile -Algorithm SHA256).Hash.ToLower()

  if ($ExpectedHash -ne $ActualHash) {
    Write-Error "Checksum mismatch!`n  Expected: $ExpectedHash`n  Actual:   $ActualHash"

    exit 1
  }

  Write-Host "Extracting $ArchiveName..."

  $ExtractDir = Join-Path $TempDir "extracted"
  Expand-Archive -Path $ArchiveFile -DestinationPath $ExtractDir -Force

  $FoundBin = Get-ChildItem -Path $ExtractDir -Recurse -Filter $BinaryName | Select-Object -First 1
  
  if (-not $FoundBin) {
    Write-Error "Binary '$BinaryName' not found inside archive."

    exit 1
  }

  New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
  $TargetFile = Join-Path $InstallDir $BinaryName
  Copy-Item -Path $FoundBin.FullName -Destination $TargetFile -Force

  Write-Host "$BinaryName $Version installed successfully to $TargetFile!" -ForegroundColor Green

  $UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
  $PathParts = ($UserPath -split ';') | Where-Object { $_ -ne "" }

  if ($PathParts -notcontains $InstallDir) {
    Write-Host "Adding $InstallDir to user PATH..."

    $NewUserPath = "$UserPath;$InstallDir".Trim(';')
    [Environment]::SetEnvironmentVariable("Path", $NewUserPath, [EnvironmentVariableTarget]::User)
    $env:Path = "$env:Path;$InstallDir"

    Write-Host "Added $InstallDir to user PATH. You may need to restart your terminal for the change to take effect." -ForegroundColor Cyan
  } else {
    Write-Host "$InstallDir is already in your PATH."
  }
} finally {
  if (Test-Path $TempDir) {
    Remove-Item -Recurse -Force -Path $TempDir
  }
}
