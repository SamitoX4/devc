const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');
const child_process = require('child_process');

const packageJson = require('./package.json');
const { getPlatformAsset } = require('./platform');

const VERSION = packageJson.version;
const REPO = 'SamitoX4/devc';
const VENDOR_DIR = path.join(__dirname, 'vendor');

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    const protocol = url.startsWith('https') ? https : http;
    protocol
      .get(url, (response) => {
        if (response.statusCode === 301 || response.statusCode === 302) {
          download(response.headers.location, dest)
            .then(resolve)
            .catch(reject);
          return;
        }
        if (response.statusCode !== 200) {
          reject(new Error(`Download failed: HTTP ${response.statusCode}`));
          return;
        }
        response.pipe(file);
        file.on('finish', () => {
          file.close(resolve);
        });
      })
      .on('error', reject);
  });
}

function extract(archivePath, destDir) {
  if (archivePath.endsWith('.zip')) {
    child_process.execSync(
      `powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${destDir}' -Force"`,
      { stdio: 'inherit' }
    );
  } else {
    fs.mkdirSync(destDir, { recursive: true });
    child_process.execSync(`tar xzf "${archivePath}" -C "${destDir}"`, {
      stdio: 'inherit',
    });
  }
}

async function main() {
  const asset = getPlatformAsset(VERSION);
  if (!asset) {
    console.error(
      `Unsupported platform: ${process.platform} ${process.arch}`
    );
    process.exit(1);
  }

  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${asset.name}`;
  const archivePath = path.join(__dirname, asset.name);

  fs.mkdirSync(VENDOR_DIR, { recursive: true });

  console.log(`Downloading devc v${VERSION} for ${asset.platform}...`);
  console.log(`URL: ${url}`);

  await download(url, archivePath);

  console.log('Extracting...');
  extract(archivePath, VENDOR_DIR);

  fs.unlinkSync(archivePath);

  if (process.platform !== 'win32') {
    const binPath = path.join(VENDOR_DIR, asset.bin);
    if (fs.existsSync(binPath)) {
      fs.chmodSync(binPath, 0o755);
    }
  }

  console.log('devc installed successfully.');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
