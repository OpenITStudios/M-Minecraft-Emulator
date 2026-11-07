// Source-only agent-rule wiring checks. Authored; not executed by this GitHub change.
// This test does not evaluate real agent routing or native Minecraft acceptance.
import assert from 'node:assert/strict';
import {existsSync, readFileSync, readdirSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const textAt = path => readFileSync(join(root, path), 'utf8');
const policy = textAt('AGENTS.md');
const skill = textAt('.agents/skills/emulator-development/SKILL.md');
const guide = textAt('docs/system/development-discipline.md');
const index = textAt('.agents/README.md');
const evals = textAt('.agents/evals/matt-pocock-method-scenarios.md');

test('external method is a conditional pointer inside one canonical Skill', () => {
  assert.match(policy, /Eight independent reference methods/);
  assert.match(policy, /Matt Pocock Skills/);
  assert.match(policy, /matt-pocock-engineering-method-adapted/);
  assert.match(policy, /Retain the single emulator-development Skill/);
  assert.match(skill, /## Selective Matt Pocock method within this Skill/);
  assert.match(index, /only.*repository Skill/);
  const dirs = readdirSync(join(root, '.agents/skills'), {withFileTypes: true})
    .filter(entry => entry.isDirectory()).map(entry => entry.name);
  assert.deepEqual(dirs, ['emulator-development']);
});

test('local adaptation covers the hard diagnosis, design and review contracts', () => {
  for (const heading of [
    '## Matt Pocock engineering method (adapted)',
    '### A. Domain vocabulary and deep module interfaces',
    '### B. Clarify only material decisions and implement vertical slices',
    '### C. Bug diagnosis: distinguish the failure before patching',
    '### D. Two-axis review before the source commit',
    '### E. Source research and agent-documentation economy',
    '### F. Evaluation and explicit proof ceiling',
  ]) assert.ok(guide.includes(heading), 'missing method section: ' + heading);
  for (const expected of [
    'PROPOSED_NOT_EXECUTED', 'AUTHORED_NOT_EXECUTED',
    'Standards', 'Spec', 'FIVE_CLIENT_ACCEPTANCE',
  ]) assert.ok(guide.includes(expected) || policy.includes(expected),
    'missing proof or review term: ' + expected);
  assert.ok(skill.includes('The operator\'s existing hold on local'));
});

test('source pointers and scenarios are well-formed and preserve hard boundaries', () => {
  for (const path of [
    '.agents/README.md',
    '.agents/evals/matt-pocock-method-scenarios.md',
    'docs/system/development-discipline.md',
    'docs/system/implementation-map.md',
  ]) assert.equal(existsSync(join(root, path)), true, 'missing path: ' + path);
  for (let n = 1; n <= 10; n++) assert.ok(
    evals.includes('| E' + n + ' '), 'missing evaluation scenario E' + n);
  assert.match(evals, /AUTHORED_NOT_EXECUTED/);
  assert.match(guide, /AGENT_EFFECTIVENESS_NOT_MEASURED/);
  assert.match(policy, /GitHub Actions CI is disabled/);
  assert.match(policy, /Local.*only active working branch/);
});
