import { useState, useEffect, useCallback, useRef } from "react";
import { getJobHistory, getJobDetail, undoJob } from "@/lib/commands";
import type { JobRecord, JobFileRecord } from "@/types";

interface HistoryPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function HistoryPanel({ isOpen, onClose }: HistoryPanelProps) {
  const [jobs, setJobs] = useState<JobRecord[]>([]);
  const [search, setSearch] = useState("");
  const [totalCount, setTotalCount] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(false);
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);
  const [jobFiles, setJobFiles] = useState<JobFileRecord[]>([]);
  const [detailLoading, setDetailLoading] = useState(false);
  const listRef = useRef<HTMLDivElement>(null);

  const loadJobs = useCallback(
    async (offset = 0, append = false) => {
      setLoading(true);
      try {
        const res = await getJobHistory(30, offset, search || undefined);
        setJobs((prev) => (append ? [...prev, ...res.jobs] : res.jobs));
        setTotalCount(res.total_count);
        setHasMore(res.has_more);
      } catch {
        // silently fail — history is non-critical
      } finally {
        setLoading(false);
      }
    },
    [search],
  );

  useEffect(() => {
    if (isOpen) loadJobs();
  }, [isOpen, loadJobs]);

  const loadMore = useCallback(() => {
    if (!loading && hasMore) {
      loadJobs(jobs.length, true);
    }
  }, [loading, hasMore, jobs.length, loadJobs]);

  const handleScroll = useCallback(() => {
    const el = listRef.current;
    if (!el) return;
    if (el.scrollTop + el.clientHeight >= el.scrollHeight - 40) {
      loadMore();
    }
  }, [loadMore]);

  const selectJob = useCallback(async (jobId: string) => {
    setSelectedJobId(jobId);
    setDetailLoading(true);
    try {
      const detail = await getJobDetail(jobId);
      setJobFiles(detail.job.files);
    } catch {
      setJobFiles([]);
    } finally {
      setDetailLoading(false);
    }
  }, []);

  const handleUndo = useCallback(
    async (jobId: string) => {
      try {
        await undoJob(jobId);
        loadJobs();
      } catch {
        // undo failed
      }
    },
    [loadJobs],
  );

  if (!isOpen) return null;

  const typeIcon: Record<string, string> = {
    rename: "R",
    convert: "C",
    metadata: "M",
  };

  const statusColor: Record<string, string> = {
    completed: "text-success",
    partial: "text-warning",
    failed: "text-error",
    rolled_back: "text-text-muted",
    running: "text-accent",
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="flex h-[80vh] w-[600px] flex-col rounded-xl border border-border bg-bg-primary shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <h2 className="text-sm font-semibold text-text-primary">
            Job History{totalCount > 0 && ` (${totalCount})`}
          </h2>
          <button
            onClick={onClose}
            className="rounded-md p-1 text-text-secondary transition-colors hover:bg-bg-card hover:text-text-primary"
            aria-label="Close history"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        {/* Search */}
        <div className="border-b border-border px-4 py-2">
          <input
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search jobs..."
            className="w-full rounded-lg border border-border bg-bg-card px-3 py-1.5 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
          />
        </div>

        <div className="flex min-h-0 flex-1">
          {/* Job list */}
          <div
            ref={listRef}
            onScroll={handleScroll}
            className="w-1/2 overflow-y-auto border-r border-border"
          >
            {jobs.length === 0 && !loading ? (
              <p className="p-4 text-center text-xs text-text-muted">No jobs found</p>
            ) : (
              jobs.map((job) => (
                <button
                  key={job.id}
                  onClick={() => selectJob(job.id)}
                  className={`flex w-full items-start gap-2 border-b border-border/30 px-3 py-2 text-left transition-colors hover:bg-bg-card ${
                    selectedJobId === job.id ? "bg-bg-card" : ""
                  }`}
                >
                  <span className="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded bg-accent/10 text-[10px] font-bold text-accent">
                    {typeIcon[job.operation_type] ?? "?"}
                  </span>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-xs font-medium text-text-primary">
                      {job.description || `${job.operation_type} job`}
                    </p>
                    <div className="flex items-center gap-2 text-[10px] text-text-muted">
                      <span>{job.file_count} files</span>
                      <span className={statusColor[job.status] ?? "text-text-muted"}>
                        {job.status}
                      </span>
                      <span>{new Date(job.created_at).toLocaleDateString()}</span>
                    </div>
                  </div>
                </button>
              ))
            )}
            {loading && (
              <div className="flex justify-center py-3">
                <div className="h-4 w-4 animate-spin rounded-full border-2 border-accent border-t-transparent" />
              </div>
            )}
          </div>

          {/* Job detail */}
          <div className="w-1/2 overflow-y-auto">
            {!selectedJobId ? (
              <p className="p-4 text-center text-xs text-text-muted">
                Select a job to view details
              </p>
            ) : detailLoading ? (
              <div className="flex justify-center py-8">
                <div className="h-5 w-5 animate-spin rounded-full border-2 border-accent border-t-transparent" />
              </div>
            ) : (
              <div className="flex flex-col gap-1">
                {/* Undo button */}
                {jobs.find((j) => j.id === selectedJobId)?.can_undo && (
                  <div className="border-b border-border/50 p-2">
                    <button
                      onClick={() => handleUndo(selectedJobId)}
                      className="w-full rounded-lg border border-border px-3 py-1.5 text-xs text-text-secondary transition-colors hover:border-warning hover:text-warning"
                    >
                      Undo this job
                    </button>
                  </div>
                )}

                {jobFiles.map((file) => (
                  <div
                    key={file.id}
                    className="flex items-center justify-between border-b border-border/30 px-3 py-1.5"
                  >
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-[11px] text-text-muted">
                        {file.original_name}
                      </p>
                      {file.transformed_name && (
                        <p className="truncate text-[11px] text-text-primary">
                          {file.transformed_name}
                        </p>
                      )}
                    </div>
                    <span
                      className={`ml-2 text-[10px] ${
                        file.status === "success"
                          ? "text-success"
                          : file.status === "failed"
                            ? "text-error"
                            : "text-text-muted"
                      }`}
                    >
                      {file.status}
                    </span>
                  </div>
                ))}
                {jobFiles.length === 0 && (
                  <p className="p-4 text-center text-xs text-text-muted">No files in this job</p>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
