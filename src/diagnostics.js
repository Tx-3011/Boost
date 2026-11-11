export function createTrafficTracker() {
  let previous = new Map();

  return {
    reset() { previous = new Map(); },
    record(rows, now) {
      const current = new Map();
      const readings = new Map();
      for (const row of rows) {
        if (row.error || row.index == null || row.received_bytes == null || row.sent_bytes == null) continue;
        if (readings.has(row.index)) continue; // Multiple IPs on one adapter share one counter.
        const old = previous.get(row.index);
        const reset = !old || row.received_bytes < old.received || row.sent_bytes < old.sent;
        const baseline = reset ? { received: row.received_bytes, sent: row.sent_bytes } : old.baseline;
        const seconds = reset ? 0 : (now - old.time) / 1000;
        const value = {
          received: row.received_bytes,
          sent: row.sent_bytes,
          time: now,
          baseline,
          receivedPerSecond: seconds > 0 ? (row.received_bytes - old.received) / seconds : null,
          sentPerSecond: seconds > 0 ? (row.sent_bytes - old.sent) / seconds : null,
          receivedSince: row.received_bytes - baseline.received,
          sentSince: row.sent_bytes - baseline.sent,
        };
        current.set(row.index, value);
        readings.set(row.index, value);
      }
      previous = current;
      return rows.map((row) => ({ ...row, reading: readings.get(row.index) ?? null }));
    },
  };
}
