import type {
  FileInfo,
  PreviewResult,
  JobRecord,
  Settings,
  JobProgressEvent,
  JobCompleteEvent,
} from "@/types";

export interface ActiveJob {
  jobId: string;
  type: "rename" | "convert" | "metadata";
  filesTotal: number;
  filesCompleted: number;
  filesFailed: number;
}

export interface AppState {
  files: FileInfo[];
  previews: PreviewResult[];
  activeJob: ActiveJob | null;
  lastCompletedJobId: string | null;
  settings: Settings;
  history: JobRecord[];
}

export const initialSettings: Settings = {
  theme: "dark",
  accent_color: "blue",
  default_output_dir: null,
  max_parallel_jobs: 0,
  auto_backup: true,
  backup_retention_days: 30,
  last_rename_pattern: null,
  last_convert_format: null,
  file_hard_cap: 5000,
};

export const initialState: AppState = {
  files: [],
  previews: [],
  activeJob: null,
  lastCompletedJobId: null,
  settings: initialSettings,
  history: [],
};

export type AppAction =
  | { type: "ADD_FILES"; files: FileInfo[] }
  | { type: "REMOVE_FILE"; fileId: string }
  | { type: "CLEAR_FILES" }
  | { type: "SET_PREVIEWS"; previews: PreviewResult[] }
  | { type: "START_JOB"; jobId: string; jobType: "rename" | "convert" | "metadata"; filesTotal: number }
  | { type: "UPDATE_FILE_STATUS"; event: JobProgressEvent }
  | { type: "COMPLETE_JOB"; event: JobCompleteEvent }
  | { type: "SET_SETTINGS"; settings: Settings }
  | { type: "SET_HISTORY"; history: JobRecord[] }
  | { type: "ADD_HISTORY_ENTRY"; job: JobRecord };

export function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "ADD_FILES":
      return {
        ...state,
        files: [...state.files, ...action.files],
      };

    case "REMOVE_FILE":
      return {
        ...state,
        files: state.files.filter((f) => f.id !== action.fileId),
        previews: state.previews.filter((p) => p.file_id !== action.fileId),
      };

    case "CLEAR_FILES":
      return {
        ...state,
        files: [],
        previews: [],
      };

    case "SET_PREVIEWS":
      return {
        ...state,
        previews: action.previews,
      };

    case "START_JOB":
      return {
        ...state,
        activeJob: {
          jobId: action.jobId,
          type: action.jobType,
          filesTotal: action.filesTotal,
          filesCompleted: 0,
          filesFailed: 0,
        },
      };

    case "UPDATE_FILE_STATUS": {
      const { event } = action;
      return {
        ...state,
        files: state.files.map((f) =>
          f.id === event.file_id
            ? { ...f, status: event.status === "failed" ? "failed" : event.status === "completed" ? "success" : "processing" }
            : f,
        ),
        activeJob: state.activeJob
          ? {
              ...state.activeJob,
              filesCompleted: event.files_completed,
              filesFailed: event.status === "failed"
                ? state.activeJob.filesFailed + 1
                : state.activeJob.filesFailed,
            }
          : null,
      };
    }

    case "COMPLETE_JOB":
      return {
        ...state,
        activeJob: null,
        lastCompletedJobId: action.event.job_id,
      };

    case "SET_SETTINGS":
      return {
        ...state,
        settings: action.settings,
      };

    case "SET_HISTORY":
      return {
        ...state,
        history: action.history,
      };

    case "ADD_HISTORY_ENTRY":
      return {
        ...state,
        history: [action.job, ...state.history],
      };

    default:
      return state;
  }
}
