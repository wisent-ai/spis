#!/usr/bin/env node

import { createHash, randomUUID } from 'node:crypto';
import {
  constants as fsConstants,
  closeSync,
  createReadStream,
  fsyncSync,
  lstatSync,
  openSync,
  readFileSync,
  realpathSync,
  linkSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import { open } from 'node:fs/promises';
import { basename, dirname, isAbsolute, join, normalize, resolve, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

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
const MAX_OUTPUT_BYTES = 4 * 1024 * 1024;
const SHA256 = /^[0-9a-f]{64}$/;
const NONTERMINAL_STATUSES = new Set(['queued', 'running', 'pending_review']);
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
const RECEIPT_FIELDS = Object.freeze([
  'schema',
  ...CORE_CLAIMS,
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

function stringArray(value, name) {
  if (!Array.isArray(value) || value.length === 0) {
    fail('invalid-config', `${name} must be a non-empty string array`);
  }
  return value.map((entry, index) => nonemptyString(entry, `${name}[${index}]`));
}

function onlyKeys(value, allowed, name) {
  const unknown = Object.keys(value).filter(key => !allowed.includes(key));
  if (unknown.length !== 0) {
    fail('invalid-input', `${name} contains unsupported fields`);
  }
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
  const stat = lstatSync(path);
  if (!stat.isFile() || stat.isSymbolicLink()) fail('invalid-input-file', `${name} must be a regular file`);
  if (stat.size > MAX_JSON_BYTES) fail('input-too-large', `${name} exceeded the size limit`);
  return readFileSync(path, 'utf8');
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
  const path = process.env.SPIS_WELES_TRUST_FILE;
  if (!path) fail('trust-unavailable', 'SPIS_WELES_TRUST_FILE is required for receipt verification');
  const trust = plainObject(readPublicTrust(path), 'public Weles receipt trust');
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
    allowedOrigins: null,
    allowedActions: [allowedAction],
    terminalOutcomes: ['completed'],
    receiptKeys,
    keySetVersion,
  };
}


function parseEnvironmentJson(name) {
  const value = process.env[name];
  if (!value) fail('config-unavailable', `${name} is required when SPIS_WELES_CONFIG_FILE is absent`);
  return parseJson(value, name);
}

function loadConfig() {
  let config;
  if (process.env.SPIS_WELES_CONFIG_FILE) {
    config = readProtectedConfig(process.env.SPIS_WELES_CONFIG_FILE);
  } else {
    config = {
      schema: CONFIG_SCHEMA,
      endpoint: process.env.WELES_API_BASE,
      bearer: process.env.WELES_TOKEN,
      organizationId: process.env.WISENT_ORGANIZATION_ID,
      allowedOrigins: parseEnvironmentJson('SPIS_WELES_ALLOWED_ORIGINS_JSON'),
      allowedActions: parseEnvironmentJson('SPIS_WELES_ALLOWED_ACTIONS_JSON'),
      terminalOutcomes: parseEnvironmentJson('SPIS_WELES_TERMINAL_OUTCOMES_JSON'),
      receiptKeys: parseEnvironmentJson('SPIS_WELES_RECEIPT_KEYS_JSON'),
      keySetVersion: process.env.SPIS_WELES_KEY_SET_VERSION,
    };
  }
  plainObject(config, 'config');
  onlyKeys(config, [
    'schema',
    'endpoint',
    'bearer',
    'organizationId',
    'allowedOrigins',
    'allowedActions',
    'terminalOutcomes',
    'receiptKeys',
    'keySetVersion',
  ], 'config');
  if (config.schema !== CONFIG_SCHEMA) fail('invalid-config', 'Weles bridge config schema is unsupported');
  config.organizationId = nonemptyString(config.organizationId, 'config.organizationId');
  config.allowedOrigins = stringArray(config.allowedOrigins, 'config.allowedOrigins');
  config.allowedActions = stringArray(config.allowedActions, 'config.allowedActions');
  config.terminalOutcomes = stringArray(config.terminalOutcomes, 'config.terminalOutcomes');
  config.keySetVersion = nonemptyString(config.keySetVersion, 'config.keySetVersion');
  plainObject(config.receiptKeys, 'config.receiptKeys');
  if (Object.keys(config.receiptKeys).length === 0) fail('invalid-config', 'config.receiptKeys must not be empty');
  for (const [keyId, publicKey] of Object.entries(config.receiptKeys)) {
    nonemptyString(keyId, 'config.receiptKeys key ID');
    nonemptyString(publicKey, `config.receiptKeys.${keyId}`);
  }
  for (const origin of config.allowedOrigins) {
    let parsed;
    try {
      parsed = new URL(origin);
    } catch {
      fail('invalid-config', 'config.allowedOrigins contains an invalid origin');
    }
    if (parsed.origin !== origin || parsed.pathname !== '/' || parsed.search || parsed.hash || parsed.username || parsed.password) {
      fail('invalid-config', 'config.allowedOrigins must contain exact URL origins');
    }
  }
  return config;
}

async function loadOfficialClient() {
  const sourcePath = join(
    dirname(fileURLToPath(import.meta.url)),
    'vendor',
    'weles-client',
    'index.mjs',
  );
  let source;
  try {
    source = readFileSync(sourcePath);
  } catch {
    fail('official-client-unavailable', 'the vendored official Weles client source is unreadable');
  }
  const digest = createHash('sha256').update(source).digest('hex');
  if (digest !== CLIENT_SOURCE_SHA256) {
    fail('official-client-mismatch', 'the vendored official Weles client does not match the pinned commit');
  }
  try {
    return await import(pathToFileURL(sourcePath).href);
  } catch {
    fail('official-client-unavailable', 'the vendored official Weles client could not be loaded');
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
  for (const field of KNOWN_TASK_FIELDS) nonemptyString(expected[field], `expectedTask.${field}`);
  if (expected.organizationId !== config.organizationId) {
    fail('expected-claim-mismatch', 'expected organizationId differs from configured receipt trust');
  }
  if (config.allowedOrigins !== null && !config.allowedOrigins.includes(expected.origin)) {
    fail('expected-claim-mismatch', 'expected origin is not an exact protected allowlist member');
  }
  if (!config.allowedActions.includes(expected.action)) {
    fail('expected-claim-mismatch', 'expected action is not an exact configured allowlist member');
  }
  return expected;
}

function validateExpectedClaims(value, config) {
  const expected = plainObject(value, 'expectedClaims');
  onlyKeys(expected, CORE_CLAIMS, 'expectedClaims');
  for (const field of CORE_CLAIMS) nonemptyString(expected[field], `expectedClaims.${field}`);
  validateExpectedTask({
    taskId: expected.taskId,
    organizationId: expected.organizationId,
    origin: expected.origin,
    action: expected.action,
  }, config);
  if (!config.terminalOutcomes.includes(expected.outcome)) {
    fail('expected-claim-mismatch', 'expected outcome is not a configured terminal outcome');
  }
  if (!SHA256.test(expected.evidenceDigest)) {
    fail('expected-claim-mismatch', 'expected evidenceDigest must be a lowercase SHA-256 digest');
  }
  return expected;
}

function validateReceipt(value, strict = true) {
  const receipt = plainObject(value, 'receipt');
  if (strict) onlyKeys(receipt, RECEIPT_FIELDS, 'receipt');
  const retained = {};
  for (const field of RECEIPT_FIELDS) retained[field] = nonemptyString(receipt[field], `receipt.${field}`);
  if (retained.schema !== 'weles.receipt.current') fail('unsupported-receipt', 'receipt schema is unsupported');
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
  if (artifact.bytes !== undefined && (!Number.isSafeInteger(artifact.bytes) || artifact.bytes < 0)) {
    fail('invalid-artifact', 'artifact.bytes must be a non-negative safe integer');
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
    const hash = createHash('sha256');
    for await (const chunk of createReadStream(null, { fd: handle.fd, autoClose: false })) hash.update(chunk);
    const sha256 = hash.digest('hex');
    if (sha256 !== artifact.sha256) fail('artifact-digest-mismatch', 'retained artifact digest differs from the caller expectation');
    if (artifact.bytes !== undefined && stat.size !== artifact.bytes) {
      fail('artifact-size-mismatch', 'retained artifact size differs from the persisted verification document');
    }
    return { path: artifact.path, sha256, bytes: stat.size };
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
    ['keySetVersion', keySetVersion],
    ['artifact.path', artifact.path],
    ['artifact.sha256', artifact.sha256],
  ]) updateFramed(hash, label, value);
  return `sha256:${hash.digest('hex')}`;
}

function buildReceiptCheckpoint(receiptValue, expectedTaskValue, config, verifyReceipt) {
  const receipt = validateReceipt(receiptValue, false);
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

async function buildProvenance(receiptValue, expectedValue, artifactValue, config, verifyReceipt) {
  const receipt = validateReceipt(receiptValue, false);
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
  if (claims.keyId !== receipt.keyId) fail('receipt-key-mismatch', 'verified keyId differs from the retained receipt');
  const artifact = await digestArtifact(artifactExpectation);
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

function networkClient(config, WelesClient) {
  const endpoint = nonemptyString(config.endpoint, 'config.endpoint');
  const bearer = nonemptyString(config.bearer, 'config.bearer');
  return new WelesClient({
    endpoint,
    bearer,
    organizationId: config.organizationId,
    allowedOrigins: config.allowedOrigins,
    allowedActions: config.allowedActions,
    receiptKeys: config.receiptKeys,
  });
}

function canonicalJson(value) {
  if (value === null || typeof value !== 'object') return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  return `{${Object.keys(value)
    .sort()
    .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
    .join(',')}}`;
}

function prepareSubmission(commandValue, config) {
  const command = plainObject(commandValue, 'command');
  if (command.schema !== COMMAND_SCHEMA) fail('unsupported-command', 'bridge command schema is unsupported');
  if (command.operation !== 'submit') fail('unsupported-operation', 'submission preparation requires submit');
  onlyKeys(command, ['schema', 'operation', 'request', 'idempotencyKey'], 'command');
  const request = plainObject(command.request, 'command.request');
  onlyKeys(request, ['origin', 'action', 'input', 'credentialRefs', 'evidencePolicy', 'justification'], 'command.request');
  const origin = nonemptyString(request.origin, 'command.request.origin');
  const action = nonemptyString(request.action, 'command.request.action');
  if (!config.allowedOrigins.includes(origin)) {
    fail('origin-denied', 'command.request.origin is not an exact protected allowlist member');
  }
  if (!config.allowedActions.includes(action)) {
    fail('action-denied', 'command.request.action is not an exact protected allowlist member');
  }
  const normalizedRequest = {
    origin,
    action,
    input: request.input === undefined ? {} : plainObject(request.input, 'command.request.input'),
    credentialRefs: request.credentialRefs === undefined
      ? []
      : stringArray(request.credentialRefs, 'command.request.credentialRefs'),
    evidencePolicy: request.evidencePolicy === undefined
      ? 'receipt'
      : nonemptyString(request.evidencePolicy, 'command.request.evidencePolicy'),
    justification: nonemptyString(request.justification, 'command.request.justification'),
  };
  const idempotencyKey = nonemptyString(command.idempotencyKey, 'command.idempotencyKey');
  const requestDigest = `sha256:${createHash('sha256').update(canonicalJson({
    schema: 'wisent.spis-weles-submit-request.v1',
    organizationId: config.organizationId,
    idempotencyKey,
    request: normalizedRequest,
  })).digest('hex')}`;
  return { request: normalizedRequest, idempotencyKey, requestDigest, origin, action };
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
      'receiptCheckpoint',
    ], 'existing submission output');
  } catch {
    fail('output-conflict', 'existing submission output is not a retained bridge submission');
  }
  const sameRequest = existing.schema === SUBMISSION_SCHEMA
    && existing.requestDigest === prepared.requestDigest
    && existing.idempotencyKey === prepared.idempotencyKey
    && existing.organizationId === config.organizationId
    && existing.origin === prepared.origin
    && existing.action === prepared.action
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
    artifactRefs = stringArray(response.artifactRefs, `${name}.artifactRefs`);
    if (new Set(artifactRefs).size !== artifactRefs.length) {
      fail('invalid-response', `${name}.artifactRefs must not contain duplicates`);
    }
  }
  return { resultRef, artifactRefs };
}

function taskStatusDocument(schema, responseValue, expectedTask, config, verifyReceipt, responseName) {
  const response = plainObject(responseValue, responseName);
  for (const field of KNOWN_TASK_FIELDS) {
    if (nonemptyString(response[field], `${responseName}.${field}`) !== expectedTask[field]) {
      fail('expected-claim-mismatch', `${responseName} ${field} differs from the known task`);
    }
  }
  const status = nonemptyString(response.status, `${responseName}.status`);
  const terminal = !NONTERMINAL_STATUSES.has(status);
  const result = {
    schema,
    ...expectedTask,
    status,
    terminal,
    outcome: null,
    ...exactResultReferences(response, responseName),
  };
  if (!terminal) {
    if (response.outcome !== undefined && response.outcome !== null) {
      fail('status-outcome-mismatch', 'a nonterminal task must not report a terminal outcome');
    }
    return result;
  }
  const outcome = nonemptyString(response.outcome, `${responseName}.outcome`);
  if (status !== outcome || !config.terminalOutcomes.includes(outcome)) {
    fail('status-outcome-mismatch', 'terminal status and configured terminal outcome must match exactly');
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
  if (receiptCheckpoint.claims.outcome !== outcome) {
    fail('status-outcome-mismatch', 'terminal status/outcome differs from the freshly verified receipt');
  }
  result.outcome = outcome;
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
    onlyKeys(command, ['schema', 'operation', 'taskId', 'expectedTask'], 'command');
    const taskId = nonemptyString(command.taskId, 'command.taskId');
    const expectedTask = validateExpectedTask(command.expectedTask, config);
    if (taskId !== expectedTask.taskId) fail('expected-claim-mismatch', 'get taskId differs from expectedTask.taskId');
    let response;
    try {
      response = await networkClient(config, official.WelesClient).get(taskId);
    } catch (error) {
      const code = typeof error?.code === 'string' ? error.code : 'weles-request-failed';
      fail(code, 'the official Weles client get operation failed');
    }
    return taskStatusDocument(
      TASK_STATUS_SCHEMA,
      response,
      expectedTask,
      config,
      official.verifyReceipt,
      'Weles get response',
    );
  }
  if (operation === 'cancel') {
    onlyKeys(command, ['schema', 'operation', 'taskId', 'expectedTask', 'reason', 'idempotencyKey'], 'command');
    const taskId = nonemptyString(command.taskId, 'command.taskId');
    const expectedTask = validateExpectedTask(command.expectedTask, config);
    if (taskId !== expectedTask.taskId) fail('expected-claim-mismatch', 'cancel taskId differs from expectedTask.taskId');
    const reason = nonemptyString(command.reason, 'command.reason');
    const idempotencyKey = nonemptyString(command.idempotencyKey, 'command.idempotencyKey');
    let response;
    try {
      response = await networkClient(config, official.WelesClient).cancel(taskId, {
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
      response = await networkClient(config, official.WelesClient).submit(
        prepared.request,
        { idempotencyKey: prepared.idempotencyKey },
      );
    } catch (error) {
      const code = typeof error?.code === 'string' ? error.code : 'weles-request-failed';
      fail(code, 'the official Weles client submit operation failed');
    }
    plainObject(response, 'Weles submit response');
    const taskId = nonemptyString(response.taskId, 'Weles submit response.taskId');
    const knownTask = {
      taskId,
      organizationId: config.organizationId,
      origin: prepared.origin,
      action: prepared.action,
    };
    const result = {
      schema: SUBMISSION_SCHEMA,
      ...knownTask,
      idempotencyKey: prepared.idempotencyKey,
      requestDigest: prepared.requestDigest,
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
  const config = operation === 'verify' ? loadTrust() : loadConfig();
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
