import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {
  ownedMinecraftProfiles, planOwnedMinecraftProfiles, planOwnedMinecraftVirtual,
  ownedMinecraftLaunch, matchesOwnedMinecraftReadback,
} from "./minecraft-profiles.mjs";

test("each client hides host game data and rejects the old read-through profile", () => {
  for (const client of ["02", "03", "04"]) {
    const plan = planOwnedMinecraftVirtual(client, []);
    const readings = plan.guardedSteps.slice(2)
      .filter(step => step.operation === "REQUIRE_READBACK")
      .map(step => ({client, box: step.arguments[1], key: step.arguments[2], value: step.expectedValue}));
    assert.deepEqual(readings.find(row => row.key === "WriteFilePath"), {
      client, box: "MivubiEdu" + client, key: "WriteFilePath",
      value: "%AppData%\\Minecraft Education Edition",
    });
    assert.equal(matchesOwnedMinecraftReadback(client, readings), true);
    assert.equal(matchesOwnedMinecraftReadback(client,
      readings.filter(row => row.key !== "WriteFilePath")), false);
    const changed = structuredClone(readings);
    changed.find(row => row.key === "WriteFilePath").value = "%AppData%";
    assert.equal(matchesOwnedMinecraftReadback(client, changed), false);
  }
});

test("three functional MIVUBI boxes have stable distinct identities", () => {
  const actual = ownedMinecraftProfiles();
  assert.deepEqual(actual.map(x=>x.client), ["02","03","04"]);
  assert.deepEqual(actual.map(x=>x.box),
    ["MivubiEdu02","MivubiEdu03","MivubiEdu04"]);
  assert.ok(actual.every(x=>x.persistent === true &&
    x.accountAuthenticated === false && x.windowsDriverVerified === false));
});

test("private box setup plan uses only exact non-destructive persistent settings", () => {
  const result = planOwnedMinecraftProfiles(["DefaultBox","MinecraftClient02"]);
  assert.equal(result.commands.length, 27);
  assert.equal(result.hostChanged, false);
  assert.equal(result.driverInstalled, false);
  assert.equal(result.compatibilityProfile,
    "MINECRAFT_EDUCATION_APP_COMPARTMENT_OPENCOM");
  assert.equal(result.reducedSecurityIsolation, true);
  assert.equal(result.requiresExplicitOperatorApproval, true);
  assert.equal(result.settingsReadbackVerified, false);
  for(let i=0; i<3; i++){
    const steps=result.commands.slice(i*9, i*9+9);
    assert.deepEqual(steps.map(step=>step.arguments[2]),
      ["AutoDelete","AutoRemove","NoSecurityIsolation","DropAdminRights",
        "Template","OpenCredentials","OpenProtectedStorage","WriteFilePath","Enabled"]);
    assert.deepEqual(steps.map(step=>step.arguments[3]),
      ["n","n","y","y","OpenCOM","n","n","%AppData%\\Minecraft Education Edition","y"]);
    assert.equal(steps[8].arguments[3],"y","enable the owned box only after configuration");
  }
  for(const cmd of result.commands) {
    assert.equal(cmd.executable, "SbieIni.exe");
    assert.deepEqual(cmd.arguments.slice(0,2), ["set",cmd.arguments[1]]);
    assert.ok(/^MivubiEdu0[234]$/.test(cmd.arguments[1]));
    assert.ok([["Enabled","y"],["AutoDelete","n"],
      ["AutoRemove","n"],["NoSecurityIsolation","y"],
      ["DropAdminRights","y"],["Template","OpenCOM"],
      ["OpenCredentials","n"],["OpenProtectedStorage","n"],["WriteFilePath","%AppData%\\Minecraft Education Edition"]]
      .some(([k,v])=>k===cmd.arguments[2]&&v===cmd.arguments[3]));
    assert.equal(cmd.arguments.length,4);
  }
});

test("owned compatibility mode retains pilot settings and adds box-only game data", () => {
  const plan = planOwnedMinecraftProfiles([]);
  const expected = {
    NoSecurityIsolation: "y", DropAdminRights: "y",
    Template: "OpenCOM", AutoDelete: "n", AutoRemove: "n",
    OpenCredentials: "n", OpenProtectedStorage: "n",
    WriteFilePath: "%AppData%\\Minecraft Education Edition", Enabled: "y",
  };
  for (const client of ["02", "03", "04"]) {
    const settings = Object.fromEntries(plan.commands
      .filter(x=>x.client===client)
      .map(x=>x.arguments.slice(2,4)));
    assert.deepEqual(settings, expected);
  }
  assert.equal(new Set(plan.commands.map(x=>x.arguments[1])).size, 3);
  assert.ok(plan.commands.every(x=>x.arguments[1] === "MivubiEdu"+x.client));
  assert.equal(plan.hostChanged, false);
  assert.equal(plan.settingsReadbackVerified, false);
});

test("existing named boxes never get silently reconfigured", () => {
  for(const names of [["MivubiEdu02"],["mivubiedu03"],["MivubiEdu04"],
    ["MinecraftClient02","minecraftclient02"]]) {
    assert.throws(()=>planOwnedMinecraftProfiles(names));
  }
  for(const invalid of [null,"MivubiEdu02",["bad name"],["../DefaultBox"]]) {
    assert.throws(()=>planOwnedMinecraftProfiles(invalid));
  }
});

test("owned source plans require an explicit authoritative box inventory", () => {
  // An omitted or unknown inventory must not be silently replaced with [].
  for (const invalid of [undefined, null, "UNKNOWN", {}, false]) {
    assert.throws(() => planOwnedMinecraftVirtual("02", invalid),
      /trustworthy existing box-name inventory/);
    assert.throws(() => planOwnedMinecraftProfiles(invalid),
      /trustworthy existing box-name inventory/);
  }
  assert.equal(planOwnedMinecraftVirtual("02", []).commands.length, 9);
  assert.equal(planOwnedMinecraftProfiles([]).commands.length, 27);
});

test("owned Start launch selects only one exact profile and Education EXE",()=>{
  for(const id of ["02","03","04"]) {
    const exe = "C:\\Program Files\\Microsoft Studios\\Minecraft Education\\Minecraft.Windows.exe";
    const out = ownedMinecraftLaunch(id,exe);
    assert.equal(out.box,"MivubiEdu"+id);
    assert.deepEqual(out.arguments,["/mivubi_client:"+id,exe]);
    assert.equal(out.rawCommandLine,'/mivubi_client:'+id+' "'+exe+'"');
    assert.equal(out.driverInstalled,false);
    assert.equal(out.gameplayVerified,false);
  }
  for(const path of [
    "C:\\MinecraftEducation.exe","D:\\Apps\\MinecraftEducationEdition.exe",
  ]) {
    assert.equal(ownedMinecraftLaunch("02",path).arguments[1],path);
  }
});

test("rejects arbitrary code, paths, client IDs, UNC, args, and shell",()=>{
  const valid="C:\\Minecraft.Windows.exe";
  for(const id of ["01","05","2","03 ","__proto__",2,null]) {
    assert.throws(()=>ownedMinecraftLaunch(id,valid));
  }
  for(const path of [
    "minecraft.windows.exe","..\\Minecraft.Windows.exe",
    "\\\\server\\Minecraft.Windows.exe","C:\\Minecraft.Windows.exe /silent",
    "C:\\Minecraft.Windows.exe\" /terminate_all",
    "C:\\Minecraft.Windows.exe\u0000",
    "C:\\Notepad.exe","C:\\foo\\..\\Minecraft.Windows.exe",
    "C:/Minecraft.Windows.exe","C:\\Minecraft.Windows.exe|powershell",
    "C:\\folder:stream\\Minecraft.Windows.exe",
    "C:\\folder/other\\Minecraft.Windows.exe",
    "C:\\folder\\subdir:ads\\Minecraft.Windows.exe",
  ]) {
    assert.throws(()=>ownedMinecraftLaunch("02",path),path);
  }
});

test("control contract and owned Start parser share the same profile namespace",()=>{
  const source=readFileSync(new URL("../fork-overlay/apply-private-source.mjs",import.meta.url),"utf8");
  const policy=readFileSync(new URL("./mivubi_owned_start_policy.h",import.meta.url),"utf8");
  assert.match(source,/mivubi_owned_start_policy\.h/);
  assert.match(policy,/mivubi_client:/);
  assert.match(policy,/MivubiEdu02/);
  assert.match(policy,/MivubiEdu03/);
  assert.match(policy,/MivubiEdu04/);
  assert.match(policy,/MivubiParseOwnedStart/);
  assert.match(policy,/end\[1\] != L'/);
  assert.match(policy,/MinecraftEducationEdition\.exe/);
  assert.ok(policy.includes("c == L':'"));
  assert.ok(policy.includes("c == L'/'"));
  assert.doesNotMatch(source,/SbieDrv2|SbieSvc2|disable.*signing/i);
});

test("one Create Virtual target plans only one native box, with Enabled last", () => {
  for (const client of ["02", "03", "04"]) {
    const result = planOwnedMinecraftVirtual(client, ["MinecraftClient02", "OtherBox"]);
    assert.deepEqual(result.profiles.map(p => p.client), [client]);
    assert.deepEqual(result.profiles.map(p => p.box), ["MivubiEdu" + client]);
    assert.equal(result.commands.length, 9);
    assert.ok(result.commands.every(cmd =>
      cmd.client === client && cmd.executable === "SbieIni.exe" &&
      cmd.arguments[1] === "MivubiEdu" + client));
    assert.deepEqual(result.commands.map(cmd => cmd.arguments[2]),
      ["AutoDelete", "AutoRemove", "NoSecurityIsolation", "DropAdminRights",
        "Template", "OpenCredentials", "OpenProtectedStorage", "WriteFilePath", "Enabled"]);
    assert.deepEqual(result.commands.map(cmd => cmd.arguments[3]),
      ["n", "n", "y", "y", "OpenCOM", "n", "n", "%AppData%\\Minecraft Education Edition", "y"]);
    assert.equal(result.settingsReadbackVerified, false);
    assert.equal(result.requiresExplicitOperatorApproval, true);
    assert.equal(result.hostChanged, false);
    assert.equal(result.driverInstalled, false);
    assert.equal(result.accountIsolationVerified, false);
  }
});

test("single Create Virtual rejects stale, duplicate and foreign target inventories", () => {
  for (const client of ["01", "05", "2", "__proto__", "", null, 2]) {
    assert.throws(() => planOwnedMinecraftVirtual(client, []));
  }
  for (const existing of [
    ["MivubiEdu02"],
    ["mivubiedu02"],
    ["OtherBox", "otherbox"],
    ["Bad Box"],
    ["../DefaultBox"],
    Array.from({length:257}, (_,i) => "Box" + i),
  ]) {
    assert.throws(() => planOwnedMinecraftVirtual("02", existing));
  }
  assert.deepEqual(
    planOwnedMinecraftVirtual("03", ["MivubiEdu02"]).profiles.map(x => x.client), ["03"]);
});

test("bulk compatibility plan stays identical to three independently planned boxes", () => {
  const bulk = planOwnedMinecraftProfiles(["OtherBox"]);
  const individual = ["02", "03", "04"]
    .flatMap(client => planOwnedMinecraftVirtual(client, ["OtherBox"]).commands);
  assert.deepEqual(bulk.commands, individual);
  assert.equal(bulk.commands.length, 27);
});

test("owned configuration plans verify every setting before advancing or enabling", () => {
  for (const client of ["02", "03", "04"]) {
    const result = planOwnedMinecraftVirtual(client, []);
    assert.equal(result.commands.length, 9, "include box-only data in the write plan");
    assert.equal(result.guardedSteps.length, 20, "disable/readback then nine write/readback pairs");
    assert.equal(result.stopOnReadbackMismatch, true);
    assert.equal(result.automaticDeletionPermitted, false);
    assert.equal(result.partialConfigurationRequiresManualReview, true);
    assert.equal(result.settingsReadbackVerified, false);
    assert.equal(result.driverInstalled, false);
    assert.deepEqual(result.guardedSteps[0].arguments,
      ["set", "MivubiEdu" + client, "Enabled", "n"]);
    assert.deepEqual(result.guardedSteps[1].arguments,
      ["query", "MivubiEdu" + client, "Enabled"]);
    assert.equal(result.guardedSteps[1].expectedValue, "n");
    for (let i = 0; i < 9; i++) {
      const write = result.guardedSteps[2 + i * 2];
      const read = result.guardedSteps[3 + i * 2];
      assert.equal(write.operation, "SET");
      assert.equal(read.operation, "REQUIRE_READBACK");
      assert.equal(write.client, client);
      assert.equal(read.client, client);
      assert.equal(write.executable, "SbieIni.exe");
      assert.equal(read.executable, "SbieIni.exe");
      assert.deepEqual(read.arguments, ["query", "MivubiEdu" + client, write.arguments[2]]);
      assert.equal(read.expectedValue, write.arguments[3]);
      assert.deepEqual(result.commands[i].arguments, write.arguments);
    }
    assert.equal(result.guardedSteps[18].arguments[2], "Enabled");
    assert.equal(result.guardedSteps[18].arguments[3], "y");
    assert.equal(result.guardedSteps[19].expectedValue, "y");
  }
  const bulk = planOwnedMinecraftProfiles([]);
  assert.equal(bulk.guardedSteps.length, 60);
  const singles = ["02", "03", "04"].flatMap(
    client => planOwnedMinecraftVirtual(client, []).guardedSteps);
  assert.deepEqual(bulk.guardedSteps, singles);
  assert.equal(bulk.stopOnReadbackMismatch, true);
  assert.equal(bulk.automaticDeletionPermitted, false);
});

test("owned readback comparison accepts only complete ordered matching values", () => {
  for (const client of ["02", "03", "04"]) {
    const plan = planOwnedMinecraftVirtual(client, []);
    // Final-state validation excludes the initial disable/readback pair:
    // the final observable Enabled value is y, not the historical n.
    const readings = plan.guardedSteps.slice(2)
      .filter(step => step.operation === "REQUIRE_READBACK")
      .map(step => ({
        client, box: step.arguments[1], key: step.arguments[2],
        value: step.expectedValue,
      }));
    assert.equal(matchesOwnedMinecraftReadback(client, readings), true);
    // A forged record with inherited required values must not be admitted
    // just because it happens to have four unrelated own properties.
    const inherited = Object.assign(Object.create({...readings[0]}), {
      noise1: 1, noise2: 2, noise3: 3, noise4: 4,
    });
    assert.equal(matchesOwnedMinecraftReadback(client,
      [inherited, ...readings.slice(1)]), false);
    const oneInherited = Object.assign(Object.create({client}), {
      box: readings[0].box, key: readings[0].key,
      value: readings[0].value, unrelated: true,
    });
    assert.equal(matchesOwnedMinecraftReadback(client,
      [oneInherited, ...readings.slice(1)]), false);
    for (const corrupt of [
      rows => rows.pop(),
      rows => rows.reverse(),
      rows => { rows[0].value = "UNKNOWN"; },
      rows => { rows[0].key = rows[1].key; },
      rows => { rows[0].box = "MinecraftClient02"; },
      rows => { rows[0].client = client === "03" ? "02" : "03"; },
      rows => { rows[0].privateAccount = "never accepted"; },
      rows => { rows[0].value = true; },
    ]) {
      const changed = structuredClone(readings);
      corrupt(changed);
      assert.equal(matchesOwnedMinecraftReadback(client, changed), false);
    }
    for (const invalid of [undefined, null, {}, "MATCH", [], false]) {
      assert.equal(matchesOwnedMinecraftReadback(client, invalid), false);
    }
  }
  assert.throws(() => matchesOwnedMinecraftReadback("05", []));
});

test("no mid-setup ENABLE or cleanup/retry is permitted in the source-only plan", () => {
  for (const client of ["02", "03", "04"]) {
    const steps = planOwnedMinecraftVirtual(client, []).guardedSteps;
    const writes = steps.filter(step => step.operation === "SET");
    assert.deepEqual(writes.filter(step => step.arguments[2] === "Enabled")
      .map(step => step.arguments[3]), ["n", "y"]);
    assert.equal(writes[0].arguments[2], "Enabled");
    assert.equal(writes[0].arguments[3], "n");
    assert.equal(writes.at(-1).arguments[2], "Enabled");
    assert.equal(writes.at(-1).arguments[3], "y");
    assert.ok(steps.every(step => step.executable === "SbieIni.exe" &&
      (step.operation === "SET" || step.operation === "REQUIRE_READBACK")));
    const plan = planOwnedMinecraftVirtual(client, []);
    assert.equal(plan.hostChanged, false);
    assert.equal(plan.driverInstalled, false);
    assert.equal(plan.accountIsolationVerified, false);
    assert.equal(plan.automaticDeletionPermitted, false);
    assert.equal(plan.partialConfigurationRequiresManualReview, true);
  }
});

test("partial owned-box plans remain disabled until all other readbacks complete", () => {
  for (const client of ["02", "03", "04"]) {
    const plan = planOwnedMinecraftVirtual(client, ["MinecraftClient02"]);
    const steps = plan.guardedSteps;
    // Failure after any earlier operation must not have requested Enabled=y.
    for (let interruptedAt = 0; interruptedAt <= 18; interruptedAt++) {
      const attempted = steps.slice(0, interruptedAt);
      const enabledWrites = attempted.filter(step =>
        step.operation === "SET" &&
        step.arguments[2] === "Enabled" && step.arguments[3] === "y");
      assert.equal(enabledWrites.length, 0,
        "interrupted setup must not enable a partially configured box");
      assert.ok(attempted.every(step => step.arguments[1] === "MivubiEdu" + client));
      assert.equal(plan.automaticDeletionPermitted, false);
      assert.equal(plan.partialConfigurationRequiresManualReview, true);
    }
    // A failed last readback is ambiguous: Enabled=y may have been written.
    // Only manual reconciliation, never an automatic delete/retry, is permitted.
    assert.equal(steps[18].operation, "SET");
    assert.equal(steps[18].arguments[3], "y");
    assert.equal(steps[19].operation, "REQUIRE_READBACK");
    assert.equal(steps[19].expectedValue, "y");
    assert.equal(plan.hostChanged, false);
  }
});
