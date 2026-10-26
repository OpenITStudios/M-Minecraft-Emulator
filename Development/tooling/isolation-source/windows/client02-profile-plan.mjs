// Adapter only: settings and occupied-name policy belong to the existing owner.
import {readFileSync} from 'node:fs';
import {planOwnedMinecraftVirtual} from '../../../engine/isolation-native/owned-runtime/minecraft-profiles.mjs';
const inventory=JSON.parse(readFileSync(process.argv[2],'utf8').replace(/^\uFEFF/,''));
if(inventory.schema!==1 || !Array.isArray(inventory.boxes)) throw Error('Invalid observed box inventory');
process.stdout.write(JSON.stringify(planOwnedMinecraftVirtual(process.argv[3] ?? '02',inventory.boxes)));
