import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import type { JobProgressEvent } from '@/types';
import { basename, mapJobProgressToFileUpdate } from './jobProgressMapping.ts';

function progress(overrides: Partial<JobProgressEvent> & Pick<JobProgressEvent, 'status'>): JobProgressEvent {
  return {
    job_id: 'job-1',
    file_id: 'file-1',
    file_name: 'old.txt',
    progress_percent: 0,
    error_message: null,
    files_completed: 1,
    files_total: 1,
    transformed_path: null,
    ...overrides,
  };
}

describe('mapJobProgressToFileUpdate', () => {
  it('trusts transformed_path on completed', () => {
    const update = mapJobProgressToFileUpdate(
      progress({
        status: 'completed',
        transformed_path: '/tmp/renamed.txt',
      })
    );
    assert.equal(update.status, 'done');
    assert.equal(update.originalPath, '/tmp/renamed.txt');
    assert.equal(update.originalName, 'renamed.txt');
  });

  it('trusts transformed_path on failed/cancel when registry moved mid-hop', () => {
    const update = mapJobProgressToFileUpdate(
      progress({
        status: 'failed',
        error_message: 'CANCELLED: Job was cancelled',
        transformed_path: '/tmp/.brp-tmp-abc123/old.txt',
      })
    );
    assert.equal(update.status, 'error');
    assert.equal(update.error, 'CANCELLED: Job was cancelled');
    assert.equal(update.originalPath, '/tmp/.brp-tmp-abc123/old.txt');
    assert.equal(update.originalName, basename('/tmp/.brp-tmp-abc123/old.txt'));
  });

  it('does not trust path on failed when backend sends no transformed_path', () => {
    const update = mapJobProgressToFileUpdate(
      progress({
        status: 'failed',
        error_message: 'BACKUP_FAILED: disk full',
      })
    );
    assert.equal(update.status, 'error');
    assert.equal(update.originalPath, undefined);
    assert.equal(update.originalName, undefined);
  });

  it('ignores transformed_path while still processing', () => {
    const update = mapJobProgressToFileUpdate(
      progress({
        status: 'processing',
        transformed_path: '/tmp/should-not-trust.txt',
      })
    );
    assert.equal(update.status, 'processing');
    assert.equal(update.originalPath, undefined);
    assert.equal(update.originalName, undefined);
  });
});
