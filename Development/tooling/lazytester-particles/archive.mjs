// Dependency-free deterministic ZIP store writer for a tiny, opt-in .mcpack.
// No runtime hooks or game/profile file access.
const CRC_TABLE = Uint32Array.from({length: 256}, (_, index) => {
  let value = index;
  for (let k = 0; k < 8; k++) value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
  return value >>> 0;
});
function crc32(bytes) {
  let value = 0xffffffff;
  for (const byte of bytes) value = CRC_TABLE[(value ^ byte) & 0xff] ^ (value >>> 8);
  return (value ^ 0xffffffff) >>> 0;
}
export function createMcpack(files) {
  if (!(files instanceof Map) || files.size < 2 || files.size > 16 || !files.has('manifest.json')) {
    throw new Error('Unsupported LazyTester pack file list');
  }
  let offset = 0;
  const local = [], central = [];
  for (const [path, value] of [...files].sort(([a], [b]) => a.localeCompare(b, 'en'))) {
    if (!/^[a-zA-Z0-9_-]+(?:\/[a-zA-Z0-9_-]+)*\.json$/.test(path)) throw new Error('Unsafe pack path');
    const name = Buffer.from(path, 'utf8'), data = Buffer.from(value);
    if (data.length > 128 * 1024) throw new Error('Oversized particle definition');
    const checksum = crc32(data), a = Buffer.alloc(30), b = Buffer.alloc(46);
    a.writeUInt32LE(0x04034b50, 0); a.writeUInt16LE(20, 4); a.writeUInt16LE(0x800, 6);
    a.writeUInt32LE(checksum, 14); a.writeUInt32LE(data.length, 18); a.writeUInt32LE(data.length, 22);
    a.writeUInt16LE(name.length, 26);
    b.writeUInt32LE(0x02014b50, 0); b.writeUInt16LE(20, 4); b.writeUInt16LE(20, 6);
    b.writeUInt16LE(0x800, 8); b.writeUInt32LE(checksum, 16);
    b.writeUInt32LE(data.length, 20); b.writeUInt32LE(data.length, 24);
    b.writeUInt16LE(name.length, 28); b.writeUInt32LE(offset, 42);
    local.push(a, name, data); central.push(b, name);
    offset += a.length + name.length + data.length;
  }
  const directory = Buffer.concat(central);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(files.size, 8); end.writeUInt16LE(files.size, 10);
  end.writeUInt32LE(directory.length, 12); end.writeUInt32LE(offset, 16);
  return Buffer.concat([...local, directory, end]);
}
