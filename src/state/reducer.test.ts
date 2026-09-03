import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { appReducer, initialState, type AppAction, type AppState } from './reducer.ts';

function reduce(actions: AppAction[], start: AppState = initialState): AppState {
  return actions.reduce(appReducer, start);
}

describe('job processing state machine', () => {
  it('does not stick processing when job_complete arrives before START_JOB', () => {
    const state = reduce([
      { type: 'SET_PROCESSING', isProcessing: true },
      { type: 'COMPLETE_JOB', jobId: 'job-tiny' },
      { type: 'START_JOB', jobId: 'job-tiny' },
    ]);
    assert.equal(state.isProcessing, false);
    assert.equal(state.activeJobId, null);
    assert.equal(state.lastCompletedJobId, 'job-tiny');
  });

  it('does not stick processing when job_started START_JOB is after complete', () => {
    const state = reduce([
      { type: 'COMPLETE_JOB', jobId: 'job-tiny' },
      { type: 'START_JOB', jobId: 'job-tiny' },
    ]);
    assert.equal(state.isProcessing, false);
    assert.equal(state.activeJobId, null);
    assert.equal(state.lastCompletedJobId, 'job-tiny');
  });

  it('clears processing on the normal START_JOB then COMPLETE_JOB order', () => {
    const state = reduce([
      { type: 'SET_PROCESSING', isProcessing: true },
      { type: 'START_JOB', jobId: 'job-ok' },
      { type: 'COMPLETE_JOB', jobId: 'job-ok' },
    ]);
    assert.equal(state.isProcessing, false);
    assert.equal(state.activeJobId, null);
    assert.equal(state.lastCompletedJobId, 'job-ok');
  });

  it('allows a later apply after a completed job', () => {
    const afterFirst = reduce([
      { type: 'START_JOB', jobId: 'job-1' },
      { type: 'COMPLETE_JOB', jobId: 'job-1' },
    ]);
    const nextClick = appReducer(afterFirst, { type: 'SET_PROCESSING', isProcessing: true });
    assert.equal(nextClick.isProcessing, true);
    const nextStart = appReducer(nextClick, { type: 'START_JOB', jobId: 'job-2' });
    assert.equal(nextStart.isProcessing, true);
    assert.equal(nextStart.activeJobId, 'job-2');
    assert.equal(nextStart.lastCompletedJobId, null);
  });

  it('updates displayed path on error when trusted on-disk path is provided', () => {
    const withFile: AppState = {
      ...initialState,
      files: [
        {
          id: 'file-1',
          original_path: '/tmp/old.txt',
          original_name: 'old.txt',
          extension: 'txt',
          file_type: 'document',
          size_bytes: 10,
          thumbnail_data_url: null,
          status: 'processing',
          transformed_name: 'new.txt',
          error: null,
        },
      ],
    };
    const state = appReducer(withFile, {
      type: 'UPDATE_FILE_STATUS',
      fileId: 'file-1',
      status: 'error',
      error: 'CANCELLED: Job was cancelled',
      originalPath: '/tmp/.brp-tmp-hop/old.txt',
      originalName: 'old.txt',
    });
    assert.equal(state.files[0]?.original_path, '/tmp/.brp-tmp-hop/old.txt');
    assert.equal(state.files[0]?.original_name, 'old.txt');
    assert.equal(state.files[0]?.transformed_name, null);
    assert.equal(state.files[0]?.status, 'error');
    assert.equal(state.files[0]?.error, 'CANCELLED: Job was cancelled');
  });

  it('keeps prior path on error when no trusted path is provided', () => {
    const withFile: AppState = {
      ...initialState,
      files: [
        {
          id: 'file-1',
          original_path: '/tmp/old.txt',
          original_name: 'old.txt',
          extension: 'txt',
          file_type: 'document',
          size_bytes: 10,
          thumbnail_data_url: null,
          status: 'processing',
          transformed_name: 'new.txt',
          error: null,
        },
      ],
    };
    const state = appReducer(withFile, {
      type: 'UPDATE_FILE_STATUS',
      fileId: 'file-1',
      status: 'error',
      error: 'BACKUP_FAILED: disk full',
    });
    assert.equal(state.files[0]?.original_path, '/tmp/old.txt');
    assert.equal(state.files[0]?.original_name, 'old.txt');
    assert.equal(state.files[0]?.transformed_name, 'new.txt');
  });
});
