const https = require('https');

const packageJson = require('./package.json');
const VERSION = packageJson.version;
const REPO = 'SamitoX4/devc';

function checkRelease() {
  return new Promise((resolve, reject) => {
    https
      .get(
        `https://api.github.com/repos/${REPO}/releases/tags/v${VERSION}`,
        {
          headers: {
            'User-Agent': 'devc-check-release',
            Accept: 'application/vnd.github.v3+json',
          },
        },
        (res) => {
          if (res.statusCode === 200) {
            resolve(true);
          } else if (res.statusCode === 404) {
            resolve(false);
          } else {
            reject(new Error(`GitHub API returned HTTP ${res.statusCode}`));
          }
        }
      )
      .on('error', reject);
  });
}

async function main() {
  console.log(`Checking GitHub Release v${VERSION}...`);
  let exists;
  try {
    exists = await checkRelease();
  } catch (err) {
    console.error(`Error checking release: ${err.message}`);
    process.exit(1);
  }

  if (exists) {
    console.log('✓ Release exists on GitHub. Safe to publish to npm.');
    process.exit(0);
  } else {
    console.error(`✗ Release v${VERSION} NOT FOUND on GitHub.`);
    console.error('');
    console.error('Please wait for GitHub Actions to finish building the release.');
    console.error('You can check progress at:');
    console.error(`  https://github.com/${REPO}/actions`);
    console.error('');
    console.error('Once the release is published, run npm publish again.');
    process.exit(1);
  }
}

main();
