import {readFileSync} from 'node:fs';
import {describe,it,expect} from 'vitest';

const source=(path:string)=>readFileSync(new URL('../'+path,import.meta.url),'utf8');
describe('Single-root developer storage admission — source-only',()=>{
  it('resolves approved dev targets to the checkout before App staging, preserving installed paths',()=>{
    const rust=source('src-tauri/src/engine/manager_settings.rs');
    expect(rust).toContain('let developer_owned_target =');
    expect(rust).toContain('workspace/builds/desktop/target');
    expect(rust).toContain('Development/apps/desktop/src-tauri/target');
    expect(rust).toContain('if ancestor.join("App").is_dir()');
    expect(rust).toContain('if source_checkout && developer_owned_target');
    expect(rust).toContain('fs::symlink_metadata(folder.join("Data"))');
    expect(rust).toContain('io::ErrorKind::AlreadyExists');
    expect(rust).toContain('Ok(folder.to_path_buf())');
    expect(rust).toContain('developer_root_without_app_stays_at_checkout_and_preserves_legacy_data');
  });
  it('keeps private profiles outside source, build output and installer exports',()=>{
    const packageSource=source('scripts/package-lazy-tester.mjs');
    const launcher=source('scripts/root-launcher.rs');
    expect(packageSource).toContain("const app = resolve(cwd,'App'), data = resolve(cwd,'Data');");
    expect(packageSource).toContain("workspace/private/local-app-backups");
    expect(packageSource).toContain("workspace/packages/lazy-tester");
    expect(launcher).toContain('let app = root.join("App");');
    expect(launcher).toContain('.current_dir(target.parent().unwrap())');
    expect(packageSource).not.toContain("renameSync(data,");
  });
});
