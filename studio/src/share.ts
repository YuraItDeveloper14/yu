// A program travels in the link: #code= and the base64url of its deflate-raw bytes.

async function through(
  bytes: Uint8Array<ArrayBuffer>,
  stream: CompressionStream | DecompressionStream,
): Promise<Uint8Array<ArrayBuffer>> {
  const piped = new Blob([bytes]).stream().pipeThrough(stream);
  return new Uint8Array(await new Response(piped).arrayBuffer());
}

function toBase64Url(bytes: Uint8Array): string {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '');
}

function fromBase64Url(token: string): Uint8Array<ArrayBuffer> {
  const binary = atob(token.replaceAll('-', '+').replaceAll('_', '/'));
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

export async function encode(code: string): Promise<string> {
  const packed = await through(new TextEncoder().encode(code), new CompressionStream('deflate-raw'));
  return toBase64Url(packed);
}

export async function decode(token: string): Promise<string> {
  const bytes = await through(fromBase64Url(token), new DecompressionStream('deflate-raw'));
  return new TextDecoder('utf-8', { fatal: true }).decode(bytes);
}

/** The token of a `#code=…` link, or null for any other hash. */
export function tokenFromHash(hash: string): string | null {
  return /^#code=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}
