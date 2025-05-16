import { describe, it, expect, beforeEach } from 'vitest';
import { setupComputeServiceTest } from './setupComputeServiceTest';
import { EVENT_COMPUTE_FINISHED, EVENT_COMPUTE_REQUESTED, EVENT_WORKER_BUSY, EVENT_WORKER_IDLE, MSG_IN_COMPUTE, MSG_IN_INIT } from '../../src/services/computeContants';

describe('ComputeService', () => {
  let testSetup;
  let computeService;
  let mockWorker;
  let mockSubscriber;
  let mockServiceOnMessage;

  beforeEach(() => {
    // Setup fresh instances for each test
    testSetup = setupComputeServiceTest();
    computeService = testSetup.computeService;
    mockWorker = testSetup.mockWorker;
    mockSubscriber = testSetup.mockSubscriber;
    mockServiceOnMessage = testSetup.mockServiceOnMessage;
  });

  it('should initialize the worker', async () => {
    // Start initialization
    const initPromise = computeService.initWorker();
    
    // Verify initialization message was sent
    expect(mockWorker.postMessage).toHaveBeenCalledTimes(1);
    expect(mockWorker.lastMessageToWorker.msg).toBe(MSG_IN_INIT);
  });

  it('should send compute message to worker', async () => {
    // Initialize the service first
    await testSetup.initializeService();
    
    // Create test data
    const specs = { param1: 'test', param2: 123 };
    
    // Call compute
    computeService.compute(specs);
    
    // Verify compute message was sent with correct format
    expect(mockWorker.postMessage).toHaveBeenCalledTimes(2); // Init + compute
    expect(mockWorker.lastMessageToWorker.msg).toBe(MSG_IN_COMPUTE);
    expect(mockWorker.lastMessageToWorker.specs.param1).toBe('test');
    expect(mockWorker.lastMessageToWorker.specs.param2).toBe(123);
    expect(mockWorker.lastMessageToWorker.requestID).toBe(0);
  });

  it('should notify subscribers when compute is requested and finishes', async () => {
    // Initialize the service first
    await testSetup.initializeService();
    
    // Subscribe to compute events
    computeService.subscribe(EVENT_COMPUTE_REQUESTED, mockSubscriber.onComputeRequested);
    computeService.subscribe(EVENT_COMPUTE_FINISHED, mockSubscriber.onComputeFinished);
    
    // Create test data
    const specs = { param1: 'test', param2: 123 };
    
    // Call compute
    computeService.compute(specs);
    
    // Simulate worker responding to compute
    mockWorker.simulateComputeFinished(computeService);
    
    // Verify subscriber was notified
    expect(testSetup.mockSubscriber.onComputeRequested).toHaveBeenCalledTimes(1);
    expect(testSetup.mockSubscriber.onComputeFinished).toHaveBeenCalledTimes(1);
  });

  it('should notify subscribers when worker is busy or idle', async () => {
    // Initialize the service first
    await testSetup.initializeService();
    
    // Subscribe to compute events
    computeService.subscribe(EVENT_WORKER_BUSY, mockSubscriber.onWorkerBusy);
    computeService.subscribe(EVENT_WORKER_IDLE, mockSubscriber.onWorkerIdle);
    
    // Create test data
    const specs = { param1: 'test', param2: 123 };
    
    // Call compute
    computeService.compute(specs);
    
    // Simulate worker responding to compute
    mockWorker.simulateComputeFinished(computeService);

    // Verify subscriber was notified
    expect(testSetup.mockSubscriber.onWorkerBusy).toHaveBeenCalledTimes(1);
    expect(testSetup.mockSubscriber.onWorkerIdle).toHaveBeenCalledTimes(1);
  });

  it('should handle errors during compute', async () => {
    // Initialize the service first
    await testSetup.initializeService();
    
    // Subscribe to compute finished event
    computeService.subscribe(EVENT_COMPUTE_FINISHED, mockSubscriber.onComputeFinished);
    
    // Create test data
    const specs = { param1: 'test', param2: 123 };
    
    // Call compute
    computeService.compute(specs);
    
    // Simulate worker responding with an error
    mockWorker.simulateComputeError(computeService, 'Test error');
    
    // Verify subscriber was notified with error message
    expect(testSetup.mockSubscriber.onComputeFinished).toHaveBeenCalledTimes(1);
    expect(mockServiceOnMessage).toHaveBeenCalledTimes(1);
    expect(mockServiceOnMessage.mock.calls[0][0].data.errorMessage).toBe('Test error');
  });

  it('should terminate the worker when asked', async () => {
    // Initialize the service first
    await testSetup.initializeService();
    
    // Terminate the worker
    computeService.terminateWorker();
    
    // Verify worker was terminated
    expect(mockWorker.terminate).toHaveBeenCalledTimes(1);
  });
});
