import { describe, it, expect, beforeEach } from 'vitest';
import { setupComputeServiceTest } from './setupComputeServiceTest';
import { EVENT_COMPUTE_FINISHED, EVENT_COMPUTE_STARTED, EVENT_WORKER_IDLE, MSG_IN_COMPUTE, MSG_IN_INIT } from '../../src/services/computeContants';

describe('ComputeService', () => {
  let testSetup;
  let computeService;
  let mockWorker;
  let mockSubscriber;

  beforeEach(() => {
    // Setup fresh instances for each test
    testSetup = setupComputeServiceTest();
    computeService = testSetup.computeService;
    mockWorker = testSetup.mockWorker;
    mockSubscriber = testSetup.mockSubscriber;
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

  it('should notify subscribers when compute starts and finishes', async () => {
    // Initialize the service first
    await testSetup.initializeService();
    
    // Subscribe to compute events
    computeService.subscribe(EVENT_COMPUTE_STARTED, mockSubscriber.onComputeStarted);
    computeService.subscribe(EVENT_COMPUTE_FINISHED, mockSubscriber.onComputeFinished);
    
    // Create test data
    const specs = { param1: 'test', param2: 123 };
    
    // Call compute
    computeService.compute(specs);
    
    // Simulate worker responding to compute
    mockWorker.simulateComputeFinished(computeService);
    
    // Verify subscriber was notified
    expect(testSetup.mockSubscriber.onComputeStarted).toHaveBeenCalledTimes(1);
    expect(testSetup.mockSubscriber.onComputeFinished).toHaveBeenCalledTimes(1);
  });

  it('should notify subscribers when worker is idle', async () => {
    // Initialize the service first
    await testSetup.initializeService();
    
    // Subscribe to compute events
    computeService.subscribe(EVENT_WORKER_IDLE, mockSubscriber.onWorkerIdle);
    
    // Create test data
    const specs = { param1: 'test', param2: 123 };
    
    // Call compute
    computeService.compute(specs);
    
    // Simulate worker responding to compute
    mockWorker.simulateComputeFinished(computeService);

    // Verify subscriber was notified
    expect(testSetup.mockSubscriber.onWorkerIdle).toHaveBeenCalledTimes(1);
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