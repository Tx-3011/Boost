import test from 'node:test';
import assert from 'node:assert/strict';
import { createTrafficTracker } from './diagnostics.js';

const sample = (address, index, rx, tx) => ({ address, index, received_bytes: rx, sent_bytes: tx, error: null });

test('rates and totals use measured time rather than assuming a fixed poll interval', () => {
  const tracker = createTrafficTracker();
  assert.equal(tracker.record([sample('10.0.0.1', 1, 100, 50)], 0)[0].reading.receivedPerSecond, null);
  const [second] = tracker.record([sample('10.0.0.1', 1, 3100, 1050)], 1500);
  assert.equal(second.reading.receivedPerSecond, 2000);
  assert.equal(second.reading.sentPerSecond, 1000 / 1.5);
  assert.equal(second.reading.receivedSince, 3000);
});

test('disconnections and counter resets never report bogus rates', () => {
  const tracker = createTrafficTracker();
  tracker.record([sample('10.0.0.1', 1, 3000, 500)], 0);
  assert.equal(tracker.record([{ address: '10.0.0.1', index: null, error: 'Disconnected' }], 2000)[0].reading, null);
  assert.equal(tracker.record([sample('10.0.0.1', 1, 10, 2)], 4000)[0].reading.receivedPerSecond, null);
  assert.equal(tracker.record([sample('10.0.0.1', 1, 3, 1)], 6000)[0].reading.receivedSince, 0);
});

test('two addresses on the same interface share counters rather than summing them', () => {
  const tracker = createTrafficTracker();
  tracker.record([sample('10.0.0.1', 1, 100, 0), sample('10.0.0.2', 1, 100, 0)], 0);
  const [first, second] = tracker.record([sample('10.0.0.1', 1, 1100, 0), sample('10.0.0.2', 1, 1100, 0)], 1000);
  assert.equal(first.reading.receivedPerSecond, 1000);
  assert.equal(second.reading.receivedPerSecond, 1000);
});
