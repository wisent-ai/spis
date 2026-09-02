#!/usr/bin/env node

import { createHash, randomUUID } from 'node:crypto';
import {
  constants as fsConstants,
  closeSync,
  createReadStream,
  fstatSync,
  fsyncSync,
  lstatSync,
  openSync,
  readFileSync,
  readSync,
  realpathSync,
  linkSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import { open } from 'node:fs/promises';
import { basename, dirname, isAbsolute, join, normalize, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const CLIENT_PACKAGE = '@wisent-ai/weles-client';
const CLIENT_COMMIT = '37798a26022a040fbd0a4a4a25c99b5559d95a32';
const CLIENT_SOURCE_SHA256 = '7cdfee8ae7d7ffc831c60d01e393640bf912d95adf0b06c9dd51a737f97ccada';
const CONFIG_SCHEMA = 'wisent.spis-weles-bridge-config.v1';
const TRUST_SCHEMA = 'wisent.spis-weles-receipt-trust.v1';
const COMMAND_SCHEMA = 'wisent.spis-weles-bridge-command.v1';
const PROVENANCE_SCHEMA = 'wisent.spis-weles-provenance.v1';
const RECEIPT_CHECKPOINT_SCHEMA = 'wisent.spis-weles-receipt-checkpoint.v1';
const SUBMISSION_SCHEMA = 'wisent.spis-weles-submission.v1';
const TASK_STATUS_SCHEMA = 'wisent.spis-weles-task-status.v1';
const CANCELLATION_SCHEMA = 'wisent.spis-weles-cancellation.v1';
const ERROR_SCHEMA = 'wisent.spis-weles-bridge-error.v1';
const MAX_JSON_BYTES = 1024 * 1024;
const MAX_TRUST_BYTES = 64 * 1024;
const MAX_OUTPUT_BYTES = 4 * 1024 * 1024;
const MAX_EVIDENCE_MANIFEST_BYTES = 4 * 1024 * 1024;
const MAX_EVIDENCE_INVENTORY_BYTES = 8 * 1024 * 1024;
const SHA256 = /^[0-9a-f]{64}$/;
const SHA256_ID = /^sha256:[0-9a-f]{64}$/;
const GIT_REVISION = /^[0-9a-f]{40}$/;
const SPIS_BINDING_SCHEMA = 'weles.spis-browser-evidence-binding.v1';
const NONTERMINAL_STATUSES = new Set(['queued', 'leased', 'running', 'pending_review']);
const TERMINAL_OUTCOME_BY_STATUS = new Map([
  ['completed', 'completed'],
  ['succeeded', 'completed'],
  ['failed', 'failed'],
  ['cancelled', 'cancelled'],
  ['canceled', 'cancelled'],
  ['rejected', 'rejected'],
]);
// Every outcome the service signs into a receipt. `completed` is the only one that can
// carry browser evidence; the others are signed proofs of a terminal non-success, and
// each of them is retained with its own manifest version below.
const TERMINAL_OUTCOMES = new Set(TERMINAL_OUTCOME_BY_STATUS.values());
const EVIDENCE_MANIFEST_SCHEMA = 'weles.browser-evidence-manifest.v1';
const NON_SUCCESS_EVIDENCE_MANIFEST_SCHEMA = 'weles.browser-evidence-manifest.v2';
// The service writes exactly these keys, canonically ordered, and nothing else:
// `finalize` builds the manifest with effectiveUrl/finalUrl spread in only for a
// succeeded task. Two exact lists, never one relaxed list, so a v1 manifest missing its
// navigation URLs and a v2 manifest carrying them are both refused.
const EVIDENCE_MANIFEST_COMMON_KEYS = Object.freeze([
  'schema',
  'taskId',
  'organizationId',
  'origin',
  'action',
  'outcome',
  'requestDigest',
  'resultDigest',
  'spisBinding',
  'requestedUrl',
  'evidenceInventory',
]);
const EVIDENCE_MANIFEST_KEYS = Object.freeze([
  ...EVIDENCE_MANIFEST_COMMON_KEYS,
  'effectiveUrl',
  'finalUrl',
]);
const NON_SUCCESS_EVIDENCE_MANIFEST_KEYS = EVIDENCE_MANIFEST_COMMON_KEYS;
const CORE_CLAIMS = Object.freeze([
  'taskId',
  'organizationId',
  'origin',
  'action',
  'outcome',
  'evidenceDigest',
]);
const KNOWN_TASK_FIELDS = Object.freeze([
  'taskId',
  'organizationId',
  'origin',
  'action',
]);
const SERVICE_IDENTITY_FIELDS = Object.freeze([
  'name',
  'generation',
  'consumer',
  'capability',
  'active_host',
  'endpoint',
  'action',
  'release_id',
  'source_revision',
]);
const SPIS_BINDING_FIELDS = Object.freeze([
  'schema',
  'run_id',
  'catalog',
  'record',
  'record_key',
  'attempt',
  'attempt_id',
  'source_revision',
  'source_input_sha256',
  'reference_sha256',
  'artifact_uri',
  'output_uri',
  'service',
]);
const SPIS_BINDING_SERVICE_FIELDS = Object.freeze([
  'name',
  'consumer',
  'capability',
  'directory_generation',
  'host',
  'endpoint',
  'action',
  'release_id',
  'source_revision',
]);
const SIGNED_SPIS_CLAIMS = Object.freeze([
  'requestDigest',
  'resultDigest',
  'spisBinding',
]);
const RECEIPT_FIELDS = Object.freeze([
  'schema',
  ...CORE_CLAIMS,
  ...SIGNED_SPIS_CLAIMS,
  'keyId',
  'signature',
  'signedPayload',
]);

class BridgeError extends Error {
  constructor(code, message) {
    super(message);
    this.name = 'BridgeError';
    this.code = code;
  }
}

function fail(code, message) {
  throw new BridgeError(code, message);
}

const VERIFIED_DATA_EXECUTION = import.meta.url.startsWith('data:text/javascript;base64,');

function resolveBridgeDirectory() {
  if (!VERIFIED_DATA_EXECUTION) {
    return dirname(fileURLToPath(import.meta.url));
  }
  const configured = process.env.SPIS_WELES_VERIFIED_BRIDGE_DIR;
  if (!configured || !isAbsolute(configured)) {
    fail(
      'official-client-unavailable',
      'verified in-memory bridge execution requires an absolute checked-in resource directory',
    );
  }
  let resolved;
  let stat;
  try {
    resolved = realpathSync(configured);
    stat = lstatSync(resolved);
  } catch {
    fail('official-client-unavailable', 'the checked-in bridge resource directory is unavailable');
  }
  if (resolved !== configured || !stat.isDirectory() || stat.isSymbolicLink()) {
    fail(
      'official-client-unavailable',
      'the checked-in bridge resource directory must be canonical and non-symlinked',
    );
  }
  return resolved;
}

const BRIDGE_DIRECTORY = resolveBridgeDirectory();
const CANONICAL_TRUST_PATH = join(BRIDGE_DIRECTORY, 'weles-receipt-trust.json');

function plainObject(value, name) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    fail('invalid-input', `${name} must be an object`);
  }
  return value;
}

function nonemptyString(value, name) {
  if (typeof value !== 'string' || !value.trim()) {
    fail('invalid-input', `${name} must be a non-empty string`);
  }
  return value;
}

function possiblyEmptyStringArray(value, name) {
  if (!Array.isArray(value)) {
    fail('invalid-input', `${name} must be a string array`);
  }
  const entries = value.map((entry, index) => nonemptyString(entry, `${name}[${index}]`));
  if (new Set(entries).size !== entries.length) {
    fail('invalid-input', `${name} must not contain duplicates`);
  }
  return entries;
}

function onlyKeys(value, allowed, name) {
  const unknown = Object.keys(value).filter(key => !allowed.includes(key));
  if (unknown.length !== 0) {
    fail('invalid-input', `${name} contains unsupported fields`);
  }
}
function exactOrigin(value, name) {
  const origin = nonemptyString(value, name);
  let parsed;
  try {
    parsed = new URL(origin);
  } catch {
    fail('invalid-input', `${name} must be an exact HTTP(S) origin`);
  }
  if (!['http:', 'https:'].includes(parsed.protocol)
      || parsed.origin !== origin
      || parsed.pathname !== '/'
      || parsed.search
      || parsed.hash
      || parsed.username
      || parsed.password) {
    fail('invalid-input', `${name} must be an exact HTTP(S) origin`);
  }
  return origin;
}

function validateServiceIdentity(value, name) {
  const identity = plainObject(value, name);
  onlyKeys(identity, SERVICE_IDENTITY_FIELDS, name);
  const validated = {};
  for (const field of SERVICE_IDENTITY_FIELDS) {
    if (field === 'generation') {
      if (!Number.isSafeInteger(identity[field]) || identity[field] < 0) {
        fail('invalid-input', `${name}.generation must be a non-negative integer`);
      }
      validated[field] = identity[field];
    } else {
      validated[field] = nonemptyString(identity[field], `${name}.${field}`);
    }
  }
  validated.endpoint = canonicalApiEndpoint(validated.endpoint, `${name}.endpoint`);
  if (validated.name !== 'weles-admission'
      || validated.consumer !== 'spis'
      || validated.capability !== 'browser-evidence'
      || validated.action !== 'generic_browser_task'
      || !validated.release_id.startsWith('weles-worker@')
      || validated.release_id === 'weles-worker@'
      || !GIT_REVISION.test(validated.source_revision)) {
    fail('invalid-input', `${name} is not the exact Weles browser-evidence service identity`);
  }
  return validated;
}

function validateSpisBinding(value, name) {
  const binding = plainObject(value, name);
  onlyKeys(binding, SPIS_BINDING_FIELDS, name);
  if (binding.schema !== SPIS_BINDING_SCHEMA) {
    fail('invalid-input', `${name}.schema is unsupported`);
  }
  const validated = { schema: binding.schema };
  for (const field of SPIS_BINDING_FIELDS.slice(1)) {
    if (field === 'attempt') {
      if (!Number.isSafeInteger(binding[field]) || binding[field] < 1) {
        fail('invalid-input', `${name}.attempt must be a positive integer`);
      }
      validated[field] = binding[field];
    } else if (field === 'service') {
      const service = plainObject(binding.service, `${name}.service`);
      onlyKeys(service, SPIS_BINDING_SERVICE_FIELDS, `${name}.service`);
      const validatedService = {};
      for (const serviceField of SPIS_BINDING_SERVICE_FIELDS) {
        if (serviceField === 'directory_generation') {
          if (!Number.isSafeInteger(service[serviceField]) || service[serviceField] < 0) {
            fail('invalid-input', `${name}.service.directory_generation must be a non-negative integer`);
          }
          validatedService[serviceField] = service[serviceField];
        } else {
          validatedService[serviceField] = nonemptyString(
            service[serviceField],
            `${name}.service.${serviceField}`,
          );
        }
      }
      validatedService.endpoint = canonicalApiEndpoint(
        validatedService.endpoint,
        `${name}.service.endpoint`,
      );
      if (validatedService.name !== 'weles-admission'
          || validatedService.consumer !== 'spis'
          || validatedService.capability !== 'browser-evidence'
          || validatedService.action !== 'generic_browser_task'
          || !validatedService.release_id.startsWith('weles-worker@')
          || validatedService.release_id === 'weles-worker@'
          || !GIT_REVISION.test(validatedService.source_revision)) {
        fail('invalid-input', `${name}.service is not the exact Weles browser-evidence identity`);
      }
      validated.service = validatedService;
    } else {
      validated[field] = nonemptyString(binding[field], `${name}.${field}`);
    }
  }
  if (!SHA256.test(validated.source_input_sha256)
      || !SHA256.test(validated.reference_sha256)
      || !GIT_REVISION.test(validated.source_revision)) {
    fail('invalid-input', `${name} source revision/digest fields are invalid`);
  }
  validateAttemptBindingUris(validated, name);
  return validated;
}

function validateRequestIdentity(value, name) {
  const identity = plainObject(value, name);
  onlyKeys(identity, ['requestDigest', 'spisBinding'], name);
  const requestDigest = nonemptyString(identity.requestDigest, `${name}.requestDigest`);
  if (!SHA256_ID.test(requestDigest)) {
    fail('invalid-input', `${name}.requestDigest must be a sha256: identifier`);
  }
  return {
    requestDigest,
    spisBinding: validateSpisBinding(identity.spisBinding, `${name}.spisBinding`),
  };
}



function parseJson(text, name) {
  try {
    return JSON.parse(text);
  } catch {
    fail('invalid-json', `${name} is not valid JSON`);
  }
}

async function readStdin() {
  const chunks = [];
  let bytes = 0;
  for await (const chunk of process.stdin) {
    bytes += chunk.length;
    if (bytes > MAX_JSON_BYTES) fail('input-too-large', 'bridge input exceeded the size limit');
    chunks.push(chunk);
  }
  return Buffer.concat(chunks).toString('utf8');
}

function readBoundedFile(path, name) {
  let fd;
  try {
    const pathStat = lstatSync(path);
    if (!pathStat.isFile() || pathStat.isSymbolicLink()) {
      fail('invalid-input-file', `${name} must be a regular non-symlink file`);
    }
    fd = openSync(path, fsConstants.O_RDONLY | (fsConstants.O_NOFOLLOW ?? 0));
    const openedStat = fstatSync(fd);
    if (!openedStat.isFile()
        || openedStat.dev !== pathStat.dev
        || openedStat.ino !== pathStat.ino) {
      fail('invalid-input-file', `${name} changed during open`);
    }
    const bytes = Buffer.allocUnsafe(MAX_JSON_BYTES + 1);
    let offset = 0;
    while (offset < bytes.length) {
      const count = readSync(fd, bytes, offset, bytes.length - offset, null);
      if (count === 0) break;
      offset += count;
    }
    if (offset > MAX_JSON_BYTES) fail('input-too-large', `${name} exceeded the size limit`);
    return bytes.subarray(0, offset).toString('utf8');
  } catch (error) {
    if (error instanceof BridgeError) throw error;
    fail('invalid-input-file', `${name} could not be read`);
  } finally {
    if (fd !== undefined) {
      try { closeSync(fd); } catch {}
    }
  }
}

function readProtectedConfig(path) {
  let stat;
  try {
    stat = lstatSync(path);
  } catch {
    fail('config-unavailable', 'protected Weles config could not be read');
  }
  if (!stat.isFile() || stat.isSymbolicLink()) {
    fail('config-unprotected', 'protected Weles config must be a regular file, not a symlink');
  }
  if (typeof process.getuid !== 'function' || stat.uid !== process.getuid() || (stat.mode & 0o077) !== 0) {
    fail('config-unprotected', 'protected Weles config must be owned by this user with mode 0600 or stricter');
  }
  if (stat.size > MAX_JSON_BYTES) fail('config-too-large', 'protected Weles config exceeded the size limit');
  try {
    return parseJson(readFileSync(path, 'utf8'), 'protected Weles config');
  } catch (error) {
    if (error instanceof BridgeError) throw error;
    fail('config-unavailable', 'protected Weles config could not be read');
  }
}
function readPublicTrust(path) {
  let stat;
  try {
    stat = lstatSync(path);
  } catch {
    fail('trust-unavailable', 'public Weles receipt trust document could not be read');
  }
  if (!stat.isFile() || stat.isSymbolicLink()) {
    fail('invalid-trust', 'public Weles receipt trust must be a regular file, not a symlink');
  }
  if (stat.size > MAX_JSON_BYTES) fail('invalid-trust', 'public Weles receipt trust exceeded the size limit');
  try {
    return parseJson(readFileSync(path, 'utf8'), 'public Weles receipt trust');
  } catch (error) {
    if (error instanceof BridgeError) throw error;
    fail('trust-unavailable', 'public Weles receipt trust document could not be read');
  }
}

function loadTrust() {
  const configuredPath = process.env.SPIS_WELES_TRUST_FILE;
  if (!configuredPath) fail('trust-unavailable', 'SPIS_WELES_TRUST_FILE is required');
  if (resolve(configuredPath) !== resolve(CANONICAL_TRUST_PATH)) {
    fail('invalid-trust', 'SPIS_WELES_TRUST_FILE must resolve to the checked-in canonical trust document');
  }
  let trustValue;
  if (VERIFIED_DATA_EXECUTION) {
    const encoded = process.env.SPIS_WELES_VERIFIED_TRUST_BASE64;
    if (typeof encoded !== 'string' || !encoded) {
      fail('trust-unavailable', 'verified in-memory execution requires the checked-in public trust bytes');
    }
    const bytes = Buffer.from(encoded, 'base64');
    if (bytes.toString('base64') !== encoded || bytes.length > MAX_TRUST_BYTES) {
      fail('invalid-trust', 'verified public trust bytes are malformed or oversized');
    }
    trustValue = parseJson(bytes.toString('utf8'), 'verified public Weles receipt trust');
  } else {
    trustValue = readPublicTrust(CANONICAL_TRUST_PATH);
  }
  const trust = plainObject(trustValue, 'public Weles receipt trust');
  onlyKeys(trust, [
    'schema',
    'organizationId',
    'allowedAction',
    'receiptKeys',
    'keySetVersion',
  ], 'public Weles receipt trust');
  if (trust.schema !== TRUST_SCHEMA) fail('invalid-trust', 'public Weles receipt trust schema is unsupported');
  const organizationId = nonemptyString(trust.organizationId, 'public Weles receipt trust.organizationId');
  const allowedAction = nonemptyString(trust.allowedAction, 'public Weles receipt trust.allowedAction');
  const keySetVersion = nonemptyString(trust.keySetVersion, 'public Weles receipt trust.keySetVersion');
  const receiptKeys = plainObject(trust.receiptKeys, 'public Weles receipt trust.receiptKeys');
  if (Object.keys(receiptKeys).length === 0) fail('invalid-trust', 'public Weles receipt trust.receiptKeys must not be empty');
  for (const [keyId, publicKey] of Object.entries(receiptKeys)) {
    nonemptyString(keyId, 'public Weles receipt trust.receiptKeys key ID');
    nonemptyString(publicKey, `public Weles receipt trust.receiptKeys.${keyId}`);
  }
  return {
    organizationId,
    allowedAction,
    receiptKeys,
    keySetVersion,
  };
}

function loadConfig(trust) {
  const path = process.env.SPIS_WELES_CONFIG_FILE;
  if (!path) fail('config-unavailable', 'SPIS_WELES_CONFIG_FILE is required for network operations');
  const config = plainObject(readProtectedConfig(path), 'config');
  onlyKeys(config, ['schema', 'endpoint', 'bearer', 'organizationId'], 'config');
  if (config.schema !== CONFIG_SCHEMA) fail('invalid-config', 'Weles bridge config schema is unsupported');
  const endpoint = canonicalApiEndpoint(config.endpoint, 'config.endpoint');
  const bearer = nonemptyString(config.bearer, 'config.bearer');
  const organizationId = nonemptyString(config.organizationId, 'config.organizationId');
  if (organizationId !== trust.organizationId) {
    fail('invalid-config', 'protected organizationId differs from public receipt trust');
  }
  return {
    ...trust,
    endpoint,
    bearer,
  };
}

async function loadOfficialClient() {
  const sourcePath = join(
    BRIDGE_DIRECTORY,
    'vendor',
    'weles-client',
    'index.mjs',
  );
  let fd;
  let source;
  try {
    const pathStat = lstatSync(sourcePath);
    if (!pathStat.isFile() || pathStat.isSymbolicLink()) {
      fail(
        'official-client-unavailable',
        'the vendored official Weles client must be a regular non-symlink file',
      );
    }
    if (pathStat.size > MAX_JSON_BYTES) {
      fail('official-client-unavailable', 'the vendored official Weles client is oversized');
    }
    fd = openSync(
      sourcePath,
      fsConstants.O_RDONLY | (fsConstants.O_NOFOLLOW ?? 0),
    );
    const openedStat = fstatSync(fd);
    if (!openedStat.isFile()
        || openedStat.dev !== pathStat.dev
        || openedStat.ino !== pathStat.ino
        || openedStat.size !== pathStat.size) {
      fail('official-client-unavailable', 'the vendored official Weles client changed during open');
    }
    source = readFileSync(fd);
  } catch (error) {
    if (error instanceof BridgeError) throw error;
    fail('official-client-unavailable', 'the vendored official Weles client source is unreadable');
  } finally {
    if (fd !== undefined) {
      try { closeSync(fd); } catch {}
    }
  }
  const digest = createHash('sha256').update(source).digest('hex');
  if (digest !== CLIENT_SOURCE_SHA256) {
    fail('official-client-mismatch', 'the vendored official Weles client does not match the pinned commit');
  }
  try {
    const verifiedModule = `data:text/javascript;base64,${source.toString('base64')}`;
    return await import(verifiedModule);
  } catch {
    fail('official-client-unavailable', 'the verified official Weles client bytes could not be loaded');
  }
}

function parseArgs(argv) {
  const result = { input: '-', output: '-' };
  for (let index = 0; index < argv.length; index += 2) {
    const option = argv[index];
    const value = argv[index + 1];
    if (!value || (option !== '--input' && option !== '--output')) {
      fail('invalid-arguments', 'only --input and --output are accepted');
    }
    result[option.slice(2)] = value;
  }
  return result;
}

function validateExpectedTask(value, config) {
  const expected = plainObject(value, 'expectedTask');
  onlyKeys(expected, KNOWN_TASK_FIELDS, 'expectedTask');
  nonemptyString(expected.taskId, 'expectedTask.taskId');
  nonemptyString(expected.organizationId, 'expectedTask.organizationId');
  exactOrigin(expected.origin, 'expectedTask.origin');
  nonemptyString(expected.action, 'expectedTask.action');
  if (expected.organizationId !== config.organizationId) {
    fail('expected-claim-mismatch', 'expected organizationId differs from public receipt trust');
  }
  if (expected.action !== config.allowedAction) {
    fail('expected-claim-mismatch', 'expected action differs from public receipt trust');
  }
  return expected;
}

function validateExpectedClaims(value, config) {
  const expected = plainObject(value, 'expectedClaims');
  onlyKeys(expected, [...CORE_CLAIMS, ...SIGNED_SPIS_CLAIMS], 'expectedClaims');
  for (const field of CORE_CLAIMS) nonemptyString(expected[field], `expectedClaims.${field}`);
  validateExpectedTask({
    taskId: expected.taskId,
    organizationId: expected.organizationId,
    origin: expected.origin,
    action: expected.action,
  }, config);
  // Spis provenance covers every terminal outcome the service signs, not only the
  // successful one: a failed, cancelled or rejected task is still delivered with a
  // signed receipt and a retained manifest, and that is the proof of its failure.
  if (!TERMINAL_OUTCOMES.has(expected.outcome)) {
    fail('expected-claim-mismatch', 'Spis provenance requires an exact terminal outcome');
  }
  if (!SHA256.test(expected.evidenceDigest)) {
    fail('expected-claim-mismatch', 'expected evidenceDigest must be a lowercase SHA-256 digest');
  }
  for (const field of ['requestDigest', 'resultDigest']) {
    if (!SHA256_ID.test(nonemptyString(expected[field], `expectedClaims.${field}`))) {
      fail('expected-claim-mismatch', `expected ${field} must be a sha256: identifier`);
    }
  }
  expected.spisBinding = validateSpisBinding(
    expected.spisBinding,
    'expectedClaims.spisBinding',
  );
  if (expected.spisBinding.service.action !== config.allowedAction) {
    fail('expected-claim-mismatch', 'expected spisBinding action differs from public receipt trust');
  }
  return expected;
}

function validateReceipt(value) {
  const receipt = plainObject(value, 'receipt');
  onlyKeys(receipt, RECEIPT_FIELDS, 'receipt');
  const retained = {};
  for (const field of [
    'schema',
    ...CORE_CLAIMS,
    'requestDigest',
    'resultDigest',
    'keyId',
    'signature',
    'signedPayload',
  ]) {
    retained[field] = nonemptyString(receipt[field], `receipt.${field}`);
  }
  if (retained.schema !== 'weles.receipt.current') fail('unsupported-receipt', 'receipt schema is unsupported');
  if (!SHA256_ID.test(retained.requestDigest) || !SHA256_ID.test(retained.resultDigest)) {
    fail('invalid-receipt', 'receipt request/result digests must be sha256: identifiers');
  }
  retained.spisBinding = validateSpisBinding(receipt.spisBinding, 'receipt.spisBinding');
  return retained;
}

function validateArtifact(value) {
  const artifact = plainObject(value, 'artifact');
  onlyKeys(artifact, ['path', 'sha256', 'bytes'], 'artifact');
  artifact.path = nonemptyString(artifact.path, 'artifact.path');
  if (isAbsolute(artifact.path) || artifact.path.includes('\\')) {
    fail('invalid-artifact', 'artifact.path must be a portable path relative to the record directory');
  }
  const normalized = normalize(artifact.path);
  if (normalized === '..' || normalized.startsWith(`..${sep}`) || normalized === '.') {
    fail('invalid-artifact', 'artifact.path must remain inside the record directory');
  }
  if (!SHA256.test(nonemptyString(artifact.sha256, 'artifact.sha256'))) {
    fail('invalid-artifact', 'artifact.sha256 must be a lowercase SHA-256 digest');
  }
  if (!Number.isSafeInteger(artifact.bytes)
      || artifact.bytes < 1
      || artifact.bytes > MAX_EVIDENCE_MANIFEST_BYTES) {
    fail('invalid-artifact', 'artifact.bytes must be a required positive bounded safe integer');
  }
  return artifact;
}

async function digestArtifact(artifact) {
  const base = realpathSync(process.cwd());
  const candidate = resolve(base, artifact.path);
  let actual;
  try {
    actual = realpathSync(candidate);
  } catch {
    fail('artifact-unavailable', 'retained artifact could not be resolved');
  }
  if (actual !== base && !actual.startsWith(`${base}${sep}`)) {
    fail('invalid-artifact', 'retained artifact resolves outside the record directory');
  }
  let handle;
  try {
    handle = await open(actual, fsConstants.O_RDONLY | (fsConstants.O_NOFOLLOW ?? 0));
    const stat = await handle.stat();
    if (!stat.isFile()) fail('invalid-artifact', 'retained artifact must be a regular file');
    if (stat.size > MAX_EVIDENCE_MANIFEST_BYTES) {
      fail('artifact-too-large', 'retained evidence manifest exceeded the strict byte limit');
    }
    const hash = createHash('sha256');
    const chunks = [];
    for await (const chunk of createReadStream(null, { fd: handle.fd, autoClose: false })) {
      hash.update(chunk);
      chunks.push(chunk);
    }
    const sha256 = hash.digest('hex');
    if (sha256 !== artifact.sha256) fail('artifact-digest-mismatch', 'retained artifact digest differs from the caller expectation');
    if (stat.size !== artifact.bytes) {
      fail('artifact-size-mismatch', 'retained artifact size differs from the persisted verification document');
    }
    return {
      artifact: { path: artifact.path, sha256, bytes: stat.size },
      value: parseJson(Buffer.concat(chunks).toString('utf8'), 'retained evidence manifest'),
    };
  } catch (error) {
    if (error instanceof BridgeError) throw error;
    fail('artifact-unavailable', 'retained artifact could not be read');
  } finally {
    await handle?.close().catch(() => {});
  }
}

function updateFramed(hash, label, value) {
  hash.update(String(Buffer.byteLength(label, 'utf8')));
  hash.update(':');
  hash.update(label);
  hash.update(String(Buffer.byteLength(value, 'utf8')));
  hash.update(':');
  hash.update(value);
}

function provenanceId(receipt, keySetVersion, artifact) {
  const hash = createHash('sha256');
  for (const [label, value] of [
    ['receipt.schema', receipt.schema],
    ['receipt.keyId', receipt.keyId],
    ['receipt.signedPayload', receipt.signedPayload],
    ['receipt.signature', receipt.signature],
    ['receipt.requestDigest', receipt.requestDigest],
    ['receipt.resultDigest', receipt.resultDigest],
    ['receipt.spisBinding', canonicalJson(receipt.spisBinding)],
    ['keySetVersion', keySetVersion],
    ['artifact.path', artifact.path],
    ['artifact.sha256', artifact.sha256],
  ]) updateFramed(hash, label, value);
  return `sha256:${hash.digest('hex')}`;
}

function buildReceiptCheckpoint(receiptValue, expectedTaskValue, config, verifyReceipt) {
  const receipt = validateReceipt(receiptValue);
  const expectedTask = validateExpectedTask(expectedTaskValue, config);
  let claims;
  try {
    claims = verifyReceipt(receipt, config.receiptKeys);
  } catch (error) {
    const code = typeof error?.code === 'string' ? error.code : 'receipt-verification-failed';
    fail(code, 'the official Weles client rejected the receipt');
  }
  plainObject(claims, 'verified claims');
  for (const field of CORE_CLAIMS) nonemptyString(claims[field], `verified claims.${field}`);
  for (const field of KNOWN_TASK_FIELDS) {
    if (claims[field] !== expectedTask[field]) {
      fail('expected-claim-mismatch', `verified ${field} differs from the known task`);
    }
  }
  claims = validateSignedSpisClaims(claims, config, 'verified claims');
  validateReceiptClaimCopies(receipt, claims);
  if (claims.keyId !== receipt.keyId) fail('receipt-key-mismatch', 'verified keyId differs from the retained receipt');
  return {
    schema: RECEIPT_CHECKPOINT_SCHEMA,
    client: {
      package: CLIENT_PACKAGE,
      commit: CLIENT_COMMIT,
      keySetVersion: config.keySetVersion,
    },
    receipt,
    claims,
  };
}
function validateSignedSpisClaims(claims, config, name) {
  onlyKeys(claims, [...CORE_CLAIMS, ...SIGNED_SPIS_CLAIMS, 'keyId'], name);
  for (const field of ['requestDigest', 'resultDigest']) {
    if (!SHA256_ID.test(nonemptyString(claims[field], `${name}.${field}`))) {
      fail('invalid-receipt-payload', `${name}.${field} must be a sha256: identifier`);
    }
  }
  const spisBinding = validateSpisBinding(claims.spisBinding, `${name}.spisBinding`);
  if (spisBinding.service.action !== config.allowedAction) {
    fail('expected-claim-mismatch', 'signed spisBinding action differs from public receipt trust');
  }
  return { ...claims, spisBinding };
}
function validateReceiptClaimCopies(receipt, claims) {
  for (const field of [...CORE_CLAIMS, 'requestDigest', 'resultDigest']) {
    if (receipt[field] !== claims[field]) {
      fail('receipt-claim-mismatch', `retained receipt ${field} differs from verified claims`);
    }
  }
  if (canonicalJson(receipt.spisBinding) !== canonicalJson(claims.spisBinding)) {
    fail('receipt-claim-mismatch', 'retained receipt spisBinding differs from verified claims');
  }
}


// The retained manifest of a successful task binds a real navigation and must carry the
// required browser evidence; the manifest of a failed, cancelled or rejected task binds
// no navigation at all, so it has no effective/final URL and no required evidence kind.
// The outcome the caller expects, already proved to equal the signed claim, selects the
// shape; neither shape is ever accepted in the other's place.
function validateEvidenceManifest(value, expectedClaims) {
  const manifest = plainObject(value, 'retained evidence manifest');
  const successful = expectedClaims.outcome === 'completed';
  onlyKeys(
    manifest,
    successful ? EVIDENCE_MANIFEST_KEYS : NON_SUCCESS_EVIDENCE_MANIFEST_KEYS,
    'retained evidence manifest',
  );
  const taskId = nonemptyString(manifest.taskId, 'retained evidence manifest.taskId');
  const requestedUrl = canonicalHttpUrl(
    manifest.requestedUrl,
    'retained evidence manifest.requestedUrl',
  );
  if (!portableAttemptComponent(taskId)) {
    fail('invalid-artifact', 'retained evidence manifest.taskId is not a portable recording component');
  }
  const navigation = successful
    ? {
      effectiveUrl: canonicalHttpUrl(
        manifest.effectiveUrl,
        'retained evidence manifest.effectiveUrl',
      ),
      finalUrl: canonicalHttpUrl(
        manifest.finalUrl,
        'retained evidence manifest.finalUrl',
      ),
    }
    : null;
  if (manifest.schema !== (successful ? EVIDENCE_MANIFEST_SCHEMA : NON_SUCCESS_EVIDENCE_MANIFEST_SCHEMA)
      || taskId !== expectedClaims.taskId
      || manifest.organizationId !== expectedClaims.organizationId
      || manifest.origin !== expectedClaims.origin
      || manifest.action !== expectedClaims.action
      || manifest.outcome !== expectedClaims.outcome
      || manifest.requestDigest !== expectedClaims.requestDigest
      || manifest.resultDigest !== expectedClaims.resultDigest
      || canonicalJson(validateSpisBinding(
        manifest.spisBinding,
        'retained evidence manifest.spisBinding',
      )) !== canonicalJson(expectedClaims.spisBinding)
      || requestedUrl !== manifest.requestedUrl
      || new URL(requestedUrl).origin !== expectedClaims.origin
      || (navigation !== null
        && (navigation.effectiveUrl !== manifest.effectiveUrl
          || navigation.finalUrl !== manifest.finalUrl
          || new URL(navigation.effectiveUrl).origin !== expectedClaims.origin
          || new URL(navigation.finalUrl).origin !== expectedClaims.origin))) {
    fail('artifact-binding-mismatch', 'retained evidence manifest differs from signed claims');
  }
  if (!Array.isArray(manifest.evidenceInventory)) {
    fail('invalid-artifact', 'retained evidence manifest inventory must be an array');
  }
  const prefix = `stado://weles/recordings/${taskId}/`;
  // `finalize` demands the screenshot and the accessibility tree only from a succeeded
  // task; a non-success retains whatever the run produced, including nothing.
  const required = successful
    ? new Map([
      ['screenshot', `${prefix}artifacts/browser_evidence_final.png`],
      ['accessibility_tree', `${prefix}artifacts/browser_evidence_accessibility_tree.txt`],
    ])
    : new Map();
  const kinds = new Set();
  const uris = new Set();
  let totalBytes = 0;
  const evidenceInventory = manifest.evidenceInventory.map((entryValue, index) => {
    const name = `retained evidence manifest.evidenceInventory[${index}]`;
    const entry = plainObject(entryValue, name);
    onlyKeys(entry, ['kind', 'uri', 'sha256', 'bytes'], name);
    const kind = nonemptyString(entry.kind, `${name}.kind`);
    const uri = nonemptyString(entry.uri, `${name}.uri`);
    const relative = uri.startsWith(prefix) ? uri.slice(prefix.length) : '';
    const supported = required.has(kind)
      ? uri === required.get(kind)
      : kind.startsWith('artifact:') && kind.slice('artifact:'.length) === relative;
    if (!supported
        || !relative
        || relative.includes('\\')
        || relative.split('/').some((part) => !portableAttemptComponent(part))
        || !SHA256.test(nonemptyString(entry.sha256, `${name}.sha256`))
        || !Number.isSafeInteger(entry.bytes)
        || entry.bytes < 1
        || kinds.has(kind)
        || uris.has(uri)) {
      fail('invalid-artifact', `${name} is not a canonical immutable evidence entry`);
    }
    totalBytes += entry.bytes;
    if (!Number.isSafeInteger(totalBytes) || totalBytes > MAX_EVIDENCE_INVENTORY_BYTES) {
      fail('artifact-too-large', 'retained evidence inventory exceeded the total byte limit');
    }
    kinds.add(kind);
    uris.add(uri);
    return { kind, uri, sha256: entry.sha256, bytes: entry.bytes };
  });
  for (const kind of required.keys()) {
    if (!kinds.has(kind)) fail('invalid-artifact', `retained evidence inventory lacks ${kind}`);
  }
  return {
    ...manifest,
    requestedUrl,
    ...(navigation ?? {}),
    evidenceInventory,
  };
}


async function buildProvenance(receiptValue, expectedValue, artifactValue, config, verifyReceipt) {
  const receipt = validateReceipt(receiptValue);
  const expectedClaims = validateExpectedClaims(expectedValue, config);
  const artifactExpectation = validateArtifact(artifactValue);
  let claims;
  try {
    claims = verifyReceipt(receipt, config.receiptKeys);
  } catch (error) {
    const code = typeof error?.code === 'string' ? error.code : 'receipt-verification-failed';
    fail(code, 'the official Weles client rejected the receipt');
  }
  plainObject(claims, 'verified claims');
  for (const field of CORE_CLAIMS) {
    nonemptyString(claims[field], `verified claims.${field}`);
    if (claims[field] !== expectedClaims[field]) {
      fail('expected-claim-mismatch', `verified ${field} differs from the caller expectation`);
    }
  }
  claims = validateSignedSpisClaims(claims, config, 'verified claims');
  validateReceiptClaimCopies(receipt, claims);
  if (claims.requestDigest !== expectedClaims.requestDigest
      || claims.resultDigest !== expectedClaims.resultDigest
      || canonicalJson(claims.spisBinding) !== canonicalJson(expectedClaims.spisBinding)) {
    fail('expected-claim-mismatch', 'signed Spis request/result/binding claims differ from caller expectations');
  }
  if (claims.keyId !== receipt.keyId) fail('receipt-key-mismatch', 'verified keyId differs from the retained receipt');
  const { artifact, value: evidenceManifest } = await digestArtifact(artifactExpectation);
  validateEvidenceManifest(evidenceManifest, expectedClaims);
  if (claims.evidenceDigest !== artifact.sha256) {
    fail('artifact-binding-mismatch', 'verified evidenceDigest does not equal the retained artifact digest');
  }
  return {
    schema: PROVENANCE_SCHEMA,
    id: provenanceId(receipt, config.keySetVersion, artifact),
    client: {
      package: CLIENT_PACKAGE,
      commit: CLIENT_COMMIT,
      keySetVersion: config.keySetVersion,
    },
    receipt,
    claims,
    expectedClaims,
    artifact,
  };
}

function operationServiceIdentity(value, config, action) {
  const identity = validateServiceIdentity(value, 'command.serviceIdentity');
  if (identity.endpoint !== config.endpoint) {
    fail('service-identity-mismatch', 'public service endpoint differs from protected network endpoint');
  }
  if (identity.action !== action || identity.action !== config.allowedAction) {
    fail('service-identity-mismatch', 'public service action differs from task and receipt trust');
  }
  return identity;
}

function networkClient(config, WelesClient, serviceIdentity, origin, action) {
  return new WelesClient({
    endpoint: serviceIdentity.endpoint,
    bearer: config.bearer,
    organizationId: config.organizationId,
    allowedOrigins: [origin],
    allowedActions: [action],
    receiptKeys: config.receiptKeys,
  });
}

function assertJcsString(value) {
  for (let index = 0; index < value.length; index += 1) {
    const unit = value.charCodeAt(index);
    if (unit >= 0xd800 && unit <= 0xdbff) {
      const next = value.charCodeAt(index + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) {
        fail('non-canonical-json', 'JCS strings must not contain lone UTF-16 surrogates');
      }
      index += 1;
    } else if (unit >= 0xdc00 && unit <= 0xdfff) {
      fail('non-canonical-json', 'JCS strings must not contain lone UTF-16 surrogates');
    }
  }
}

function compareUtf16(left, right) {
  const length = Math.min(left.length, right.length);
  for (let index = 0; index < length; index += 1) {
    const difference = left.charCodeAt(index) - right.charCodeAt(index);
    if (difference !== 0) return difference;
  }
  return left.length - right.length;
}

function canonicalJson(value) {
  if (value === null) return 'null';
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'string') {
    assertJcsString(value);
    return JSON.stringify(value);
  }
  if (typeof value === 'number') {
    if (!Number.isSafeInteger(value)) {
      fail('non-canonical-json', 'JCS input numbers must be safe integers');
    }
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  if (typeof value !== 'object'
      || (Object.getPrototypeOf(value) !== Object.prototype
        && Object.getPrototypeOf(value) !== null)) {
    fail('non-canonical-json', 'JCS input must contain only JSON values');
  }
  return `{${Object.keys(value)
    .sort(compareUtf16)
    .map((key) => {
      assertJcsString(key);
      return `${JSON.stringify(key)}:${canonicalJson(value[key])}`;
    })
    .join(',')}}`;
}
function canonicalHttpUrl(value, name) {
  const raw = nonemptyString(value, name);
  let parsed;
  try {
    parsed = new URL(raw);
  } catch {
    fail('invalid-input', `${name} must be an absolute HTTP(S) URL`);
  }
  if (!['http:', 'https:'].includes(parsed.protocol)
      || parsed.username
      || parsed.password) {
    fail('invalid-input', `${name} must be an absolute HTTP(S) URL without credentials`);
  }
  return parsed.href;
}

function canonicalApiEndpoint(value, name) {
  const raw = nonemptyString(value, name);
  const canonical = canonicalHttpUrl(raw, name);
  const parsed = new URL(canonical);
  if (canonical !== raw
      || parsed.pathname !== '/api/v1'
      || parsed.search
      || parsed.hash) {
    fail('invalid-input', `${name} must be the canonical exact /api/v1 service base`);
  }
  return raw;
}

function portableAttemptComponent(value) {
  return typeof value === 'string'
    && value !== ''
    && value !== '.'
    && value !== '..'
    && /^[A-Za-z0-9._-]+$/.test(value);
}

function sha256Text(value) {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

function validateAttemptBindingDerivation(binding, name) {
  const catalogKey = sha256Text(
    `${binding.source_revision}\0${binding.run_id}\0${binding.catalog}`,
  );
  const recordKey = sha256Text(
    `${catalogKey}\0${binding.record}\0${binding.source_input_sha256}`,
  );
  if (binding.record_key !== recordKey) {
    fail('invalid-input', `${name}.record_key is not the runtime record-key derivation`);
  }
  const attemptFingerprint = sha256Text(
    `${binding.record_key}\0${binding.attempt}\0${binding.service.host}`,
  ).slice(0, 16);
  if (binding.attempt_id !== `attempt-${binding.attempt}-${attemptFingerprint}`) {
    fail('invalid-input', `${name}.attempt_id is not the runtime attempt-identity derivation`);
  }
}

function validateAttemptBindingUris(binding, name) {
  for (const field of ['run_id', 'catalog', 'record', 'attempt_id']) {
    if (!portableAttemptComponent(binding[field])) {
      fail('invalid-input', `${name}.${field} is not a portable attempt URI component`);
    }
  }
  if (!SHA256.test(binding.record_key)) {
    fail('invalid-input', `${name}.record_key must be a lowercase SHA-256`);
  }
  validateAttemptBindingDerivation(binding, name);
  const base = `stado://spis-crawls/${binding.run_id}/${binding.catalog}/${binding.record}`
    + `/${binding.record_key}/attempts/${binding.attempt}/${binding.attempt_id}`;
  if (binding.artifact_uri !== `${base}/artifacts.tar.gz`
      || binding.output_uri !== `${base}/worker-output.log`) {
    fail('invalid-input', `${name} artifact/output URIs are not canonical attempt coordinates`);
  }
}
function validateBrowserTaskInput(value, name) {
  const input = plainObject(value, name);
  onlyKeys(input, ['product_url', 'objective', 'constraints', 'spisBinding'], name);
  return {
    product_url: canonicalHttpUrl(input.product_url, `${name}.product_url`),
    objective: nonemptyString(input.objective, `${name}.objective`),
    constraints: possiblyEmptyStringArray(input.constraints, `${name}.constraints`),
    spisBinding: validateSpisBinding(input.spisBinding, `${name}.spisBinding`),
  };
}

function validateOfficialTaskRequest(value, expectedClaims, config) {
  const request = plainObject(value, 'command.requestDocument');
  onlyKeys(request, [
    'schema',
    'organizationId',
    'origin',
    'action',
    'input',
    'credentialRefs',
    'evidencePolicy',
    'justification',
  ], 'command.requestDocument');
  const normalized = {
    schema: request.schema,
    organizationId: nonemptyString(request.organizationId, 'command.requestDocument.organizationId'),
    origin: exactOrigin(request.origin, 'command.requestDocument.origin'),
    action: nonemptyString(request.action, 'command.requestDocument.action'),
    input: validateBrowserTaskInput(request.input, 'command.requestDocument.input'),
    credentialRefs: possiblyEmptyStringArray(
      request.credentialRefs,
      'command.requestDocument.credentialRefs',
    ),
    evidencePolicy: nonemptyString(
      request.evidencePolicy,
      'command.requestDocument.evidencePolicy',
    ),
    justification: nonemptyString(
      request.justification,
      'command.requestDocument.justification',
    ),
  };
  if (normalized.schema !== 'weles.task.current'
      || normalized.organizationId !== config.organizationId
      || normalized.organizationId !== expectedClaims.organizationId
      || normalized.origin !== expectedClaims.origin
      || normalized.origin !== new URL(normalized.input.product_url).origin
      || normalized.action !== config.allowedAction
      || normalized.action !== expectedClaims.action
      || normalized.credentialRefs.length !== 0
      || normalized.evidencePolicy !== 'full'
      || canonicalJson(normalized.input.spisBinding)
        !== canonicalJson(expectedClaims.spisBinding)) {
    fail('expected-claim-mismatch', 'retained official request differs from signed expectations');
  }
  const requestDigest = `sha256:${createHash('sha256')
    .update(canonicalJson(normalized))
    .digest('hex')}`;
  if (requestDigest !== expectedClaims.requestDigest) {
    fail('expected-claim-mismatch', 'retained official request digest differs from the signed receipt');
  }
  return normalized;
}
function assertBindingServiceIdentity(binding, serviceIdentity) {
  const service = binding.service;
  if (service.name !== serviceIdentity.name
      || service.consumer !== serviceIdentity.consumer
      || service.capability !== serviceIdentity.capability
      || service.directory_generation !== serviceIdentity.generation
      || service.host !== serviceIdentity.active_host
      || service.endpoint !== serviceIdentity.endpoint
      || service.action !== serviceIdentity.action
      || service.release_id !== serviceIdentity.release_id
      || service.source_revision !== serviceIdentity.source_revision) {
    fail('service-identity-mismatch', 'input spisBinding differs from the exact service-directory identity');
  }
}


function prepareSubmission(commandValue, config) {
  const command = plainObject(commandValue, 'command');
  if (command.schema !== COMMAND_SCHEMA) fail('unsupported-command', 'bridge command schema is unsupported');
  if (command.operation !== 'submit') fail('unsupported-operation', 'submission preparation requires submit');
  onlyKeys(command, ['schema', 'operation', 'serviceIdentity', 'request', 'idempotencyKey'], 'command');
  const request = plainObject(command.request, 'command.request');
  onlyKeys(request, ['origin', 'action', 'input', 'credentialRefs', 'evidencePolicy', 'justification'], 'command.request');
  const origin = exactOrigin(request.origin, 'command.request.origin');
  const action = nonemptyString(request.action, 'command.request.action');
  if (action !== config.allowedAction) {
    fail('action-denied', 'command.request.action differs from public receipt trust');
  }
  const serviceIdentity = operationServiceIdentity(command.serviceIdentity, config, action);
  const input = validateBrowserTaskInput(request.input, 'command.request.input');
  const { spisBinding } = input;
  assertBindingServiceIdentity(spisBinding, serviceIdentity);
  const normalizedRequest = {
    origin,
    action,
    input,
    credentialRefs: request.credentialRefs === undefined
      ? []
      : possiblyEmptyStringArray(request.credentialRefs, 'command.request.credentialRefs'),
    evidencePolicy: request.evidencePolicy === undefined
      ? 'full'
      : nonemptyString(request.evidencePolicy, 'command.request.evidencePolicy'),
    justification: nonemptyString(request.justification, 'command.request.justification'),
  };
  if (normalizedRequest.credentialRefs.length !== 0
      || normalizedRequest.evidencePolicy !== 'full'
      || normalizedRequest.origin !== new URL(normalizedRequest.input.product_url).origin) {
    fail(
      'invalid-input',
      'browser request must be anonymous, full-evidence, and use the product URL origin',
    );
  }
  const idempotencyKey = nonemptyString(command.idempotencyKey, 'command.idempotencyKey');
  const officialBody = {
    schema: 'weles.task.current',
    organizationId: config.organizationId,
    ...normalizedRequest,
  };
  const requestDigest = `sha256:${createHash('sha256')
    .update(canonicalJson(officialBody))
    .digest('hex')}`;
  return {
    request: normalizedRequest,
    serviceIdentity,
    spisBinding,
    requestDocument: officialBody,
    idempotencyKey,
    requestDigest,
    origin,
    action,
  };
}

function reusableSubmission(path, prepared, config) {
  if (path === '-') {
    fail('durable-output-required', 'submit requires a retained output file for request-bound recovery');
  }
  const destination = resolve(path);
  let stat;
  try {
    stat = lstatSync(destination);
  } catch (error) {
    if (error?.code === 'ENOENT') return null;
    fail('output-failed', 'existing submission output could not be inspected');
  }
  if (stat.isSymbolicLink() || !stat.isFile()) {
    fail('invalid-output', 'submission output must be a regular non-symlink file');
  }
  if (stat.size > MAX_OUTPUT_BYTES) fail('output-conflict', 'existing submission output is oversized');
  let existing;
  try {
    existing = JSON.parse(readFileSync(destination, 'utf8'));
    plainObject(existing, 'existing submission output');
    onlyKeys(existing, [
      'schema',
      'taskId',
      'organizationId',
      'origin',
      'action',
      'idempotencyKey',
      'requestDigest',
      'requestDocument',
      'requestIdentity',
      'receiptCheckpoint',
      'serviceIdentity',
    ], 'existing submission output');
  } catch {
    fail('output-conflict', 'existing submission output is not a retained bridge submission');
  }
  const requestIdentity = validateRequestIdentity(
    existing.requestIdentity,
    'existing submission output.requestIdentity',
  );
  const requestDocument = validateOfficialTaskRequest(existing.requestDocument, {
    organizationId: config.organizationId,
    origin: prepared.origin,
    action: prepared.action,
    spisBinding: prepared.spisBinding,
    requestDigest: prepared.requestDigest,
  }, config);
  const sameRequest = existing.schema === SUBMISSION_SCHEMA
    && existing.requestDigest === prepared.requestDigest
    && existing.idempotencyKey === prepared.idempotencyKey
    && canonicalJson(requestDocument) === canonicalJson(prepared.requestDocument)
    && requestIdentity.requestDigest === prepared.requestDigest
    && canonicalJson(requestIdentity.spisBinding) === canonicalJson(prepared.spisBinding)
    && existing.organizationId === config.organizationId
    && existing.origin === prepared.origin
    && existing.action === prepared.action
    && canonicalJson(existing.serviceIdentity) === canonicalJson(prepared.serviceIdentity)
    && typeof existing.taskId === 'string'
    && existing.taskId.length > 0;
  if (!sameRequest) {
    fail('output-conflict', 'existing submission output belongs to a different canonical request');
  }
  return existing;
}

function exactResultReferences(response, name) {
  let resultRef = null;
  if (response.resultRef !== undefined && response.resultRef !== null) {
    resultRef = nonemptyString(response.resultRef, `${name}.resultRef`);
  }
  let artifactRefs = [];
  if (response.artifactRefs !== undefined) {
    artifactRefs = possiblyEmptyStringArray(response.artifactRefs, `${name}.artifactRefs`);
  }
  return { resultRef, artifactRefs };
}

function taskStatusDocument(
  schema,
  responseValue,
  expectedTask,
  serviceIdentity,
  config,
  verifyReceipt,
  responseName,
) {
  const response = plainObject(responseValue, responseName);
  for (const field of KNOWN_TASK_FIELDS) {
    if (nonemptyString(response[field], `${responseName}.${field}`) !== expectedTask[field]) {
      fail('expected-claim-mismatch', `${responseName} ${field} differs from the known task`);
    }
  }
  const responseServiceIdentity = validateServiceIdentity(
    response.serviceIdentity,
    `${responseName}.serviceIdentity`,
  );
  if (canonicalJson(responseServiceIdentity) !== canonicalJson(serviceIdentity)) {
    fail(
      'service-identity-mismatch',
      `${responseName}.serviceIdentity differs from the service-directory/version readback`,
    );
  }
  const requestIdentity = validateRequestIdentity(
    response.requestIdentity,
    `${responseName}.requestIdentity`,
  );
  assertBindingServiceIdentity(requestIdentity.spisBinding, serviceIdentity);
  if (requestIdentity.spisBinding.service.action !== expectedTask.action) {
    fail('expected-claim-mismatch', 'status request identity action differs from the known task');
  }
  const status = nonemptyString(response.status, `${responseName}.status`);
  const mappedOutcome = TERMINAL_OUTCOME_BY_STATUS.get(status);
  if (!NONTERMINAL_STATUSES.has(status) && mappedOutcome === undefined) {
    fail('unsupported-task-status', `${responseName}.status is not in the typed Weles status contract`);
  }
  const terminal = mappedOutcome !== undefined;
  const result = {
    schema,
    ...expectedTask,
    serviceIdentity: responseServiceIdentity,
    requestIdentity,
    resultDigest: null,
    status,
    terminal,
    outcome: null,
    ...exactResultReferences(response, responseName),
  };
  if (!terminal) {
    if (response.outcome !== undefined && response.outcome !== null) {
      fail('status-outcome-mismatch', 'a nonterminal task must not report a terminal outcome');
    }
    if (response.resultDigest !== undefined && response.resultDigest !== null) {
      fail('status-outcome-mismatch', 'a nonterminal task must not report a result digest');
    }
    return result;
  }
  const outcome = nonemptyString(response.outcome, `${responseName}.outcome`);
  if (outcome !== mappedOutcome) {
    fail('status-outcome-mismatch', 'terminal status does not map to its exact outcome');
  }
  if (!response.receipt) {
    fail('missing-terminal-receipt', 'a terminal Weles task requires a fresh signed receipt checkpoint');
  }
  const receiptCheckpoint = buildReceiptCheckpoint(
    response.receipt,
    expectedTask,
    config,
    verifyReceipt,
  );
  if (receiptCheckpoint.claims.requestDigest !== requestIdentity.requestDigest
      || canonicalJson(receiptCheckpoint.claims.spisBinding)
        !== canonicalJson(requestIdentity.spisBinding)) {
    fail('expected-claim-mismatch', 'terminal receipt differs from the status request identity');
  }
  const resultDigest = nonemptyString(response.resultDigest, `${responseName}.resultDigest`);
  if (!SHA256_ID.test(resultDigest)
      || receiptCheckpoint.claims.resultDigest !== resultDigest) {
    fail('status-outcome-mismatch', 'terminal result digest differs from the verified receipt');
  }
  if (receiptCheckpoint.claims.outcome !== outcome) {
    fail('status-outcome-mismatch', 'terminal status/outcome differs from the freshly verified receipt');
  }
  result.outcome = outcome;
  result.resultDigest = resultDigest;
  result.receiptCheckpoint = receiptCheckpoint;
  return result;
}

async function execute(commandValue, config, official, preparedSubmission) {
  const command = plainObject(commandValue, 'command');
  if (command.schema !== COMMAND_SCHEMA) fail('unsupported-command', 'bridge command schema is unsupported');
  const operation = nonemptyString(command.operation, 'command.operation');
  if (operation === 'verify') {
    onlyKeys(command, ['schema', 'operation', 'receipt', 'expectedClaims', 'artifact'], 'command');
    return buildProvenance(command.receipt, command.expectedClaims, command.artifact, config, official.verifyReceipt);
  }
  if (operation === 'get') {
    onlyKeys(command, ['schema', 'operation', 'serviceIdentity', 'taskId', 'expectedTask'], 'command');
    const taskId = nonemptyString(command.taskId, 'command.taskId');
    const expectedTask = validateExpectedTask(command.expectedTask, config);
    if (taskId !== expectedTask.taskId) fail('expected-claim-mismatch', 'get taskId differs from expectedTask.taskId');
    const serviceIdentity = operationServiceIdentity(
      command.serviceIdentity,
      config,
      expectedTask.action,
    );
    let response;
    try {
      response = await networkClient(
        config,
        official.WelesClient,
        serviceIdentity,
        expectedTask.origin,
        expectedTask.action,
      ).get(taskId);
    } catch (error) {
      const code = typeof error?.code === 'string' ? error.code : 'weles-request-failed';
      fail(code, 'the official Weles client get operation failed');
    }
    return taskStatusDocument(
      TASK_STATUS_SCHEMA,
      response,
      expectedTask,
      serviceIdentity,
      config,
      official.verifyReceipt,
      'Weles get response',
    );
  }
  if (operation === 'cancel') {
    onlyKeys(command, ['schema', 'operation', 'serviceIdentity', 'taskId', 'expectedTask', 'reason', 'idempotencyKey'], 'command');
    const taskId = nonemptyString(command.taskId, 'command.taskId');
    const expectedTask = validateExpectedTask(command.expectedTask, config);
    if (taskId !== expectedTask.taskId) fail('expected-claim-mismatch', 'cancel taskId differs from expectedTask.taskId');
    const serviceIdentity = operationServiceIdentity(
      command.serviceIdentity,
      config,
      expectedTask.action,
    );
    const reason = nonemptyString(command.reason, 'command.reason');
    const idempotencyKey = nonemptyString(command.idempotencyKey, 'command.idempotencyKey');
    let response;
    try {
      response = await networkClient(
        config,
        official.WelesClient,
        serviceIdentity,
        expectedTask.origin,
        expectedTask.action,
      ).cancel(taskId, {
        reason,
        idempotencyKey,
      });
    } catch (error) {
      const code = typeof error?.code === 'string' ? error.code : 'weles-request-failed';
      fail(code, 'the official Weles client cancel operation failed');
    }
    return {
      ...taskStatusDocument(
        CANCELLATION_SCHEMA,
        response,
        expectedTask,
        serviceIdentity,
        config,
        official.verifyReceipt,
        'Weles cancel response',
      ),
      idempotencyKey,
    };
  }
  if (operation === 'submit') {
    const prepared = preparedSubmission ?? prepareSubmission(command, config);
    let response;
    try {
      response = await networkClient(
        config,
        official.WelesClient,
        prepared.serviceIdentity,
        prepared.origin,
        prepared.action,
      ).submit(
        prepared.request,
        { idempotencyKey: prepared.idempotencyKey },
      );
    } catch (error) {
      const code = typeof error?.code === 'string' ? error.code : 'weles-request-failed';
      fail(code, 'the official Weles client submit operation failed');
    }
    plainObject(response, 'Weles submit response');
    const taskId = nonemptyString(response.taskId, 'Weles submit response.taskId');
    const responseServiceIdentity = validateServiceIdentity(
      response.serviceIdentity,
      'Weles submit response.serviceIdentity',
    );
    if (canonicalJson(responseServiceIdentity) !== canonicalJson(prepared.serviceIdentity)) {
      fail(
        'service-identity-mismatch',
        'Weles submit response.serviceIdentity differs from the service-directory/version readback',
      );
    }
    const responseRequestIdentity = validateRequestIdentity(
      response.requestIdentity,
      'Weles submit response.requestIdentity',
    );
    if (responseRequestIdentity.requestDigest !== prepared.requestDigest
        || canonicalJson(responseRequestIdentity.spisBinding)
          !== canonicalJson(prepared.spisBinding)) {
      fail(
        'expected-claim-mismatch',
        'Weles submit response request identity differs from the canonical submitted request',
      );
    }
    const knownTask = {
      taskId,
      organizationId: config.organizationId,
      origin: prepared.origin,
      action: prepared.action,
    };
    const result = {
      schema: SUBMISSION_SCHEMA,
      ...knownTask,
      serviceIdentity: responseServiceIdentity,
      idempotencyKey: prepared.idempotencyKey,
      requestDigest: prepared.requestDigest,
      requestDocument: prepared.requestDocument,
      requestIdentity: responseRequestIdentity,
    };
    if (response.receipt) {
      result.receiptCheckpoint = buildReceiptCheckpoint(
        response.receipt,
        knownTask,
        config,
        official.verifyReceipt,
      );
    }
    return result;
  }
  fail('unsupported-operation', 'bridge operation must be submit, get, cancel, or verify');
}

function existingOutputMatches(destination, bytes) {
  try {
    const existing = lstatSync(destination);
    if (existing.isSymbolicLink() || !existing.isFile()) {
      fail('invalid-output', 'output must not be a symlink or non-file');
    }
    if (existing.size > MAX_OUTPUT_BYTES) fail('output-conflict', 'existing output differs from this result');
    if (readFileSync(destination).equals(bytes)) return true;
    fail('output-conflict', 'existing output differs from this result');
  } catch (error) {
    if (error instanceof BridgeError) throw error;
    if (error?.code === 'ENOENT') return false;
    fail('output-failed', 'existing bridge output could not be inspected');
  }
}
function syncOutput(destination, parent) {
  const persistedFd = openSync(destination, fsConstants.O_RDONLY);
  try {
    fsyncSync(persistedFd);
  } finally {
    closeSync(persistedFd);
  }
  const parentFd = openSync(parent, fsConstants.O_RDONLY);
  try {
    fsyncSync(parentFd);
  } finally {
    closeSync(parentFd);
  }
}


function writeOutput(path, document) {
  const text = `${JSON.stringify(document, null, 2)}\n`;
  const bytes = Buffer.from(text, 'utf8');
  if (bytes.length > MAX_OUTPUT_BYTES) fail('output-too-large', 'bridge output exceeded the size limit');
  if (path === '-') {
    process.stdout.write(bytes);
    return;
  }
  const destination = resolve(path);
  const parent = dirname(destination);
  const temporary = join(parent, `.${basename(destination)}.${process.pid}.${randomUUID()}.tmp`);
  let fd;
  try {
    if (existingOutputMatches(destination, bytes)) {
      syncOutput(destination, parent);
      return;
    }
    fd = openSync(temporary, fsConstants.O_WRONLY | fsConstants.O_CREAT | fsConstants.O_EXCL, 0o600);
    writeFileSync(fd, bytes);
    fsyncSync(fd);
    closeSync(fd);
    fd = undefined;
    try {
      linkSync(temporary, destination);
    } catch (error) {
      if (error?.code !== 'EEXIST') throw error;
      if (existingOutputMatches(destination, bytes)) {
        syncOutput(destination, parent);
        return;
      }
    }
    unlinkSync(temporary);
    syncOutput(destination, parent);
  } catch (error) {
    if (error instanceof BridgeError) throw error;
    fail('output-failed', 'bridge output could not be persisted');
  } finally {
    if (fd !== undefined) {
      try { closeSync(fd); } catch {}
    }
    try { unlinkSync(temporary); } catch {}
  }
}

try {
  const args = parseArgs(process.argv.slice(2));
  const inputText = args.input === '-' ? await readStdin() : readBoundedFile(args.input, 'bridge input');
  const command = parseJson(inputText, 'bridge input');
  const commandEnvelope = plainObject(command, 'command');
  if (commandEnvelope.schema !== COMMAND_SCHEMA) fail('unsupported-command', 'bridge command schema is unsupported');
  const operation = nonemptyString(commandEnvelope.operation, 'command.operation');
  if (!['submit', 'get', 'cancel', 'verify'].includes(operation)) {
    fail('unsupported-operation', 'bridge operation must be submit, get, cancel, or verify');
  }
  if (operation === 'get' && args.output !== '-') {
    fail(
      'poll-output-contract',
      'get must return bounded stdout for caller-owned content-addressed immutable persistence',
    );
  }
  const trust = loadTrust();
  const config = operation === 'verify' ? trust : loadConfig(trust);
  const preparedSubmission = command?.operation === 'submit'
    ? prepareSubmission(command, config)
    : null;
  const recoveredSubmission = preparedSubmission
    ? reusableSubmission(args.output, preparedSubmission, config)
    : null;
  const result = recoveredSubmission ?? await execute(
    command,
    config,
    await loadOfficialClient(),
    preparedSubmission,
  );
  writeOutput(args.output, result);
} catch (error) {
  const code = error instanceof BridgeError ? error.code : 'bridge-failed';
  const message = error instanceof BridgeError ? error.message : 'the Weles bridge failed closed';
  process.stderr.write(`${JSON.stringify({ schema: ERROR_SCHEMA, code, message })}\n`);
  process.exitCode = 1;
}
