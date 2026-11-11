// Copy the Dispatch binary into the Tauri resource folder so the installer can
// ship it. Build machines need `dispatch` on PATH (or BOOST_DISPATCH_PATH).
import { copyFileSync, mkdirSync, statSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const target = join(root, 'src-tauri', 'binaries', 'dispatch.exe');

let source = process.env.BOOST_DISPATCH_PATH;
if (!source) {
  try {
    source = execFileSync('where.exe', ['dispatch'], { encoding: 'utf8' }).split(/\r?\n/).find(Boolean)?.trim();
  } catch {
    source = undefined;
  }
}
if (!source || !source.toLowerCase().endsWith('.exe')) {
  console.error('Cannot package Boost: dispatch.exe was not found. Install it (cargo install dispatch-proxy) or set BOOST_DISPATCH_PATH.');
  process.exit(1);
}

const version = execFileSync(source, ['--version'], { encoding: 'utf8' }).trim();
mkdirSync(dirname(target), { recursive: true });
copyFileSync(source, target);
console.log(`Bundled ${version} from ${source}`);
console.log(`  ${target} (${statSync(target).size.toLocaleString()} bytes)`);
