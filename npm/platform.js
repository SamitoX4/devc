function getPlatformAsset(version) {
  const platform = process.platform;
  const arch = process.arch;

  const targets = {
    win32: {
      x64: {
        name: `devc-v${version}-x86_64-pc-windows-msvc.zip`,
        platform: 'windows-x64',
        bin: 'devc.exe',
      },
    },
    linux: {
      x64: {
        name: `devc-v${version}-x86_64-unknown-linux-gnu.tar.gz`,
        platform: 'linux-x64',
        bin: 'devc',
      },
    },
    darwin: {
      x64: {
        name: `devc-v${version}-x86_64-apple-darwin.tar.gz`,
        platform: 'macos-x64',
        bin: 'devc',
      },
      arm64: {
        name: `devc-v${version}-aarch64-apple-darwin.tar.gz`,
        platform: 'macos-arm64',
        bin: 'devc',
      },
    },
  };

  const p = targets[platform];
  if (!p) return null;
  return p[arch] || null;
}

module.exports = { getPlatformAsset };
