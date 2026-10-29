"""Finite synthetic Ghast/PBR fixture; no Mojang binary assets are bundled."""
import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

from PIL import Image

from build import compile_preview, pinned_tree, build
from entity_preview import GHAST_PINS, IMAGES, compile_ghast, git_blob_sha


def image_bytes(fmt: str) -> bytes:
    image = Image.new("RGBA", (128, 64))
    for y in range(64):
        for x in range(128):
            image.putpixel((x, y), (255 if x % 2 else 0, 64 if y % 2 else 0, 100, 128))
    output = io.BytesIO()
    image.save(output, format=fmt)
    return output.getvalue()


def source_fixture(root: Path):
    source = root / 'resource_pack'
    def write(path, value):
        target = source / path
        target.parent.mkdir(exist_ok=True, parents=True)
        raw = value if isinstance(value, bytes) else json.dumps(value).encode()
        target.write_bytes(raw)
        return raw
    desc = {
        'identifier':'minecraft:ghast',
        'textures':{'default':'textures/entity/ghast/ghast','shooting':'textures/entity/ghast/ghast_shooting'},
        'geometry':{'default':'geometry.ghast'},
        'render_controllers':['controller.render.ghast']}
    geo = {
        'format_version':'1.21.0',
        'minecraft:geometry':[{'description':{'identifier':'geometry.ghast','texture_width':64,'texture_height':32},
            'bones':[{'name':'body','cubes':[{'uv':{side:{'uv':[0,0],'uv_size':[16,16]} for side in ['north','south','east','west','up','down']}}]}]}]}
    rc = {'render_controllers':{'controller.render.ghast':{
        'geometry':'Geometry.default',
        'textures':['Array.skins[variable.ischarging]'],
        'arrays':{'textures':{'Array.skins':['Texture.default','Texture.shooting']}}}}}
    pins = {}
    for p, value in [
        ('entity/ghast.entity.json',{'minecraft:client_entity':{'description':desc}}),
        ('models/entity/ghast.geo.json',geo),
        ('render_controllers/ghast.render_controllers.json',rc),
    ]:
        pins[p] = git_blob_sha(write(p,value))
    for name in ('ghast','ghast_shooting'):
        p=f'textures/entity/ghast/{name}.texture_set.json'
        pins[p]=git_blob_sha(write(p, {'minecraft:texture_set':{
            'color':name,'metalness_emissive_roughness_subsurface': name+'_mers'}}))
    for p in IMAGES:
        pins[p] = git_blob_sha(write(p,image_bytes('PNG' if p.endswith('.png') else 'TGA')))
    self_block=source/'textures'/'blocks'
    self_block.mkdir(exist_ok=True)
    stone=Image.new('RGBA',(16,16),(40,90,100,255))
    buf=io.BytesIO(); stone.save(buf,format='PNG')
    (self_block/'stone.png').write_bytes(buf.getvalue())
    return source,pins,geo,rc


class EntityPreviewTests(unittest.TestCase):
    def test_all_five_sources_and_uv_chain_bounded_before_conversion(self):
        with tempfile.TemporaryDirectory() as tmp:
            src,pins,_,_=source_fixture(Path(tmp))
            result,receipt=compile_ghast(src,pins)
            self.assertEqual(set(result),set(IMAGES))
            self.assertEqual(receipt['skin_variants'],2)
            self.assertEqual(receipt['image_overrides'],5)
            self.assertEqual(receipt['validated_uv_faces'],6)
            self.assertEqual(receipt['hypothetical_rgba8_difference_bytes'],122_880)
            self.assertEqual(receipt['runtime_ram_saving'],'NOT_MEASURED')
            for path,raw in result.items():
                with Image.open(io.BytesIO(raw)) as i:
                    self.assertEqual(i.size,(64,32))
                    self.assertEqual(i.mode,'RGBA')
                    self.assertEqual(i.format,'PNG' if path.endswith('png') else 'TGA')

    def test_combined_archive_reuses_original_texture_pack_not_a_new_service(self):
        with tempfile.TemporaryDirectory() as tmp:
            src,pins,_,_=source_fixture(Path(tmp))
            extra,_=compile_ghast(src,pins)
            block=src/'textures'/'blocks'
            tree,_=pinned_tree(block)
            data,receipt=compile_preview(block,tree,extra)
            self.assertEqual(data,compile_preview(block,tree,extra)[0])
            self.assertEqual(receipt['converted'],1)
            with zipfile.ZipFile(io.BytesIO(data)) as z:
                self.assertEqual(len(z.namelist()),7)
                self.assertEqual(json.loads(z.read('manifest.json'))['header']['version'],[0,2,0])
                self.assertEqual(z.testzip(),None)
                self.assertFalse(any(n.startswith('models/') or n.startswith('entity/') or n.startswith('render_controllers/') for n in z.namelist()))
            default,_=compile_preview(block,tree)
            with zipfile.ZipFile(io.BytesIO(default)) as z:
                self.assertEqual(z.namelist(),['manifest.json','textures/blocks/stone.png'])
                self.assertEqual(json.loads(z.read('manifest.json'))['header']['version'],[0,1,0])

    def test_changed_source_refuses_before_output_or_pack_mutation(self):
        with tempfile.TemporaryDirectory() as tmp:
            src,pins,_,_=source_fixture(Path(tmp))
            target=src/'models/entity/ghast.geo.json'
            target.write_bytes(target.read_bytes()+b'\n')
            with self.assertRaisesRegex(ValueError,'fingerprint mismatch'):
                compile_ghast(src,pins)
            with self.assertRaisesRegex(ValueError,'fingerprint mismatch'):
                compile_ghast(src,GHAST_PINS)
            out=Path(tmp)/'not-created.mcpack'
            with self.assertRaises(ValueError):
                build(['--source',str(src),'--output',str(out),'--with-ghast-entity'])
            self.assertFalse(out.exists())

    def test_unknown_uv_bounds_and_render_controller_refused_even_if_rehashed(self):
        with tempfile.TemporaryDirectory() as tmp:
            src,pins,geo,rc=source_fixture(Path(tmp))
            p='models/entity/ghast.geo.json'
            geo['minecraft:geometry'][0]['bones'][0]['cubes'][0]['uv']['north']['uv']=[61,2]
            raw=json.dumps(geo).encode(); (src/p).write_bytes(raw)
            pins[p]=git_blob_sha(raw)
            with self.assertRaisesRegex(ValueError,'UV exceeds'):
                compile_ghast(src,pins)
            geo['minecraft:geometry'][0]['bones'][0]['cubes'][0]['uv']['north']['uv']=[0,2]
            raw=json.dumps(geo).encode(); (src/p).write_bytes(raw); pins[p]=git_blob_sha(raw)
            rc['render_controllers']['controller.render.ghast']['textures']=['Texture.unknown']
            p='render_controllers/ghast.render_controllers.json';raw=json.dumps(rc).encode()
            (src/p).write_bytes(raw);pins[p]=git_blob_sha(raw)
            with self.assertRaisesRegex(ValueError,'render controller'):
                compile_ghast(src,pins)

    def test_symlinks_and_unknown_extra_entries_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            src,pins,_,_=source_fixture(Path(tmp))
            p=src/'textures/entity/ghast/ghast.png';p.unlink()
            try:
                p.symlink_to(src/'textures/entity/ghast/ghast_shooting.png')
            except (OSError,NotImplementedError):
                self.skipTest('Symlink not permitted')
            with self.assertRaisesRegex(ValueError,'Linked'):
                compile_ghast(src,pins)
            tree,_=pinned_tree(src/'textures'/'blocks')
            with self.assertRaisesRegex(ValueError,'Only the pinned Ghast'):
                compile_preview(src/'textures'/'blocks',tree,{'textures/entity/invalid.png':b'broken'})


if __name__ == '__main__':
    unittest.main()
