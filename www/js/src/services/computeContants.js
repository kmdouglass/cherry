// Worker/main thread messages
// In -> Into worker
// Out -> Out of worker
export const MSG_IN_INIT = "Initialize";
export const MSG_OUT_INIT = "Initialized";
export const MSG_IN_COMPUTE = "Compute";
export const MSG_OUT_COMPUTE = "Computed";

export const EVENT_COMPUTE_REQUESTED = "computeRequested";
export const EVENT_COMPUTE_FINISHED = "computeFinished";
export const EVENT_WORKER_BUSY = "workerBusy";
export const EVENT_WORKER_IDLE = "workerIdle";
