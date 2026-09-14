#Requires -Version 7.4
#
# Contract tests for scripts/publish/node/package-artifacts.ps1.
#
# FAN-OUT SHARED: byte-identical in xberg and kreuzberg-lts. The two copies of the script differ
# on thirty lines and every one of them is the product token, so this suite must never type one --
# NodeCrate and NodePrefix are derived from crates/*-node. Copy it with `cp`, never by hand.
#
# The subject resolves everything from the current working directory, so each test runs it from a
# scratch tree rather than the repository. That also keeps it from writing a tarball into the
# checkout.

BeforeAll {
    $script:RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..' '..' '..')).Path
    $script:Sut = Join-Path $script:RepoRoot 'scripts/publish/node/package-artifacts.ps1'
    $script:Pwsh = (Get-Process -Id $PID).Path

    # Derived, never written: this file is byte-identical in the sibling repository whose crate is
    # named for a different product. ~keep
    $nodeCrateDir = Get-ChildItem -Path (Join-Path $script:RepoRoot 'crates') -Directory |
        Where-Object { $_.Name -like '*-node' } | Select-Object -First 1
    if ($null -eq $nodeCrateDir) { throw "no crates/*-node crate under $script:RepoRoot" }
    $script:NodeCrate = $nodeCrateDir.Name            # e.g. xberg-node
    $script:NodePrefix = $script:NodeCrate             # the .node files are named for the crate

    function New-Workspace {
        param([switch]$OmitNpmDir)
        $root = Join-Path $TestDrive ([Guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory -Path $root -Force | Out-Null
        if (-not $OmitNpmDir) {
            New-Item -ItemType Directory -Force -Path (Join-Path $root "crates/$script:NodeCrate/npm") | Out-Null
        }
        return $root
    }

    function Add-BuiltBinary {
        param(
            [Parameter(Mandatory)][string]$Root,
            [Parameter(Mandatory)][string]$NodeFile,
            [ValidateSet('artifacts', 'crate-root')][string]$Where = 'artifacts'
        )
        $directory = if ($Where -eq 'artifacts') {
            Join-Path $Root "crates/$script:NodeCrate/artifacts"
        } else {
            Join-Path $Root "crates/$script:NodeCrate"
        }
        New-Item -ItemType Directory -Force -Path $directory | Out-Null
        Set-Content -LiteralPath (Join-Path $directory $NodeFile) -Value 'NAPI' -NoNewline
    }

    function Invoke-Sut {
        param([Parameter(Mandatory)][string]$Root, [string]$Target)
        Push-Location $Root
        try {
            $output = & $script:Pwsh -NoProfile -NonInteractive -Command @"
`$env:TARGET = '$Target'
& '$($script:Sut)'
"@ 2>&1
            [pscustomobject]@{ ExitCode = $LASTEXITCODE; Output = ($output | Out-String) }
        } finally {
            Pop-Location
        }
    }
}

Describe 'package-artifacts.ps1' {

    It 'should_place_the_napi_binary_under_the_platform_directory_for_<target>' -ForEach @(
        @{ Target = 'aarch64-apple-darwin'; PlatformDir = 'darwin-arm64' }
        @{ Target = 'x86_64-apple-darwin'; PlatformDir = 'darwin-x64' }
        @{ Target = 'x86_64-pc-windows-msvc'; PlatformDir = 'win32-x64-msvc' }
        @{ Target = 'aarch64-pc-windows-msvc'; PlatformDir = 'win32-arm64-msvc' }
        @{ Target = 'x86_64-unknown-linux-gnu'; PlatformDir = 'linux-x64-gnu' }
        @{ Target = 'aarch64-unknown-linux-gnu'; PlatformDir = 'linux-arm64-gnu' }
        @{ Target = 'armv7-unknown-linux-gnueabihf'; PlatformDir = 'linux-arm-gnueabihf' }
    ) {
        # The seven-way switch is the whole contract of this script: a wrong platform directory
        # publishes a binary npm will never resolve for that platform.
        $root = New-Workspace
        $nodeFile = "$script:NodePrefix.$PlatformDir.node"
        Add-BuiltBinary -Root $root -NodeFile $nodeFile

        $result = Invoke-Sut -Root $root -Target $Target

        $result.ExitCode | Should -Be 0
        Test-Path (Join-Path $root "crates/$script:NodeCrate/npm/$PlatformDir/$nodeFile") | Should -BeTrue
    }

    It 'should_write_a_tarball_named_for_the_target_when_packaging_succeeds' {
        $root = New-Workspace
        Add-BuiltBinary -Root $root -NodeFile "$script:NodePrefix.darwin-arm64.node"

        Invoke-Sut -Root $root -Target 'aarch64-apple-darwin' | Out-Null

        Test-Path (Join-Path $root 'node-bindings-aarch64-apple-darwin.tar.gz') | Should -BeTrue
    }

    It 'should_find_the_binary_at_the_crate_root_when_it_is_not_under_artifacts' {
        $root = New-Workspace
        Add-BuiltBinary -Root $root -NodeFile "$script:NodePrefix.darwin-arm64.node" -Where 'crate-root'

        $result = Invoke-Sut -Root $root -Target 'aarch64-apple-darwin'

        $result.ExitCode | Should -Be 0
        Test-Path (Join-Path $root "crates/$script:NodeCrate/npm/darwin-arm64/$script:NodePrefix.darwin-arm64.node") |
            Should -BeTrue
    }

    It 'should_throw_and_name_the_target_when_it_is_not_a_supported_napi_triple' {
        $root = New-Workspace
        Add-BuiltBinary -Root $root -NodeFile "$script:NodePrefix.darwin-arm64.node"

        $result = Invoke-Sut -Root $root -Target 'sparc64-unknown-linux-gnu'

        $result.ExitCode | Should -Not -Be 0
        $result.Output | Should -Match 'Unsupported NAPI target: sparc64-unknown-linux-gnu'
    }

    It 'should_throw_when_the_target_variable_is_empty' {
        $root = New-Workspace
        $result = Invoke-Sut -Root $root -Target ''
        $result.ExitCode | Should -Not -Be 0
        $result.Output | Should -Match 'TARGET not set'
    }

    It 'should_throw_before_reading_the_target_when_the_npm_directory_is_missing' {
        $root = New-Workspace -OmitNpmDir
        $result = Invoke-Sut -Root $root -Target 'aarch64-apple-darwin'
        $result.ExitCode | Should -Not -Be 0
        $result.Output | Should -Match 'npm artifact directory missing'
    }

    It 'should_list_the_node_files_it_did_find_when_the_expected_binary_is_absent' {
        # The listing is the whole point of that branch: a bare "missing" on a release runner
        # leaves no way to tell a misnamed artifact from an unbuilt one.
        $root = New-Workspace
        Add-BuiltBinary -Root $root -NodeFile "$script:NodePrefix.linux-x64-gnu.node"

        $result = Invoke-Sut -Root $root -Target 'aarch64-apple-darwin'

        $result.ExitCode | Should -Not -Be 0
        $result.Output | Should -Match 'NAPI binary missing'
        $result.Output | Should -Match ([regex]::Escape("$script:NodePrefix.linux-x64-gnu.node"))
    }

    Context 'when a PDFium library is present' {

        BeforeEach {
            $script:Root = New-Workspace
            Add-BuiltBinary -Root $script:Root -NodeFile "$script:NodePrefix.darwin-arm64.node"
            New-Item -ItemType Directory -Force -Path (Join-Path $script:Root 'target/release') | Out-Null
            Set-Content -LiteralPath (Join-Path $script:Root 'target/release/libpdfium.dylib') -Value 'PDFIUM' -NoNewline
            $platformDir = Join-Path $script:Root "crates/$script:NodeCrate/npm/darwin-arm64"
            New-Item -ItemType Directory -Force -Path $platformDir | Out-Null
            Set-Content -LiteralPath (Join-Path $platformDir 'package.json') -Value '{"name":"pkg","files":["index.js"]}'
        }

        It 'should_copy_the_library_beside_the_binary_and_list_it_in_package_json' {
            $result = Invoke-Sut -Root $script:Root -Target 'aarch64-apple-darwin'
            $result.ExitCode | Should -Be 0

            $platformDir = Join-Path $script:Root "crates/$script:NodeCrate/npm/darwin-arm64"
            Test-Path (Join-Path $platformDir 'libpdfium.dylib') | Should -BeTrue

            $pkg = Get-Content (Join-Path $platformDir 'package.json') -Raw | ConvertFrom-Json
            $pkg.files | Should -Contain 'libpdfium.dylib'
            # The pre-existing entry has to survive: npm publishes exactly this list.
            $pkg.files | Should -Contain 'index.js'
        }

        It 'should_not_add_a_duplicate_entry_when_package_json_already_lists_the_library' {
            $platformDir = Join-Path $script:Root "crates/$script:NodeCrate/npm/darwin-arm64"
            Set-Content -LiteralPath (Join-Path $platformDir 'package.json') `
                -Value '{"name":"pkg","files":["index.js","libpdfium.dylib"]}'

            Invoke-Sut -Root $script:Root -Target 'aarch64-apple-darwin' | Out-Null

            $pkg = Get-Content (Join-Path $platformDir 'package.json') -Raw | ConvertFrom-Json
            @($pkg.files | Where-Object { $_ -eq 'libpdfium.dylib' }).Count | Should -Be 1
        }

        It 'should_throw_when_the_platform_package_json_is_missing' {
            Remove-Item (Join-Path $script:Root "crates/$script:NodeCrate/npm/darwin-arm64/package.json") -Force
            $result = Invoke-Sut -Root $script:Root -Target 'aarch64-apple-darwin'
            $result.ExitCode | Should -Not -Be 0
            $result.Output | Should -Match 'Platform package.json missing'
        }
    }

    It 'should_warn_rather_than_fail_when_no_pdfium_library_is_found' {
        # Deliberately non-fatal: the pdfium-less build is a supported configuration, so turning
        # this into a throw would break it.
        $root = New-Workspace
        Add-BuiltBinary -Root $root -NodeFile "$script:NodePrefix.darwin-arm64.node"

        $result = Invoke-Sut -Root $root -Target 'aarch64-apple-darwin'

        $result.ExitCode | Should -Be 0
        $result.Output | Should -Match 'libpdfium\.dylib not found in any expected location'
    }
}
