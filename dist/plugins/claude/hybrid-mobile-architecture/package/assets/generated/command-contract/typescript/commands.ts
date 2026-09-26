// Generated from the command manifest; do not edit.
import { invoke } from "@tauri-apps/api/core";

export const uar_run = (input: RunRequest): Promise<RunAccepted> => invoke<RunAccepted>("uar_run", { input });
export const uar_cancel = (input: CancelRequest): Promise<CancelAccepted> => invoke<CancelAccepted>("uar_cancel", { input });
export const uar_recover = (input: RecoverRequest): Promise<PersistedProjection> => invoke<PersistedProjection>("uar_recover", { input });
