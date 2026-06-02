#!/usr/bin/env node
const path = require('path');
const child_process = require('child_process');
const fs = require('fs');
const { getPlatformAsset } = require('../platform');

const vendorDir = path.join(__dirname, '..', 'vendor');
const asset = getPlatformAsset(require('../package.json').version);

if (!asset) {
  console.error(
    `Unsupported platform: ${process.platform} ${process.arch}`
  );
  process.exit(1);
}

const binPath = path.join(vendorDir, asset.bin);

if (!fs.existsSync(binPath)) {
  console.error(
    'devc binary not found. Please reinstall the package (npm install -g devc).'
  );
  process.exit(1);
}

const result = child_process.spawnSync(binPath, process.argv.slice(2), {
  stdio: 'inherit',
  shell: false,
});

process.exit(result.status !== null ? result.status : 1);
