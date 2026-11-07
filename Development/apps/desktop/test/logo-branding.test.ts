import {readFileSync} from 'node:fs';
import {describe,expect,it} from 'vitest';

const source=(path:string)=>readFileSync(new URL('../'+path,import.meta.url),'utf8');
const config=(path:string)=>JSON.parse(source(path));

describe('LazyTester approved icon and installed-identity separation',()=>{
  it('uses one real vector source in every existing WebView logo surface',()=>{
    const logo=source('src-tauri/app-icon.svg');
    const css=source('src/lazy-tester.css');
    const play=source('src/LazyTesterApp.svelte');
    const titlebar=source('src/WindowTitlebar.svelte');
    expect(logo).toContain('viewBox="0 0 256 256"');
    expect(logo).toContain('aria-label="LazyTester"');
    expect(logo).toContain('five session tiles');
    expect(logo).toContain('fill="#343DF3"');
    expect(logo).toContain('<g fill="#FFFFFF">');
    expect((logo.match(/<path /g)??[])).toHaveLength(8);
    expect(logo).not.toMatch(/<image|data:image\/|<foreignObject|<script|<text\b/i);
    expect(css).toContain('url("../src-tauri/app-icon.svg")');
    expect(play).toContain('<span class="lt-brand-name">LazyTester</span>');
    expect(titlebar).toContain('<span data-tauri-drag-region>LazyTester</span>');
  });

  it('regenerates native icon binaries from a changed SVG independent of checkout mtime',()=>{
    const generator=source('scripts/prepare-icons.mjs');
    expect(generator).toContain("resolve(root,'src-tauri','app-icon.svg')");
    expect(generator).toContain("createHash('sha256')");
    expect(generator).toContain("readFileSync(source)");
    expect(generator).toContain("'.app-icon-source.sha256'");
    for(const icon of ['32x32.png','128x128.png','128x128@2x.png','icon.ico'])
      expect(generator).toContain("'"+icon+"'");
    expect(generator).toContain("if(!outputs.every(existsSync))throw new Error");
    expect(generator).toContain("writeFileSync(stamp,digest+'\\n','utf8')");
    const base=config('src-tauri/tauri.conf.json');
    expect(base.bundle.icon).toContain('icons/icon.ico');
    expect(base.bundle.windows.nsis.installerIcon).toBe('icons/icon.ico');
    expect(base.bundle.windows.nsis.uninstallerIcon).toBe('icons/icon.ico');
  });

  it('uses LazyTester as visible brand but never silently changes installed identity',()=>{
    const base=config('src-tauri/tauri.conf.json');
    const active=config('src-tauri/tauri.lazy-tester.conf.json');
    expect(base.app.windows[0].title).toBe('LazyTester');
    expect(active.app.windows[0].title).toBe('LazyTester');
    expect(active.productName).toBe('Lazy Tester');
    expect(active.identifier).toBe('com.mivubi.lazytester');
    expect(active.mainBinaryName).toBe('lazy-tester');
    expect(active.bundle.windows.nsis.startMenuFolder).toBe('Lazy Tester');
    expect(source('src/LazyTesterApp.svelte')).toContain('<title>LazyTester</title>');
    expect(source('src/GeneralSettings.svelte')).not.toContain('Lazy Tester');
  });
});
