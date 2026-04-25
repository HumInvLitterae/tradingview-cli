$ErrorActionPreference = "Stop"

$repoRoot = git rev-parse --show-toplevel
Set-Location $repoRoot

$gitVersionOutput = git version
if ($gitVersionOutput -notmatch 'git version (?<version>\d+)\.(?<minor>\d+)') {
    throw "Could not parse Git version from: $gitVersionOutput"
}

$major = [int]$Matches.version
$minor = [int]$Matches.minor
if (($major -lt 2) -or (($major -eq 2) -and ($minor -lt 54))) {
    throw "Git 2.54 or newer is required for config-based hooks. Found: $gitVersionOutput"
}

$shell = Get-Command pwsh -ErrorAction SilentlyContinue
if ($null -eq $shell) {
    $shell = Get-Command powershell.exe -ErrorAction SilentlyContinue
}
if ($null -eq $shell) {
    throw "PowerShell was not found. Install PowerShell or run the shell installer from Git Bash."
}

$shellName = $shell.Name
if ($shellName -eq "powershell.exe") {
    $fastCommand = 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/git-hooks/pre-commit-fast.ps1'
    $baselineCommand = 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/git-hooks/pre-push-baseline.ps1'
} else {
    $fastCommand = 'pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/git-hooks/pre-commit-fast.ps1'
    $baselineCommand = 'pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/git-hooks/pre-push-baseline.ps1'
}

git config --local hook.tv-fast.event pre-commit
git config --local hook.tv-fast.command $fastCommand
git config --local hook.tv-fast.enabled true

git config --local hook.tv-baseline.event pre-push
git config --local hook.tv-baseline.command $baselineCommand
git config --local hook.tv-baseline.enabled true

git hook list pre-commit
git hook list pre-push
